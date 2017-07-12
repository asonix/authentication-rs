/*
 * This file is part of Authentication.
 *
 * Copyright © 2017 Riley Trautman
 *
 * Authentication is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Authentication is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Authentication.  If not, see <http://www.gnu.org/licenses/>.
 */

use diesel;
use diesel::prelude::*;
use error::Result;
use CONFIG;
use models::user::User;
use super::VerificationCode;
use schema::verification_codes;

#[derive(Insertable)]
#[table_name = "verification_codes"]
pub struct NewVerificationCode {
    code: String,
    user_id: i32,
}

impl NewVerificationCode {
    pub fn new_by_username(uname: &str) -> Result<Self> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let user: User = users.filter(username.eq(uname)).first::<User>(db.conn())?;

        Self::new_by_id(user.id())
    }

    pub fn new_by_id(user_id: i32) -> Result<Self> {
        use rand::Rng;
        use rand::OsRng;

        let mut os_rng = OsRng::new()?;

        Ok(NewVerificationCode {
            code: os_rng.gen_ascii_chars().take(30).collect(),
            user_id: user_id,
        })
    }

    pub fn save(&self) -> Result<VerificationCode> {
        use schema::verification_codes;

        let db = CONFIG.db()?;

        let verification_code = diesel::insert(self)
            .into(verification_codes::table)
            .get_result(db.conn())?;

        Ok(verification_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use models::user::new_user::NewUser;
    use authenticatable::Authenticatable;

    #[test]
    fn new_by_username_creates_verification_code() {
        with_user(|user| {
            let result = NewVerificationCode::new_by_username(&user.username());

            assert!(
                result.is_ok(),
                "Failed to create verification_code for user"
            );
        });
    }

    #[test]
    fn new_by_username_fails_with_bad_username() {
        let result = NewVerificationCode::new_by_username("this username doesn't exist");

        assert!(
            !result.is_ok(),
            "Created verification_code for invalid User"
        );
    }

    #[test]
    fn new_by_id_makes_new_verification_code() {
        let result = NewVerificationCode::new_by_id(20);

        assert!(result.is_ok(), "Failed to create verification code");
    }

    #[test]
    fn save_saves_verification_code() {
        use schema::users;

        let auth = Authenticatable::UserAndPass {
            username: &generate_username(),
            password: "P4ssw0rd$.",
        };

        let new_user = NewUser::new(&auth).unwrap();

        let user: User = diesel::insert(&new_user)
            .into(users::table)
            .get_result(CONFIG.db().unwrap().conn())
            .unwrap();

        let new_verification_code = NewVerificationCode::new_by_id(user.id()).unwrap();

        let result = new_verification_code.save();

        assert!(result.is_ok(), "Failed to save verification_code");
        teardown_by_user_id(user.id());
    }

    #[test]
    fn save_fails_with_bad_user_id() {
        let new_verification_code = NewVerificationCode::new_by_id(-1).unwrap();

        let result = new_verification_code.save();

        assert!(
            !result.is_ok(),
            "Created verification_code with bad user_id"
        );
    }

    fn with_user<T>(test: T) -> ()
    where
        T: FnOnce(User) -> () + panic::UnwindSafe,
    {
        let new_user = generate_new_user().expect("Failed to create NewUser for with_user");
        let user = new_user.save().expect("Failed to save User for with_user");

        let u_id = user.id();
        let result = panic::catch_unwind(|| test(user));
        teardown_by_user_id(u_id);
        result.unwrap();
    }

    fn generate_username() -> String {
        use rand::Rng;
        use rand::OsRng;

        OsRng::new().unwrap().gen_ascii_chars().take(10).collect()
    }

    fn test_password_one() -> &'static str {
        "Passw0rd$."
    }

    fn generate_new_user() -> Result<NewUser> {
        let auth = Authenticatable::UserAndPass {
            username: &generate_username(),
            password: test_password_one(),
        };

        NewUser::new(&auth)
    }

    fn teardown_by_user_id(u_id: i32) -> () {
        use schema::users::dsl::*;

        let _ = diesel::delete(users.filter(id.eq(u_id))).execute(CONFIG.db().unwrap().conn());
    }

}

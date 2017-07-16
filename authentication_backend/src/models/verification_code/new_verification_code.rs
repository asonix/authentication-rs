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
use schema::verification_codes;
use models::{User, VerificationCode};
use models::user::UserTrait;

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

        Self::new_by_id(UserTrait::id(&user))
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
    use test_helper::*;
    use models::user::test_helper::{with_user, teardown};
    use models::user::NewUser;
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
            username: &generate_string(),
            password: "P4ssw0rd$.",
        };

        let new_user = NewUser::new(&auth).unwrap();

        let user: User = diesel::insert(&new_user)
            .into(users::table)
            .get_result(CONFIG.db().unwrap().conn())
            .unwrap();

        let new_verification_code = NewVerificationCode::new_by_id(UserTrait::id(&user)).unwrap();

        let result = new_verification_code.save();

        assert!(result.is_ok(), "Failed to save verification_code");
        teardown(UserTrait::id(&user));
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
}

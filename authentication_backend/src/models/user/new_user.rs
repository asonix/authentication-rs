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
use bcrypt::{DEFAULT_COST, hash};
use CONFIG;
use super::{UserTrait, User};
use schema::users;
use error::{InputErrorKind, Error, Result};
use authenticatable::Authenticatable;
use super::helpers::{validate_username, validate_password};

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    username: String,
    password: String,
}

impl NewUser {
    pub fn new(auth: &Authenticatable) -> Result<Self> {
        let (username, password) = match *auth {
            Authenticatable::UserAndPass {
                username: u,
                password: p,
            } => (u, p),
            _ => return Err(Error::InputError(InputErrorKind::Authenticatable)),
        };

        let password = validate_password(password)?;
        let username = validate_username(username)?;

        let hash = hash(password, DEFAULT_COST)?;

        Ok(NewUser {
            username: username.to_string(),
            password: hash,
        })
    }

    pub fn save(&self) -> Result<User> {
        use schema::users;
        use models::verification_code::NewVerificationCode;

        let db = CONFIG.db()?;

        let user: User = diesel::insert(self).into(users::table).get_result(
            db.conn(),
        )?;

        let verification_code = NewVerificationCode::new_by_id(UserTrait::id(&user))?;

        let _ = verification_code.save()?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::user;
    use test_helper::*;

    #[test]
    fn new_creates_new_user() {
        let new_user: Result<NewUser> = user::test_helper::generate_new_user();

        assert!(
            new_user.is_ok(),
            "Valid username and password failed to create NewUser"
        );
    }

    #[test]
    fn new_rejects_empty_usernames() {
        let auth = Authenticatable::UserAndPass {
            username: "",
            password: test_password(),
        };

        let new_user: Result<NewUser> = NewUser::new(&auth);

        assert!(!new_user.is_ok(), "Invalid username still created NewUser");
    }

    #[test]
    fn new_rejects_short_passwords() {
        let auth = Authenticatable::UserAndPass {
            username: &generate_string(),
            password: "4sdf$.",
        };

        let new_user: Result<NewUser> = NewUser::new(&auth);

        assert!(!new_user.is_ok(), "Short password still created NewUser");
    }

    #[test]
    fn new_rejects_weak_passwords() {
        let auth = Authenticatable::UserAndPass {
            username: &generate_string(),
            password: "asdfasdfasdf",
        };

        let new_user: Result<NewUser> = NewUser::new(&auth);

        assert!(!new_user.is_ok(), "Weak password still created NewUser")
    }

    #[test]
    fn save_creates_user() {
        user::test_helper::with_new_user(|new_user| {
            let user: Result<User> = new_user.save();

            assert!(user.is_ok(), "Failed to save NewUser");
            user::test_helper::teardown(UserTrait::id(&user.unwrap()));
        });
    }

    #[test]
    fn save_creates_verification_code() {
        use schema::verification_codes::dsl::*;
        use models::verification_code::VerificationCode;

        user::test_helper::with_new_user(|new_user| {
            let user = new_user.save().expect("Failed to save User");

            let vc = verification_codes
                .filter(user_id.eq(UserTrait::id(&user)))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn());

            assert!(vc.is_ok(), "Failed to create Verification Code for User");

            user::test_helper::teardown(UserTrait::id(&user));
        });
    }

    #[test]
    fn cannot_save_multiple_identical_users() {
        user::test_helper::with_new_user(|new_user| {
            let result: Result<User> = new_user.save();
            let result2: Result<User> = new_user.save();

            assert!(result.is_ok(), "Failed to save user");
            assert!(!result2.is_ok(), "Saved user with same username");

            let user = result.unwrap();

            user::test_helper::teardown(UserTrait::id(&user));
        });
    }
}

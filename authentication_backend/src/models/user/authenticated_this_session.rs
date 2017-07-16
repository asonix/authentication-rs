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
use error::{Error, Result};
use webtoken::Webtoken;
use super::{UserTrait, User, Authenticated};
use super::helpers::{validate_username, validate_password};

pub struct AuthenticatedThisSession {
    id: i32,
    username: String,
    verified: bool,
}

impl UserTrait for AuthenticatedThisSession {
    fn id(&self) -> i32 {
        self.id
    }

    fn username(&self) -> &str {
        &self.username
    }

    fn is_verified(&self) -> bool {
        self.verified
    }
}

impl AuthenticatedThisSession {
    pub fn delete(&self) -> Result<()> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        diesel::delete(users.filter(username.eq(&self.username)))
            .execute(db.conn())?;

        Ok(())
    }

    pub fn create_webtoken(&self) -> Result<Webtoken> {
        if !self.verified {
            return Err(Error::UserNotVerifiedError);
        }

        let token = Webtoken::create(self.id, &self.username)?;

        Ok(token)
    }

    pub fn update_username(&mut self, new_username: &str) -> Result<()> {
        use schema::users::dsl::{users, id, username};

        let new_username = validate_username(new_username)?;

        let db = CONFIG.db()?;

        let _ = diesel::update(users.filter(id.eq(self.id)))
            .set(username.eq(new_username))
            .execute(db.conn())?;

        self.username = new_username.to_string();
        Ok(())
    }

    pub fn update_password(&mut self, new_pass: &str) -> Result<()> {
        use schema::users::dsl::*;

        let new_pass = validate_password(new_pass)?;

        let hash = hash(new_pass, DEFAULT_COST)?;

        let db = CONFIG.db()?;

        let _ = diesel::update(users.filter(id.eq(self.id)))
            .set(password.eq(&hash))
            .execute(db.conn())?;

        Ok(())
    }

    pub fn from_webtoken_and_password(webtoken: &str, password: &str) -> Result<Self> {
        let authenticated = Authenticated::from_webtoken(webtoken)?;

        AuthenticatedThisSession::from_authenticated(&authenticated, password)
    }

    pub fn from_username_and_password(uname: &str, pword: &str) -> Result<Self> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let user: User = users.filter(username.eq(uname)).first(db.conn())?;

        if user.verify_password(pword)? {
            Ok(AuthenticatedThisSession::from_user(&user))
        } else {
            Err(Error::PasswordMatchError)
        }
    }

    fn from_authenticated(auth: &Authenticated, password: &str) -> Result<Self> {
        if auth.verify_password(password)? {
            Ok(AuthenticatedThisSession {
                id: auth.id(),
                username: auth.username().to_owned(),
                verified: auth.is_verified(),
            })
        } else {
            Err(Error::PasswordMatchError)
        }
    }

    fn from_user(user: &User) -> Self {
        AuthenticatedThisSession {
            id: UserTrait::id(user),
            username: user.username().to_owned(),
            verified: user.is_verified(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::user::test_helper::with_user;

    #[test]
    fn update_password_updates_password() {
        with_user(|mut user| {
            let result = user.update_password(test_password(), "P455w0rd$.");

            assert!(result.is_ok(), "Failed to update password");
        });
    }

    #[test]
    fn update_password_fails_with_bad_credentials() {
        with_user(|mut user| {
            let result = user.update_password("not the password", test_password());

            assert!(!result.is_ok(), "Updated password with bad credentials");
        });
    }

    #[test]
    fn update_password_fails_with_weak_password() {
        with_user(|mut user| {
            let result = user.update_password(test_password(), "asdfasdfasdf");

            assert!(!result.is_ok(), "Allowed update to weak password");
        });
    }

    #[test]
    fn update_username_updates_username() {
        with_user(|mut user| {
            let result = user.update_username("some_new_username", test_password());

            assert!(result.is_ok(), "Failed to update username");
        });
    }

    #[test]
    fn update_username_fails_with_empty_username() {
        with_user(|mut user| {
            let result = user.update_username("", test_password());

            assert!(!result.is_ok(), "Updated username to empty string");
        });
    }

    #[test]
    fn update_username_fails_with_bad_password() {
        with_user(|mut user| {
            let result = user.update_username("new_username", "not the password");

            assert!(!result.is_ok(), "Updated username with bad credentials");
        });
    }

    #[test]
    fn create_webtoken_creates_webtoken() {
        with_user(|mut user| {
            user.verify(&CONFIG.db().unwrap());

            let result = user.create_webtoken();

            assert!(result.is_ok(), "Failed to create webtoken");
        });
    }

    #[test]
    fn unverified_users_cant_create_webtoken() {
        with_user(|user| {
            let result = user.create_webtoken();

            assert!(!result.is_ok(), "Unverified User created webtoken");
        });
    }

    #[test]
    fn delete_deletes_existing_user() {
        with_user(|user| {
            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let result = User::delete(&auth);

            assert!(result.is_ok(), "Failed to delete existing user");
        });
    }

    #[test]
    fn delete_deletes_associated_verification_code() {
        with_user(|user| {
            let vc = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn());

            assert!(vc.is_ok(), "Could not get verification_code for user");

            let auth = Authenticatable::UserAndPass {
                username: &user.username(),
                password: test_password(),
            };

            let auth_session =
                User::authenticate_session(&auth).expect("Failed to authenticate User");

            let _ = auth_session.delete().expect("Failed to delete User");

            let vc = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn());

            assert!(!vc.is_ok(), "Verification code still exists after delete");
        });
    }
}

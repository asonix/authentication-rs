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
use schema::users;
use CONFIG;
use authenticatable::{Authenticatable, ToAuth};
use bcrypt::verify;
use error::{Error, InputErrorKind, Result};
use diesel::prelude::*;
use super::{UserTrait, NewUser, Authenticated, AuthenticatedThisSession};

#[derive(Debug, PartialEq, Queryable, Identifiable, AsChangeset, Associations)]
pub struct User {
    id: i32,
    username: String,
    password: String,
    verified: bool,
}

impl UserTrait for User {
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

impl User {
    pub fn create<T>(auth: &T) -> Result<Self>
    where
        T: ToAuth,
    {
        let auth = auth.to_auth();

        let new_user = NewUser::new(&auth)?;

        new_user.save()
    }

    pub fn authenticate<T>(auth: &T) -> Result<Authenticated>
    where
        T: ToAuth,
    {
        match auth.to_auth() {
            Authenticatable::UserToken { user_token: t } => Authenticated::from_webtoken(t),
            _ => {
                let authenticate_session = User::authenticate_session(auth)?;
                Ok(authenticate_session.into())
            }
        }
    }

    pub fn authenticate_session<T>(auth: &T) -> Result<AuthenticatedThisSession>
    where
        T: ToAuth,
    {
        match auth.to_auth() {
            Authenticatable::UserAndPass {
                username: u,
                password: p,
            } => AuthenticatedThisSession::from_username_and_password(u, p),
            Authenticatable::UserTokenAndPass {
                user_token: t,
                password: p,
            } => AuthenticatedThisSession::from_webtoken_and_password(t, p),
            _ => Err(Error::InputError(InputErrorKind::Authenticatable)),
        }
    }

    pub fn find_by_name(u_name: &str) -> Result<Self> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;
        let user = users.filter(username.eq(u_name)).first::<Self>(db.conn())?;

        Ok(user)
    }

    pub fn find_by_id(u_id: i32) -> Result<Self> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;
        let user = users.filter(id.eq(u_id)).first::<Self>(db.conn())?;

        Ok(user)
    }

    pub fn verify_with_code(vc: &str) -> Result<Self> {
        use schema::verification_codes::dsl::{verification_codes, code, user_id};
        use schema::users::dsl::*;
        use models::verification_code::VerificationCode;

        let db = CONFIG.db()?;

        let (_, mut user) = verification_codes
            .inner_join(users)
            .filter(id.eq(user_id))
            .filter(code.eq(vc))
            .first::<(VerificationCode, User)>(db.conn())?;

        if !user.verify() {
            return Err(Error::UserNotVerifiedError);
        }

        let _ = VerificationCode::delete_by_user_id(user.id)?;

        Ok(user)
    }

    pub fn verify(&mut self) -> bool {
        use schema::users::dsl::*;

        let db = match CONFIG.db() {
            Ok(db) => db,
            Err(_) => return false,
        };

        let updated_record = diesel::update(users.filter(id.eq(self.id)))
            .set(verified.eq(true))
            .execute(db.conn());

        match updated_record {
            Ok(_) => {
                self.verified = true;
                true
            }
            Err(_) => false,
        }
    }

    pub fn verify_password(&self, password: &str) -> Result<bool> {
        let verified = verify(password, &self.password)?;

        Ok(verified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::*;
    use models::verification_code::VerificationCode;
    use schema::verification_codes::dsl::{verification_codes, user_id};
    use models::user::test_helper::{with_user, teardown};

    #[test]
    fn find_by_name_finds_user() {
        with_user(|user| {
            let result = User::find_by_name(user.username());

            assert!(result.is_ok(), "Failed to find user");

            let result = result.unwrap();

            assert_eq!(
                result.username,
                user.username,
                "Found user has a different username"
            );
            assert_eq!(result.id, user.id, "Found user has a different id");
        });
    }

    #[test]
    fn find_by_name_fails_with_bad_name() {
        let result = User::find_by_name("This is not a valid username");

        assert!(!result.is_ok(), "Found user with invalid username");
    }

    #[test]
    fn find_by_id_finds_user() {
        with_user(|user| {
            let result = User::find_by_id(UserTrait::id(&user));

            assert!(result.is_ok(), "Failed to find user");

            let result = result.unwrap();

            assert_eq!(
                result.username,
                user.username,
                "Found user has a different username"
            );
            assert_eq!(result.id, user.id, "Found user has a different id");
        });
    }

    #[test]
    fn find_by_id_fails_with_bad_id() {
        let result = User::find_by_id(-1);

        assert!(!result.is_ok(), "Found user with invalid id");
    }

    #[test]
    fn create_creates_user() {
        let auth = Authenticatable::UserAndPass {
            username: &generate_string(),
            password: &test_password(),
        };

        let result = User::create(&auth);

        assert!(result.is_ok(), "Failed to create user");
        teardown(UserTrait::id(&result.unwrap()));
    }

    #[test]
    fn verify_with_code_verifies_user() {
        with_user(|user| {
            let vc = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn())
                .unwrap();

            let result = User::verify_with_code(&vc.code());
            assert!(
                result.is_ok(),
                "Failed to verify user with verification code"
            );
        });
    }

    #[test]
    fn verify_with_code_deletes_code() {
        with_user(|user| {
            let vc = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn())
                .unwrap();

            let user = User::verify_with_code(&vc.code()).unwrap();

            let result = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn());
            assert!(
                !result.is_ok(),
                "Verification code still exists after verify"
            );
        });
    }

    #[test]
    fn verify_verifies_user() {
        with_user(|mut user| {
            let result = user.verify();

            assert!(result, "Failed to verify user");
            assert!(user.verified, "User not verified");
        });
    }

    #[test]
    fn authenticate_gets_user_from_valid_webtoken() {
        with_user(|mut user| {
            user.verify();

            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let auth = User::authenticate_session(&auth).expect("Failed to authenticate User");

            let webtoken = auth.create_webtoken().unwrap();
            let auth = Authenticatable::UserToken { user_token: webtoken.user_token() };

            let result = User::authenticate(&auth);

            assert!(result.is_ok(), "Failed to fetch user from webtoken");

            let user_2 = result.unwrap();
            assert_eq!(
                user.id,
                user_2.id(),
                "Returned user differs from expected user"
            );
        });
    }

    #[test]
    fn authenticate_fails_with_bad_webtoken() {
        with_user(|_| {
            let auth = Authenticatable::UserToken { user_token: "this is not a token" };

            let result = User::authenticate(&auth);

            assert!(!result.is_ok(), "Fetched user from fake webtoken");
        });
    }

    #[test]
    fn authenticate_with_token_and_password_works() {
        with_user(|mut user| {
            assert!(user.verify(), "Failed to verify User");

            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let auth = User::authenticate_session(&auth).expect("Failed to authenticate User");

            let webtoken = auth.create_webtoken().unwrap();

            let auth = Authenticatable::UserTokenAndPass {
                user_token: webtoken.user_token(),
                password: &test_password(),
            };

            let result = User::authenticate(&auth);

            assert!(
                result.is_ok(),
                "Failed to authenticate User with token and password"
            );
        });
    }

    #[test]
    fn authenticate_fails_with_token_and_bad_password() {
        with_user(|mut user| {
            assert!(user.verify(), "Failed to verify User");

            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let auth = User::authenticate_session(&auth).expect("Failed to authenticate User");

            let webtoken = auth.create_webtoken().unwrap();

            let auth = Authenticatable::UserTokenAndPass {
                user_token: webtoken.user_token(),
                password: "this is not the password",
            };

            let result = User::authenticate(&auth);

            assert!(
                !result.is_ok(),
                "Authenticated User with token and bad password"
            );
        });
    }

    #[test]
    fn authenticate_fails_with_bad_username() {
        with_user(|_| {
            let auth = Authenticatable::UserAndPass {
                username: "not the username",
                password: test_password(),
            };

            let result = User::authenticate(&auth);

            assert!(!result.is_ok(), "User should not have been authenticated");
        });
    }

    #[test]
    fn authenticate_fails_with_bad_password() {
        with_user(|user| {
            let auth = Authenticatable::UserAndPass {
                username: &user.username(),
                password: "not the password",
            };

            let result = User::authenticate(&auth);

            assert!(!result.is_ok(), "User should not have been authenticated");
        });
    }

    #[test]
    fn authenticate_authenticates_user() {
        with_user(|user| {
            let auth = Authenticatable::UserAndPass {
                username: &user.username(),
                password: test_password(),
            };

            let result = User::authenticate(&auth);

            assert!(result.is_ok(), "Failed to authenticate user");
        });
    }

    #[test]
    fn verify_password_verifies_password() {
        with_user(|user| {
            let result = user.verify_password(test_password());

            assert!(result.is_ok(), "Failed to verify password");

            let result = result.unwrap();

            assert!(result, "Password was incorrect");
        });
    }

    #[test]
    fn verify_password_fails_with_bad_password() {
        with_user(|user| {
            let result = user.verify_password("This is not the password");

            assert!(result.is_ok(), "Failed to verify password");

            let result = result.unwrap();

            assert!(!result, "Incorrect password was succesfully matched");
        });
    }
}

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
use config::db::DB;
use CONFIG;
use authenticatable::Authenticatable;
use webtoken::Webtoken;
use bcrypt::{DEFAULT_COST, hash, verify, BcryptResult};
use error::{Error, Result};
use diesel::prelude::*;
use self::new_user::NewUser;
use self::helpers::{validate_username, validate_password};

pub mod new_user;
pub mod helpers;

#[cfg(test)]
pub mod test_helper;

#[derive(Debug, PartialEq, Queryable, Identifiable, AsChangeset, Associations)]
pub struct User {
    id: i32,
    username: String,
    password: String,
    verified: bool,
}

impl User {
    pub fn create(auth: &Authenticatable) -> Result<Self> {
        let new_user = NewUser::new(auth)?;

        new_user.save()
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn is_verified(&self) -> bool {
        self.verified
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        use models::user_permission::UserPermission;
        use models::permission::Permission;

        let permission = match Permission::find(permission) {
            Ok(permission) => permission,
            _ => return false,
        };

        UserPermission::has_permission(&self, &permission)
    }

    pub fn give_permission(&self, authorizer: &Self, permission: &str) -> Result<()> {
        use models::user_permission::UserPermission;
        use models::permission::Permission;

        if authorizer.has_permission("admin") {
            let permission = Permission::find(permission)?;

            let _ = UserPermission::create(&self, &permission)?;

            Ok(())
        } else {
            Err(Error::PermissionError)
        }
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

        if !user.verify(&db) {
            return Err(Error::UserNotVerifiedError);
        }

        let _ = diesel::delete(verification_codes.filter(user_id.eq(user.id)))
            .execute(db.conn())?;

        Ok(user)
    }

    fn verify(&mut self, db: &DB) -> bool {
        use schema::users::dsl::*;

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

    fn verify_password(&self, password: &str) -> BcryptResult<bool> {
        verify(password, &self.password)
    }

    pub fn update_password(&mut self, old_pass: &str, new_pass: &str) -> Result<()> {
        use schema::users::dsl::*;

        let new_pass = validate_password(new_pass)?;

        if self.verify_password(old_pass)? {
            let hash = hash(new_pass, DEFAULT_COST)?;

            let db = CONFIG.db()?;

            let _ = diesel::update(users.filter(id.eq(self.id)))
                .set(password.eq(&hash))
                .execute(db.conn())?;

            self.password = hash;
            Ok(())
        } else {
            Err(Error::PasswordMatchError)
        }
    }

    pub fn update_username(&mut self, new_username: &str, password: &str) -> Result<()> {
        use schema::users::dsl::{users, id, username};

        let new_username = validate_username(new_username)?;

        if self.verify_password(password)? {
            let db = CONFIG.db()?;

            let _ = diesel::update(users.filter(id.eq(self.id)))
                .set(username.eq(new_username))
                .execute(db.conn())?;

            self.username = new_username.to_string();
            Ok(())
        } else {
            Err(Error::PasswordMatchError)
        }
    }

    pub fn create_webtoken(&self) -> Result<Webtoken> {
        if !self.verified {
            return Err(Error::UserNotVerifiedError);
        }

        let token = Webtoken::create(self.id, &self.username)?;

        Ok(token)
    }

    pub fn authenticate(auth: &Authenticatable) -> Result<Self> {
        match *auth {
            Authenticatable::UserAndPass {
                username: u,
                password: p,
            } => User::from_username_and_password(u, p),
            Authenticatable::Token { token: t } => User::from_webtoken(t),
            Authenticatable::TokenAndPass {
                token: t,
                password: p,
            } => User::from_webtoken_and_password(t, p),
        }
    }

    fn from_webtoken(webtoken: &str) -> Result<Self> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let (user_id, _) = Webtoken::from_user_token(webtoken)?;

        let user = users
            .filter(verified.eq(true))
            .filter(id.eq(user_id))
            .first::<Self>(db.conn())?;

        Ok(user)
    }

    fn from_webtoken_and_password(webtoken: &str, password: &str) -> Result<Self> {
        let user = User::from_webtoken(webtoken)?;

        if user.verify_password(password)? {
            Ok(user)
        } else {
            Err(Error::PasswordMatchError)
        }
    }

    fn from_username_and_password(uname: &str, pword: &str) -> Result<Self> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let user: User = users.filter(username.eq(uname)).first(db.conn())?;

        if user.verify_password(pword)? {
            Ok(user)
        } else {
            Err(Error::PasswordMatchError)
        }
    }

    pub fn delete(auth: &Authenticatable) -> Result<()> {
        use schema::users::dsl::*;

        let user = match *auth {
            Authenticatable::TokenAndPass {
                token: t,
                password: p,
            } => User::from_webtoken_and_password(t, p)?,
            Authenticatable::UserAndPass {
                username: u,
                password: p,
            } => User::from_username_and_password(u, p)?,
            _ => return Err(Error::PermissionError),
        };

        let db = CONFIG.db()?;

        diesel::delete(users.filter(username.eq(user.username)))
            .execute(db.conn())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::*;
    use models::user;
    use models::verification_code::VerificationCode;
    use schema::verification_codes::dsl::{verification_codes, user_id};

    #[test]
    fn has_permission_verifies_new_user_is_not_admin() {
        user::test_helper::with_user(|user| {
            assert!(!user.has_permission("admin"), "New user is admin");
        });
    }

    #[test]
    fn admin_can_give_permissions_to_non_admins() {
        use models::permission::Permission;
        use models::user_permission::UserPermission;

        user::test_helper::with_user(|admin| {
            let admin_permission =
                Permission::find("admin").expect("Failed to find admin permission");

            let _ = UserPermission::create(&admin, &admin_permission).expect(
                "Failed to make test admin user_permission",
            );

            assert!(
                admin.has_permission("admin"),
                "Failed to make test admin an admin"
            );

            user::test_helper::with_user(|user| {
                let result = user.give_permission(&admin, "admin");

                assert!(result.is_ok(), "Admin failed to give user new permission");
            });
        });
    }

    #[test]
    fn admin_cannot_give_nonexistant_permission() {
        use models::permission::Permission;
        use models::user_permission::UserPermission;

        user::test_helper::with_user(|admin| {
            let admin_permission =
                Permission::find("admin").expect("Failed to find admin permission");

            let _ = UserPermission::create(&admin, &admin_permission).expect(
                "Failed to make test admin user_permission",
            );

            assert!(
                admin.has_permission("admin"),
                "Failed to make test admin an admin"
            );

            user::test_helper::with_user(|user| {
                let result = user.give_permission(&admin, "this is not a permission");

                assert!(
                    !result.is_ok(),
                    "Admin gave a user a nonexistant permission"
                );
            });
        });
    }

    #[test]
    fn create_creates_user() {
        let auth = Authenticatable::UserAndPass {
            username: &generate_string(),
            password: &test_password(),
        };

        let result = User::create(&auth);

        assert!(result.is_ok(), "Failed to create user");
        user::test_helper::teardown(result.unwrap().id());
    }

    #[test]
    fn update_password_updates_password() {
        user::test_helper::with_user(|mut user| {
            let result = user.update_password(test_password(), "P455w0rd$.");

            assert!(result.is_ok(), "Failed to update password");
        });
    }

    #[test]
    fn update_password_fails_with_bad_credentials() {
        user::test_helper::with_user(|mut user| {
            let result = user.update_password("not the password", test_password());

            assert!(!result.is_ok(), "Updated password with bad credentials");
        });
    }

    #[test]
    fn update_password_fails_with_weak_password() {
        user::test_helper::with_user(|mut user| {
            let result = user.update_password(test_password(), "asdfasdfasdf");

            assert!(!result.is_ok(), "Allowed update to weak password");
        });
    }

    #[test]
    fn update_username_updates_username() {
        user::test_helper::with_user(|mut user| {
            let result = user.update_username("some_new_username", test_password());

            assert!(result.is_ok(), "Failed to update username");
        });
    }

    #[test]
    fn update_username_fails_with_empty_username() {
        user::test_helper::with_user(|mut user| {
            let result = user.update_username("", test_password());

            assert!(!result.is_ok(), "Updated username to empty string");
        });
    }

    #[test]
    fn update_username_fails_with_bad_password() {
        user::test_helper::with_user(|mut user| {
            let result = user.update_username("new_username", "not the password");

            assert!(!result.is_ok(), "Updated username with bad credentials");
        });
    }

    #[test]
    fn verify_with_code_verifies_user() {
        user::test_helper::with_user(|user| {
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
        user::test_helper::with_user(|user| {
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
        user::test_helper::with_user(|mut user| {
            let result = user.verify(&CONFIG.db().unwrap());

            assert!(result, "Failed to verify user");
            assert!(user.verified, "User not verified");
        });
    }

    #[test]
    fn create_webtoken_creates_webtoken() {
        user::test_helper::with_user(|mut user| {
            user.verify(&CONFIG.db().unwrap());

            let result = user.create_webtoken();

            assert!(result.is_ok(), "Failed to create webtoken");
        });
    }

    #[test]
    fn unverified_users_cant_create_webtoken() {
        user::test_helper::with_user(|user| {
            let result = user.create_webtoken();

            assert!(!result.is_ok(), "Unverified User created webtoken");
        });
    }

    #[test]
    fn authenticate_gets_user_from_valid_webtoken() {
        user::test_helper::with_user(|mut user| {
            user.verify(&CONFIG.db().unwrap());
            let auth = Authenticatable::Token { token: &user.create_webtoken().unwrap() };

            let result = User::authenticate(&auth);

            assert!(result.is_ok(), "Failed to fetch user from webtoken");

            let user_2 = result.unwrap();
            assert_eq!(
                user.id,
                user_2.id,
                "Returned user differs from expected user"
            );
        });
    }

    #[test]
    fn authenticate_fails_with_bad_webtoken() {
        user::test_helper::with_user(|_| {
            let auth = Authenticatable::Token { token: "this is not a token" };

            let result = User::authenticate(&auth);

            assert!(!result.is_ok(), "Fetched user from fake webtoken");
        });
    }

    #[test]
    fn authenticate_with_token_and_password_works() {
        user::test_helper::with_user(|mut user| {
            assert!(user.verify(&CONFIG.db().unwrap()), "Failed to verify User");

            let auth = Authenticatable::TokenAndPass {
                token: &user.create_webtoken().unwrap(),
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
        user::test_helper::with_user(|mut user| {
            assert!(user.verify(&CONFIG.db().unwrap()), "Failed to verify User");

            let auth = Authenticatable::TokenAndPass {
                token: &user.create_webtoken().unwrap(),
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
        user::test_helper::with_user(|_| {
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
        user::test_helper::with_user(|user| {
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
        user::test_helper::with_user(|user| {
            let auth = Authenticatable::UserAndPass {
                username: &user.username(),
                password: test_password(),
            };

            let result = User::authenticate(&auth);

            assert!(result.is_ok(), "Failed to authenticate user");
        });
    }

    #[test]
    fn delete_deletes_existing_user() {
        user::test_helper::with_user(|user| {
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
        user::test_helper::with_user(|user| {
            let vc = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn());

            assert!(vc.is_ok(), "Could not get verification_code for user");

            let auth = Authenticatable::UserAndPass {
                username: &user.username(),
                password: test_password(),
            };

            let _ = User::delete(&auth);

            let vc = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn());

            assert!(!vc.is_ok(), "Verification code still exists after delete");
        });
    }
}

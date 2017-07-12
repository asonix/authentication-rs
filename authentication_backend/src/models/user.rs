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
use config::DB;
use CONFIG;
use webtoken::Claims;
use bcrypt::{DEFAULT_COST, hash, verify, BcryptResult};
use error::{Error, Result};
use diesel::prelude::*;

pub enum Authenticatable<'a> {
    UserAndPass {
        username: &'a str,
        password: &'a str,
    },
    Token { token: &'a str },
    TokenAndPass { token: &'a str, password: &'a str },
}

#[derive(Debug, PartialEq, Queryable, Identifiable, AsChangeset, Associations)]
pub struct User {
    id: i32,
    username: String,
    password: String,
    verified: bool,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    username: String,
    password: String,
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

    pub fn create_webtoken(&self) -> Result<String> {
        if !self.verified {
            return Err(Error::UserNotVerifiedError);
        }

        let token = Claims::new(self.id, &self.username).to_token()?;

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

        let claims = Claims::from_token(webtoken)?;

        let user = users
            .filter(verified.eq(true))
            .filter(id.eq(claims.user_id()))
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

impl NewUser {
    pub fn new(auth: &Authenticatable) -> Result<Self> {
        let (username, password) = match *auth {
            Authenticatable::UserAndPass {
                username: u,
                password: p,
            } => (u, p),
            _ => return Err(Error::InvalidAuthError),
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
        use models::verification_code::CreateVerificationCode;

        let db = CONFIG.db()?;

        let user: User = diesel::insert(self).into(users::table).get_result(
            db.conn(),
        )?;

        let verification_code = CreateVerificationCode::new_by_id(user.id)?;

        let _ = verification_code.save()?;

        Ok(user)
    }
}

fn validate_password(password: &str) -> Result<&str> {
    if password.len() < 8 {
        return Err(Error::InvalidPasswordError);
    }

    if !CONFIG.password_regex().numbers().is_match(password) {
        return Err(Error::InvalidPasswordError);
    }

    if !CONFIG.password_regex().symbols().is_match(password) {
        return Err(Error::InvalidPasswordError);
    }

    if !CONFIG.password_regex().upper().is_match(password) {
        return Err(Error::InvalidPasswordError);
    }

    if !CONFIG.password_regex().lower().is_match(password) {
        return Err(Error::InvalidPasswordError);
    }

    Ok(password)
}

fn validate_username(username: &str) -> Result<&str> {
    if username.len() < 1 {
        return Err(Error::InvalidUsernameError);
    }

    Ok(username)
}

#[cfg(test)]
mod tests {
    use models::verification_code::VerificationCode;
    use schema::verification_codes::dsl::{verification_codes, user_id};
    use schema::users::dsl::*;
    use super::*;
    use std::panic;

    // User tests

    #[test]
    fn has_permission_verifies_new_user_is_not_admin() {
        with_user(|user| {
            assert!(!user.has_permission("admin"), "New user is admin");
        });
    }

    #[test]
    fn admin_can_give_permissions_to_non_admins() {
        use models::permission::Permission;
        use models::user_permission::UserPermission;

        with_user(|admin| {
            let admin_permission =
                Permission::find("admin").expect("Failed to find admin permission");

            let _ = UserPermission::create(&admin, &admin_permission).expect(
                "Failed to make test admin user_permission",
            );

            assert!(
                admin.has_permission("admin"),
                "Failed to make test admin an admin"
            );

            with_user(|user| {
                let result = user.give_permission(&admin, "admin");

                assert!(result.is_ok(), "Admin failed to give user new permission");
            });
        });
    }

    #[test]
    fn admin_cannot_give_nonexistant_permission() {
        use models::permission::Permission;
        use models::user_permission::UserPermission;

        with_user(|admin| {
            let admin_permission =
                Permission::find("admin").expect("Failed to find admin permission");

            let _ = UserPermission::create(&admin, &admin_permission).expect(
                "Failed to make test admin user_permission",
            );

            assert!(
                admin.has_permission("admin"),
                "Failed to make test admin an admin"
            );

            with_user(|user| {
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
            username: &generate_username(),
            password: &test_password_one(),
        };

        let result = User::create(&auth);

        assert!(result.is_ok(), "Failed to create user");
        teardown_by_id(result.unwrap().id());
    }

    #[test]
    fn update_password_updates_password() {
        with_user(|mut user| {
            let result = user.update_password(test_password_one(), "P455w0rd$.");

            assert!(result.is_ok(), "Failed to update password");
        });
    }

    #[test]
    fn update_password_fails_with_bad_credentials() {
        with_user(|mut user| {
            let result = user.update_password("not the password", test_password_one());

            assert!(!result.is_ok(), "Updated password with bad credentials");
        });
    }

    #[test]
    fn update_password_fails_with_weak_password() {
        with_user(|mut user| {
            let result = user.update_password(test_password_one(), "asdfasdfasdf");

            assert!(!result.is_ok(), "Allowed update to weak password");
        });
    }

    #[test]
    fn update_username_updates_username() {
        with_user(|mut user| {
            let result = user.update_username("some_new_username", test_password_one());

            assert!(result.is_ok(), "Failed to update username");
        });
    }

    #[test]
    fn update_username_fails_with_empty_username() {
        with_user(|mut user| {
            let result = user.update_username("", test_password_one());

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
            let result = user.verify(&CONFIG.db().unwrap());

            assert!(result, "Failed to verify user");
            assert!(user.verified, "User not verified");
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
    fn authenticate_gets_user_from_valid_webtoken() {
        with_user(|mut user| {
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
        with_user(|_| {
            let auth = Authenticatable::Token { token: "this is not a token" };

            let result = User::authenticate(&auth);

            assert!(!result.is_ok(), "Fetched user from fake webtoken");
        });
    }

    #[test]
    fn authenticate_with_token_and_password_works() {
        with_user(|mut user| {
            assert!(user.verify(&CONFIG.db().unwrap()), "Failed to verify User");

            let auth = Authenticatable::TokenAndPass {
                token: &user.create_webtoken().unwrap(),
                password: &test_password_one(),
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
        with_user(|_| {
            let auth = Authenticatable::UserAndPass {
                username: "not the username",
                password: test_password_one(),
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
                password: test_password_one(),
            };

            let result = User::authenticate(&auth);

            assert!(result.is_ok(), "Failed to authenticate user");
        });
    }

    #[test]
    fn delete_deletes_existing_user() {
        with_user(|user| {
            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password_one(),
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
                password: test_password_one(),
            };

            let _ = User::delete(&auth);

            let vc = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn());

            assert!(!vc.is_ok(), "Verification code still exists after delete");
        });
    }

    // NewUser tests

    #[test]
    fn new_creates_new_user() {
        let new_user: Result<NewUser> = generate_new_user();

        assert!(
            new_user.is_ok(),
            "Valid username and password failed to create NewUser"
        );
    }

    #[test]
    fn new_rejects_empty_usernames() {
        let auth = Authenticatable::UserAndPass {
            username: "",
            password: test_password_one(),
        };

        let new_user: Result<NewUser> = NewUser::new(&auth);

        assert!(!new_user.is_ok(), "Invalid username still created NewUser");
    }

    #[test]
    fn new_rejects_short_passwords() {
        let auth = Authenticatable::UserAndPass {
            username: &generate_username(),
            password: "4sdf$.",
        };

        let new_user: Result<NewUser> = NewUser::new(&auth);

        assert!(!new_user.is_ok(), "Short password still created NewUser");
    }

    #[test]
    fn new_rejects_weak_passwords() {
        let auth = Authenticatable::UserAndPass {
            username: &generate_username(),
            password: "asdfasdfasdf",
        };

        let new_user: Result<NewUser> = NewUser::new(&auth);

        assert!(!new_user.is_ok(), "Weak password still created NewUser")
    }

    #[test]
    fn save_creates_user() {
        with_new_user(|new_user| {
            let user: Result<User> = new_user.save();

            assert!(user.is_ok(), "Failed to save NewUser");
            teardown_by_id(user.unwrap().id());
        });
    }

    #[test]
    fn save_creates_verification_code() {
        with_new_user(|new_user| {
            let user = new_user.save().expect("Failed to save User");

            let vc = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn());

            assert!(vc.is_ok(), "Failed to create Verification Code for User");

            teardown_by_id(user.id);
        });
    }

    #[test]
    fn cannot_save_multiple_identical_users() {
        with_new_user(|new_user| {
            let user: Result<User> = new_user.save();
            let user2: Result<User> = new_user.save();

            assert!(user.is_ok(), "Failed to save user");
            assert!(!user2.is_ok(), "Saved user with same username");

            teardown_by_id(user.unwrap().id)
        });
    }

    fn teardown_by_id(u_id: i32) -> () {
        let _ = diesel::delete(users.filter(id.eq(u_id))).execute(CONFIG.db().unwrap().conn());
    }

    fn with_new_user<T>(test: T) -> ()
    where
        T: FnOnce(NewUser) -> () + panic::UnwindSafe,
    {
        let new_user = generate_new_user().expect("Failed to create NewUser for save test");
        panic::catch_unwind(|| test(new_user)).unwrap();
    }

    fn with_user<T>(test: T) -> ()
    where
        T: FnOnce(User) -> () + panic::UnwindSafe,
    {
        let new_user = generate_new_user().expect("Failed to create NewUser for with_user");
        let user = new_user.save().expect(
            "Failed to create User for with_user",
        );

        let u_id = user.id();
        let result = panic::catch_unwind(|| test(user));
        teardown_by_id(u_id);
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
}

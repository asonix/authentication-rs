use diesel;
use schema::users;
use config::DB;
use CONFIG;
use webtoken::Claims;
use bcrypt::{DEFAULT_COST, hash, verify, BcryptResult};
use error::{Error, Result};
use diesel::prelude::*;

#[derive(Queryable, Identifiable, AsChangeset, Associations)]
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
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn is_verified(&self) -> bool {
        self.verified
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

        if !user.verify(db) {
            return Err(Error::UserNotVerifiedError);
        }

        let db = CONFIG.db()?;

        let _ = diesel::delete(verification_codes.filter(code.eq(vc)))
            .execute(db.conn())?;

        Ok(user)
    }

    fn verify(&mut self, db: DB) -> bool {
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

    pub fn from_webtoken(webtoken: String) -> Result<Self> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let claims = Claims::from_token(webtoken)?;

        let user = users
            .filter(verified.eq(true))
            .filter(id.eq(claims.user_id()))
            .first::<Self>(db.conn())?;

        Ok(user)
    }

    pub fn authenticate(uname: &str, pword: &str) -> Result<Self> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let user: User = users.filter(username.eq(uname)).first(db.conn())?;

        if user.verify_password(pword)? {
            Ok(user)
        } else {
            Err(Error::PasswordMatchError)
        }
    }

    pub fn delete(uname: &str, pword: &str) -> Result<()> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let user: User = users.filter(username.eq(uname)).first(db.conn())?;

        if user.verify_password(pword)? {
            diesel::delete(users.filter(username.eq(uname))).execute(
                db.conn(),
            )?;

            Ok(())
        } else {
            Err(Error::PasswordMatchError)
        }
    }
}

impl NewUser {
    pub fn new(username: &str, password: &str) -> Result<Self> {
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

    fn test_password_one() -> &'static str {
        "Passw0rd$."
    }

    // User tests

    #[test]
    fn update_password_updates_password() {
        user_test(|mut user| {
            let result = user.update_password(test_password_one(), "P455w0rd$.");

            assert!(result.is_ok(), "Failed to update password");
        });
    }

    #[test]
    fn update_password_fails_with_bad_credentials() {
        user_test(|mut user| {
            let result = user.update_password("not the password", test_password_one());

            assert!(!result.is_ok(), "Updated password with bad credentials");
        });
    }

    #[test]
    fn update_password_fails_with_weak_password() {
        user_test(|mut user| {
            let result = user.update_password(test_password_one(), "asdfasdfasdf");

            assert!(!result.is_ok(), "Allowed update to weak password");
        });
    }

    #[test]
    fn update_username_updates_username() {
        user_test(|mut user| {
            let result = user.update_username("some_new_username", test_password_one());

            assert!(result.is_ok(), "Failed to update username");
        });
    }

    #[test]
    fn update_username_fails_with_empty_username() {
        user_test(|mut user| {
            let result = user.update_username("", test_password_one());

            assert!(!result.is_ok(), "Updated username to empty string");
        });
    }

    #[test]
    fn update_username_fails_with_bad_password() {
        user_test(|mut user| {
            let result = user.update_username("new_username", "not the password");

            assert!(!result.is_ok(), "Updated username with bad credentials");
        });
    }

    #[test]
    fn verify_with_code_verifies_user() {
        user_test(|user| {
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
        user_test(|user| {
            let vc = verification_codes
                .filter(user_id.eq(user.id))
                .first::<VerificationCode>(CONFIG.db().unwrap().conn())
                .unwrap();

            let user = User::verify_with_code(&vc.code()).unwrap();

            let result = verification_codes.filter(user_id.eq(user.id)).execute(
                CONFIG
                    .db()
                    .unwrap()
                    .conn(),
            );
            assert!(
                !result.is_ok(),
                "Verification code still exists after verify"
            );
        });
    }

    #[test]
    fn verify_verifies_user() {
        user_test(|mut user| {
            let result = user.verify(CONFIG.db().unwrap());

            assert!(result, "Failed to verify user");
            assert!(user.verified, "User not verified");
        });
    }

    #[test]
    fn create_webtoken_creates_webtoken() {
        user_test(|mut user| {
            user.verify(CONFIG.db().unwrap());

            let result = user.create_webtoken();

            assert!(result.is_ok(), "Failed to create webtoken");
        });
    }

    #[test]
    fn from_webtoken_gets_user_from_valid_webtoken() {
        user_test(|mut user| {
            user.verify(CONFIG.db().unwrap());
            let token = user.create_webtoken().unwrap();
            let result = User::from_webtoken(token);

            assert!(result.is_ok(), "Failed to fetch user from webtoken");
        });
    }

    #[test]
    fn from_webtoken_fails_with_bad_webtoken() {
        user_test(|_| {
            let token: String = "this is not a token".to_string();
            let result = User::from_webtoken(token);

            assert!(!result.is_ok(), "Fetched user from fake webtoken");
        });
    }

    #[test]
    fn authenticate_fails_with_bad_username() {
        user_test(|_| {
            let result = User::authenticate("not the username", test_password_one());

            assert!(!result.is_ok(), "User should not have been authenticated");
        });
    }

    #[test]
    fn authenticate_fails_with_bad_password() {
        user_test(|user| {
            let result = User::authenticate(&user.username(), "not the password");

            assert!(!result.is_ok(), "User should not have been authenticated");
        });
    }

    #[test]
    fn authenticate_authenticates_user() {
        user_test(|user| {
            let result = User::authenticate(&user.username(), test_password_one());

            assert!(result.is_ok(), "Failed to authenticate user");
        });
    }

    #[test]
    fn delete_deletes_existing_user() {
        user_test(|user| {
            let result = User::delete(user.username(), test_password_one());

            assert!(result.is_ok(), "Failed to delete existing user");
        });
    }

    #[test]
    fn delete_deletes_associated_verification_code() {
        user_test(|user| {
            let vc = verification_codes.filter(user_id.eq(user.id)).execute(
                CONFIG
                    .db()
                    .unwrap()
                    .conn(),
            );

            assert!(vc.is_ok(), "Could not get verification_code for user");

            let _ = User::delete(user.username(), test_password_one());

            let vc = verification_codes.filter(user_id.eq(user.id)).execute(
                CONFIG
                    .db()
                    .unwrap()
                    .conn(),
            );

            assert!(!vc.is_ok(), "Verification code still exists after delete");
        });
    }

    fn user_test<T>(test: T) -> ()
    where
        T: FnOnce(User) -> () + panic::UnwindSafe,
    {
        use rand::Rng;
        use rand::OsRng;

        let uname: String = OsRng::new().unwrap().gen_ascii_chars().take(10).collect();
        let new_user = NewUser::new(&uname, test_password_one());

        match new_user {
            Ok(new_user) => {
                match new_user.save() {
                    Ok(user) => {
                        let u_id = user.id();
                        let _ = panic::catch_unwind(|| test(user));
                        teardown_by_id(u_id);
                    }
                    Err(_) => {
                        assert!(false, "Failed to create User for user test");
                    }
                };
            }
            Err(_) => {
                assert!(false, "Failed to create NewUser for user test");
            }
        };
    }

    // NewUser tests

    #[test]
    fn new_creates_new_user() {
        let new_user: Result<NewUser> = NewUser::new(&generate_username(), test_password_one());

        assert!(
            new_user.is_ok(),
            "Valid username and password failed to create NewUser"
        );
    }

    #[test]
    fn new_rejects_empty_usernames() {
        let new_user: Result<NewUser> = NewUser::new("", test_password_one());

        assert!(!new_user.is_ok(), "Invalid username still created NewUser");
    }

    #[test]
    fn new_rejects_short_passwords() {
        let new_user: Result<NewUser> = NewUser::new(&generate_username(), "4sdf$.");

        assert!(!new_user.is_ok(), "Short password still created NewUser");
    }

    #[test]
    fn new_rejects_weak_passwords() {
        let new_user: Result<NewUser> = NewUser::new(&generate_username(), "asdfasdfasdf");

        assert!(!new_user.is_ok(), "Weak password still created NewUser")
    }

    #[test]
    fn save_creates_user() {
        save_test(|new_user| {
            let user: Result<User> = new_user.save();

            assert!(user.is_ok(), "Failed to save NewUser");
        });
    }

    #[test]
    fn save_creates_verification_code() {
        save_test(|new_user| {
            let user: Result<User> = new_user.save();

            match user {
                Ok(user) => {
                    let vc = verification_codes.filter(user_id.eq(user.id)).execute(
                        CONFIG
                            .db()
                            .unwrap()
                            .conn(),
                    );

                    assert!(vc.is_ok(), "Failed to create Verification Code for User");
                }
                _ => assert!(false, "Failed to save User"),
            };
        });
    }

    #[test]
    fn cannot_save_multiple_identical_users() {
        save_test(|new_user| {
            let user: Result<User> = new_user.save();
            let user2: Result<User> = new_user.save();

            assert!(user.is_ok(), "Failed to save user");
            assert!(!user2.is_ok(), "Saved user with same username");
        });
    }

    fn teardown_by_id(u_id: i32) -> () {
        let _ = diesel::delete(users.filter(id.eq(u_id))).execute(CONFIG.db().unwrap().conn());
    }

    fn teardown(uname: &str) -> () {
        let _ =
            diesel::delete(users.filter(username.eq(uname))).execute(CONFIG.db().unwrap().conn());
    }

    fn save_test<T>(test: T) -> ()
    where
        T: FnOnce(NewUser) -> () + panic::UnwindSafe,
    {
        let uname = generate_username();
        let new_user = NewUser::new(&uname, test_password_one());

        match new_user {
            Ok(new_user) => {
                let _ = panic::catch_unwind(|| test(new_user));
                ()
            }
            Err(_) => {
                teardown(&uname);
                assert!(false, "Failed to create NewUser for save test");
            }
        };

        teardown(&uname);
    }

    fn generate_username() -> String {
        use rand::Rng;
        use rand::OsRng;

        OsRng::new().unwrap().gen_ascii_chars().take(10).collect()
    }
}

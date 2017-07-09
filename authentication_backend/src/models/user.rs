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

    pub fn verify_with_code(vc: String) -> Result<Self> {
        use schema::verification_codes::dsl::{verification_codes, code, user_id};
        use schema::users::dsl::*;
        use models::verification_code::VerificationCode;

        let db = CONFIG.db()?;

        let (_, mut user) = verification_codes
            .inner_join(users)
            .filter(id.eq(user_id))
            .filter(code.eq(&vc))
            .first::<(VerificationCode, User)>(db.conn())?;

        if !user.verify(db) {
            return Err(Error::UserNotVerifiedError);
        }

        let db = CONFIG.db()?;

        let _ = diesel::delete(verification_codes.filter(code.eq(&vc)))
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
        if self.verify_password(old_pass)? {
            let hash = hash(new_pass, DEFAULT_COST)?;
            self.password = hash;
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
        use schema::verification_codes::dsl::{verification_codes, user_id};
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let user: User = users.filter(username.eq(uname)).first(db.conn())?;

        if user.verify_password(pword)? {
            if !user.is_verified() {
                diesel::delete(verification_codes.filter(user_id.eq(user.id)))
                    .execute(db.conn())?;
            }

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

#[cfg(test)]
mod tests {
    use schema::verification_codes::dsl::{verification_codes, user_id};
    use schema::users::dsl::*;
    use super::*;
    use std::panic;

    #[test]
    fn new_creates_new_user() {
        let new_user: Result<NewUser> = NewUser::new("username", "password$.");

        assert!(
            new_user.is_ok(),
            "Valid username and password failed to create NewUser"
        );
    }

    #[test]
    fn new_rejects_empty_usernames() {
        let new_user: Result<NewUser> = NewUser::new("", "password$.");

        assert!(!new_user.is_ok(), "Invalid username still created NewUser");
    }

    #[test]
    fn new_rejects_short_passwords() {
        let new_user: Result<NewUser> = NewUser::new("username", "asdf$.");

        assert!(!new_user.is_ok(), "Short password still created NewUser");
    }

    #[test]
    fn new_rejects_weak_passwords() {
        let new_user: Result<NewUser> = NewUser::new("username", "asdfasdfasdf");

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

    fn save_setup() -> Result<NewUser> {
        NewUser::new("username", "password$.")
    }

    fn save_teardown() -> () {
        let _ = diesel::delete(users.filter(username.eq("username")))
            .execute(CONFIG.db().unwrap().conn());
        ()
    }

    fn save_test<T>(test: T) -> ()
    where
        T: FnOnce(NewUser) -> () + panic::UnwindSafe,
    {
        let new_user = save_setup();

        match new_user {
            Ok(new_user) => {
                let _ = panic::catch_unwind(|| test(new_user));
                ()
            }
            Err(_) => {
                assert!(false, "Failed to create NewUser for save test");
            }
        };

        save_teardown();
    }
}

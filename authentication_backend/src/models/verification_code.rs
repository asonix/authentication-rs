use diesel;
use schema::verification_codes;
use models::user::User;
use CONFIG;
use error::Result;
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User)]
pub struct VerificationCode {
    id: i32,
    code: String,
    user_id: i32,
}

#[derive(Insertable)]
#[table_name = "verification_codes"]
pub struct CreateVerificationCode {
    code: String,
    user_id: i32,
}

impl VerificationCode {
    pub fn new_by_username(username: &str) -> Result<CreateVerificationCode> {
        CreateVerificationCode::new_by_username(username)
    }

    pub fn new_by_id(user_id: i32) -> Result<CreateVerificationCode> {
        CreateVerificationCode::new_by_id(user_id)
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn user_id(&self) -> i32 {
        self.user_id
    }
}

impl CreateVerificationCode {
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

        Ok(CreateVerificationCode {
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

    #[test]
    fn new_by_username_creates_verification_code() {
        with_user(|user| {
            let result = CreateVerificationCode::new_by_username(&user.username());

            assert!(
                result.is_ok(),
                "Failed to create verification_code for user"
            );
        });
    }

    #[test]
    fn new_by_username_fails_with_bad_username() {
        let result = CreateVerificationCode::new_by_username("this username doesn't exist");

        assert!(
            !result.is_ok(),
            "Created verification_code for invalid User"
        );
    }

    #[test]
    fn new_by_id_makes_create_verification_code() {
        let result = CreateVerificationCode::new_by_id(20);

        assert!(result.is_ok(), "Failed to create verification code");
    }

    #[test]
    fn save_saves_verification_code() {
        use models::user::NewUser;
        use schema::users;

        let new_user = NewUser::new(&generate_username(), "P4ssw0rd$.").unwrap();

        let user: User = diesel::insert(&new_user)
            .into(users::table)
            .get_result(CONFIG.db().unwrap().conn())
            .unwrap();

        let create_verification_code = CreateVerificationCode::new_by_id(user.id()).unwrap();

        let result = create_verification_code.save();

        assert!(result.is_ok(), "Failed to save verification_code");
        teardown_by_user_id(user.id());
    }

    #[test]
    fn save_fails_with_bad_user_id() {
        let create_verification_code = CreateVerificationCode::new_by_id(-1).unwrap();

        let result = create_verification_code.save();

        assert!(
            !result.is_ok(),
            "Created verification_code with bad user_id"
        );
    }

    fn with_user<T>(test: T) -> ()
    where
        T: FnOnce(User) -> () + panic::UnwindSafe,
    {
        use models::user::NewUser;

        let uname: String = generate_username();
        let new_user = NewUser::new(&uname, test_password_one()).expect(
            "Failed to create NewUser for with_user",
        );
        let user = new_user.save().expect("Failed to save User for with_user");

        let u_id = user.id();
        let _ = panic::catch_unwind(|| test(user));
        teardown_by_user_id(u_id);
    }

    fn generate_username() -> String {
        use rand::Rng;
        use rand::OsRng;

        OsRng::new().unwrap().gen_ascii_chars().take(10).collect()
    }

    fn test_password_one() -> &'static str {
        "Passw0rd$."
    }

    fn teardown_by_user_id(u_id: i32) -> () {
        use schema::users::dsl::*;

        let _ = diesel::delete(users.filter(id.eq(u_id))).execute(CONFIG.db().unwrap().conn());
    }
}

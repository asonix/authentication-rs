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
    pub fn new_by_username(username: String) -> Result<CreateVerificationCode> {
        CreateVerificationCode::new_by_username(username)
    }

    pub fn new_by_id(user_id: i32) -> Result<CreateVerificationCode> {
        CreateVerificationCode::new_by_id(user_id)
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn code(&self) -> String {
        self.code.clone()
    }

    pub fn user_id(&self) -> i32 {
        self.user_id
    }
}

impl CreateVerificationCode {
    pub fn new_by_username(uname: String) -> Result<Self> {
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

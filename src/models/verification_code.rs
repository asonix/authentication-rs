extern crate diesel;

use super::super::schema::verification_codes;
use super::user::User;
use super::super::{ DB_POOL, DB };
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User)]
pub struct VerificationCode {
    id: i32,
    code: String,
    user_id: i32,
}

#[derive(Insertable)]
#[table_name="verification_codes"]
pub struct CreateVerificationCode {
    code: String,
    user_id: i32,
}

impl VerificationCode {
    pub fn new_by_username(username: String) -> Result<CreateVerificationCode, String> {
        CreateVerificationCode::new_by_username(username)
    }

    pub fn new_by_id(user_id: i32) -> Result<CreateVerificationCode, String> {
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
    pub fn new_by_username(uname: String) -> Result<Self, String> {
        use super::super::schema::users::dsl::*;
        use super::user::User;

        let db = match DB_POOL.get() {
            Ok(conn) => DB(conn),
            Err(_) => return Err("Could not get the database".to_string()),
        };

        let user = users.filter(username.eq(uname))
            .first::<User>(db.conn());

        let user: User = match user {
            Ok(user) => user,
            Err(_) => return Err("Could not find user".to_string()),
        };

        Self::new_by_id(user.id())
    }

    pub fn new_by_id(user_id: i32) -> Result<Self, String> {
        use rand::Rng;
        use rand::OsRng;

        let mut os_rng = match OsRng::new() {
            Ok(os_rng) => os_rng,
            Err(m) => return Err(m.to_string()),
        };

        Ok(CreateVerificationCode {
            code: os_rng.gen_ascii_chars().take(30).collect(),
            user_id: user_id,
        })
    }

    pub fn save(&self) -> Result<VerificationCode, String> {
        use schema::verification_codes;

        let db = match DB_POOL.get() {
            Ok(conn) => DB(conn),
            Err(_) => return Err("Could not get the database".to_string()),
        };

        let result = diesel::insert(self)
            .into(verification_codes::table)
            .get_result(db.conn());

        match result {
            Ok(verification_code) => Ok(verification_code),
            Err(m) => Err(m.to_string()),
        }
    }
}

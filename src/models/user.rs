extern crate diesel;

use super::super::schema::users;
use super::super::{ DB_POOL, DB, JWT_SECRET };
use bcrypt::{DEFAULT_COST, hash, verify, BcryptResult};
use frank_jwt::{Header, Payload, Algorithm, encode, decode};
use super::super::error::{ Error, Result };
use diesel::prelude::*;

#[derive(Queryable, Identifiable, AsChangeset, Associations)]
pub struct User {
    id: i32,
    username: String,
    password: String,
    verified: bool,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
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
        use super::super::schema::verification_codes::dsl::{verification_codes, code, user_id};
        use super::super::schema::users::dsl::*;
        use super::verification_code::VerificationCode;

        let conn = DB_POOL.get()?;
        let db = DB(conn);

        let (_, mut user) = verification_codes.inner_join(users)
            .filter(id.eq(user_id))
            .filter(code.eq(&vc))
            .first::<(VerificationCode, User)>(db.conn())?;

        if !user.verify(db) {
            return Err(Error::UserNotVerifiedError);
        }

        let conn = DB_POOL.get()?;
        let db = DB(conn);

        let _ = diesel::delete(verification_codes.filter(code.eq(&vc)))
            .execute(db.conn())?;

        Ok(user)
    }

    fn verify(&mut self, db: DB) -> bool {
        use super::super::schema::users::dsl::*;

        let updated_record = diesel::update(users.filter(id.eq(self.id)))
            .set(verified.eq(true))
            .execute(db.conn());

        match updated_record {
            Ok(_) => {
                self.verified = true;
                true
            },
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

        let mut payload = Payload::new();
        payload.insert("id".to_string(), self.id.to_string());
        payload.insert("username".to_string(), self.username.clone());
        let header = Header::new(Algorithm::HS256);

        Ok(encode(header, JWT_SECRET.to_string(), payload.clone()))
    }

    pub fn from_webtoken(webtoken: String) -> Result<Self> {
        use super::super::schema::users::dsl::*;

        let conn = DB_POOL.get()?;
        let db = DB(conn);

        let (_header, payload) = decode(webtoken,
                             JWT_SECRET.to_string(),
                             Algorithm::HS256)?;

        let user_id = match payload.get("id") {
            Some(user_id) => user_id,
            None => return Err(Error::InvalidWebtokenError),
        };

        let user_id: i32 = user_id.parse::<i32>()?;

        let user = users.filter(verified.eq(true))
            .filter(id.eq(user_id))
            .first::<Self>(db.conn())?;

        Ok(user)
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

    pub fn set_password(&mut self, password: &str) -> Result<()> {
        let hash = hash(password, DEFAULT_COST)?;
        self.password = hash;
        Ok(())
    }

    pub fn save(&self) -> Result<User> {
        use schema::users;
        use super::verification_code::CreateVerificationCode;

        let conn = DB_POOL.get()?;
        let db = DB(conn);

        let user: User = diesel::insert(self)
            .into(users::table)
            .get_result(db.conn())?;

        let verification_code = CreateVerificationCode::new_by_id(user.id)?;

        let _ = verification_code.save()?;

        Ok(user)
    }
}

impl CreateUser {
    pub fn insertable(&self) -> Result<NewUser> {
        NewUser::new(&self.username, &self.password)
    }
}

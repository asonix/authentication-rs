extern crate diesel;

use super::super::schema::users;
use super::super::{ DB_POOL, DB, JWT_SECRET };
use bcrypt::{DEFAULT_COST, hash, verify, BcryptResult};
use frank_jwt::{Header, Payload, Algorithm, encode, decode};
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

    pub fn verify_with_code(vc: String) -> Result<Self, String> {
        use super::super::schema::verification_codes::dsl::{verification_codes, code, user_id};
        use super::super::schema::users::dsl::*;
        use super::verification_code::VerificationCode;

        let db = match DB_POOL.get() {
            Ok(conn) => DB(conn),
            Err(m) => return Err(m.to_string()),
        };

        let vc_and_user = verification_codes.inner_join(users)
            .filter(id.eq(user_id))
            .filter(code.eq(&vc))
            .first::<(VerificationCode, User)>(db.conn());

        let mut user = match vc_and_user {
            Ok((_, user)) => user,
            Err(m) => return Err(m.to_string()),
        };

        if !user.verify(db) {
            return Err("User was not verified".to_string());
        }

        let db = match DB_POOL.get() {
            Ok(conn) => DB(conn),
            Err(m) => return Err(m.to_string()),
        };

        let deleted = diesel::delete(verification_codes.filter(code.eq(&vc)))
            .execute(db.conn());

        match deleted {
            Ok(_) => Ok(user),
            Err(m) => Err(m.to_string()),
        }
    }

    fn verify(&mut self, db: DB) -> bool {
        use super::super::schema::users::dsl::*;
        let updated_record: QueryResult<User> = diesel::update(users.filter(id.eq(self.id)))
            .set(verified.eq(true))
            .get_result(db.conn());

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

    pub fn update_password(&mut self, old_pass: &str, new_pass: &str) -> Result<(), &str> {
        match self.verify_password(old_pass) {
            Ok(true) => {
                match hash(new_pass, DEFAULT_COST) {
                    Ok(hash) => {
                        self.password = hash;
                        Ok(())
                    }
                    Err(_) => Err("New password could not be set"),
                }
            },
            Ok(false) => Err("Old password does not match"),
            _ => Err("Something went wrong"),
        }
    }

    pub fn create_webtoken(&self) -> Result<String, &'static str> {
        if !self.verified {
            return Err("User is not verified");
        }

        let mut payload = Payload::new();
        payload.insert("id".to_string(), self.id.to_string());
        payload.insert("username".to_string(), self.username.clone());
        let header = Header::new(Algorithm::HS256);

        Ok(encode(header, JWT_SECRET.to_string(), payload.clone()))
    }

    pub fn from_webtoken(webtoken: String) -> Result<Self, &'static str> {
        use super::super::schema::users::dsl::*;

        let db = match DB_POOL.get() {
            Ok(conn) => DB(conn),
            Err(_) => return Err("Could not get the database"),
        };

        let decoded = decode(webtoken,
                             JWT_SECRET.to_string(),
                             Algorithm::HS256);

        let payload = match decoded {
            Ok((_header, payload)) => payload,
            Err(_) => return Err("Could not decode webtoken"),
        };

        let user_id = match payload.get("id") {
            Some(user_id) => user_id,
            None => return Err("Webtoken is invalid"),
        };

        let user_id: i32 = match user_id.parse::<i32>() {
            Ok(user_id) => user_id,
            Err(_) => return Err("Could not parse user_id from JWT"),
        };

        let user = users.filter(verified.eq(true))
            .filter(id.eq(user_id))
            .first::<Self>(db.conn());

        match user {
            Ok(user) => Ok(user),
            Err(_) => Err("Could not fetch user from DB"),
        }
    }
}

impl NewUser {
    pub fn new(username: &str) -> Self {
        NewUser {
            username: username.to_string(),
            password: "".to_string(),
        }
    }

    pub fn set_password(&mut self, password: &str) -> Result<(), &'static str> {
        match hash(password, DEFAULT_COST) {
            Ok(hash) => {
                self.password = hash;
                Ok(())
            },
            Err(_) => Err("Password could not be set"),
        }
    }

    pub fn save(&self) -> Result<User, String> {
        use schema::users;
        use super::verification_code::CreateVerificationCode;

        let db = match DB_POOL.get() {
            Ok(conn) => DB(conn),
            Err(_) => return Err("Could not get the database".to_string()),
        };

        let result: QueryResult<User> = diesel::insert(self)
            .into(users::table)
            .get_result(db.conn());

        match result {
            Ok(user) => {
                match CreateVerificationCode::new_by_id(user.id) {
                    Ok(verification_code) => {
                        match verification_code.save() {
                            Ok(_) => Ok(user),
                            Err(m) => Err(m.to_string()),
                        }
                    },
                    Err(m) => Err(m.to_string()),
                }
            },
            Err(m) => Err(m.to_string()),
        }
    }
}

impl CreateUser {
    pub fn insertable(&self) -> Result<NewUser, &'static str> {
        let mut new_user = NewUser::new(&self.username);

        match new_user.set_password(&self.password) {
            Ok(()) => Ok(new_user),
            Err(m) => Err(m),
        }
    }
}

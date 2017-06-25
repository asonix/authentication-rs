use super::schema::users;
use bcrypt::{DEFAULT_COST, hash, verify, BcryptResult};
use frank_jwt::{Header, Payload, Algorithm, encode, decode};
use diesel::prelude::*;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    password: String,
    pub verified: bool,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
    password: String,
}

impl User {
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
        
        Ok(encode(header,
                  super::JWT_SECRET.to_string(),
                  payload.clone()))
    }

    pub fn from_webtoken(webtoken: String) -> Result<Self, &'static str> {
        use super::schema::users::dsl::*;

        let db = match super::DB_POOL.get() {
            Ok(conn) => super::DB(conn),
            Err(_) => return Err("Could not get the database"),
        };

        let decoded = decode(webtoken,
                             super::JWT_SECRET.to_string(),
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
    pub fn set_password(&mut self, password: &str) -> Result<(), &str> {
        match hash(password, DEFAULT_COST) {
            Ok(hash) => {
                self.password = hash;
                Ok(())
            },
            Err(_) => Err("Password could not be set"),
        }
    }
}

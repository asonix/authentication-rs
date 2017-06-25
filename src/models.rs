use super::schema::users;
use bcrypt::{DEFAULT_COST, hash, verify, BcryptResult};
use frank_jwt::{Header, Payload, Algorithm, encode, decode};

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

    pub fn create_webtoken(&self) -> String {
        let mut payload = Payload::new();
        payload.insert("id".to_string(), self.id.to_string());
        payload.insert("username".to_string(), self.username.clone());
        let header = Header::new(Algorithm::HS256);
        
        encode(header, super::JWT_SECRET.to_string(), payload.clone())
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

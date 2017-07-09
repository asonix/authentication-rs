use CONFIG;
use jwt::{Header, Algorithm, Validation};
use error::Result;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    user_id: i32,
    username: String,
}

impl Claims {
    pub fn new(user_id: i32, username: &str) -> Self {
        Claims {
            user_id: user_id,
            username: username.to_string(),
        }
    }

    pub fn user_id(&self) -> i32 {
        self.user_id
    }

    pub fn to_token(&self) -> Result<String> {
        let mut header = Header::default();
        header.alg = Algorithm::RS512;

        CONFIG.jwt_secret().encode(&header, &self)
    }

    pub fn from_token(token: &str) -> Result<Self> {
        let validation = &Validation::default();

        CONFIG.jwt_secret().decode(token, validation)
    }
}

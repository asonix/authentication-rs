use CONFIG;
use jwt::{encode, decode, Header, Algorithm, Validation};
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
        header.alg = Algorithm::HS512;

        let secret = &CONFIG.jwt_secret();

        let token = encode(&header, &self, secret.as_ref())?;

        Ok(token)
    }

    pub fn from_token(token: String) -> Result<Self> {
        let validation = &Validation::default();
        let secret = &CONFIG.jwt_secret();

        let token_data = decode::<Claims>(&token, secret.as_ref(), validation)?;

        Ok(token_data.claims)
    }
}

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

    pub fn username(&self) -> &str {
        &self.username
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_token_creates_token() {
        let claims = Claims::new(1, "hello");

        let result = claims.to_token();

        assert!(result.is_ok(), "Failed to create token from claims");
    }

    #[test]
    fn from_token_creates_claims() {
        let claims = Claims::new(1, "hello");

        let token = claims.to_token().expect(
            "Failed to create token from claims",
        );
        let result = Claims::from_token(&token);

        assert!(result.is_ok(), "Failed to get claims from token");

        let result = result.unwrap();

        assert_eq!(
            result.user_id(),
            claims.user_id(),
            "Token returns different user_id from start"
        );
        assert_eq!(
            result.username(),
            claims.username(),
            "Token returns different user_id from start"
        );
    }
}

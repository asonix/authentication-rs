use authentication_backend::models::user::Authenticatable;

pub trait ToAuth {
    fn to_auth(&self) -> Authenticatable;
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

impl ToAuth for CreateUser {
    fn to_auth(&self) -> Authenticatable {
        Authenticatable::UserAndPass {
            username: &self.username,
            password: &self.password,
        }
    }
}

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

impl ToAuth for Token {
    fn to_auth(&self) -> Authenticatable {
        Authenticatable::Token { token: &self.token }
    }
}

#[derive(Deserialize)]
pub struct TokenWithPassword {
    token: String,
    password: String,
}

impl ToAuth for TokenWithPassword {
    fn to_auth(&self) -> Authenticatable {
        Authenticatable::TokenAndPass {
            token: &self.token,
            password: &self.password,
        }
    }
}

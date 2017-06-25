// use super::models::{ User, NewUser };

#[post("/sign-up")]
pub fn sign_up() -> &'static str {
    "{}"
}

#[post("/log-in")]
pub fn log_in() -> &'static str {
    "{}"
}

#[post("/is-authenticated")]
pub fn is_authenticated() -> &'static str {
    "{}"
}

#[post("/verify/<verification_token>")]
pub fn verify(verification_token: String) -> &'static str {
    "{}"
}

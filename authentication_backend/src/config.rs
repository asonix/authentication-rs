use std::env;
use dotenv::dotenv;
use webtoken::Claims;
use jwt;
use jwt::{Header, Validation};
use diesel::pg::PgConnection;
use r2d2;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use error::Result;
use regex::Regex;

type ManagedConnection = ConnectionManager<PgConnection>;
type ConnectionPool = Pool<ManagedConnection>;

pub struct DB(PooledConnection<ManagedConnection>);

impl DB {
    pub fn conn(&self) -> &PgConnection {
        &*self.0
    }
}

pub struct PasswordRegex {
    numbers: Regex,
    symbols: Regex,
    upper: Regex,
    lower: Regex,
}

impl PasswordRegex {
    fn initialize() -> Self {
        PasswordRegex {
            numbers: Regex::new("[0-9]").unwrap(),
            symbols: Regex::new("[!@#$%^&*();\\\\/|<>\"'_+\\-\\.,?=]").unwrap(),
            upper: Regex::new("[A-Z]").unwrap(),
            lower: Regex::new("[a-z]").unwrap(),
        }
    }

    pub fn numbers(&self) -> &Regex {
        &self.numbers
    }

    pub fn symbols(&self) -> &Regex {
        &self.symbols
    }

    pub fn upper(&self) -> &Regex {
        &self.upper
    }

    pub fn lower(&self) -> &Regex {
        &self.lower
    }
}

pub struct JWTSecret<'a> {
    public_key: &'a [u8],
    private_key: &'a [u8],
}

impl<'a> JWTSecret<'a> {
    pub fn encode(&self, header: &Header, claims: &Claims) -> Result<String> {
        let token = jwt::encode(header, claims, self.private_key)?;

        Ok(token)
    }

    pub fn decode(&self, token: &str, validation: &Validation) -> Result<Claims> {
        let token_data = jwt::decode::<Claims>(token, self.public_key, validation)?;

        Ok(token_data.claims)
    }
}

pub struct Config<'a> {
    jwt_secret: JWTSecret<'a>,
    db_pool: ConnectionPool,
    password_regex: PasswordRegex,
}

impl<'a> Config<'a> {
    pub fn initialize() -> Self {
        Config {
            jwt_secret: get_jwt_secret(),
            db_pool: create_db_pool(),
            password_regex: PasswordRegex::initialize(),
        }
    }

    pub fn db(&self) -> Result<DB> {
        Ok(DB(self.db_pool.get()?))
    }

    pub fn jwt_secret(&self) -> &JWTSecret {
        &self.jwt_secret
    }

    pub fn password_regex(&self) -> &PasswordRegex {
        &self.password_regex
    }
}

fn create_db_pool() -> ConnectionPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let kept_url = database_url.clone();

    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::new(config, manager).expect(&format!(
        "Could not create connection pool for: {}",
        kept_url
    ))
}

fn get_jwt_secret<'a>() -> JWTSecret<'a> {
    dotenv().ok();

    JWTSecret {
        private_key: include_bytes!(env!("JWT_PRIVATE_KEY")),
        public_key: include_bytes!(env!("JWT_PUBLIC_KEY")),
    }
}

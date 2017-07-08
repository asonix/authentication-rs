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

type ManagedConnection = ConnectionManager<PgConnection>;
type ConnectionPool = Pool<ManagedConnection>;

pub struct DB(PooledConnection<ManagedConnection>);

impl DB {
    pub fn conn(&self) -> &PgConnection {
        &*self.0
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
}

impl<'a> Config<'a> {
    pub fn initialize() -> Self {
        Config {
            jwt_secret: get_jwt_secret(),
            db_pool: create_db_pool(),
        }
    }

    pub fn db(&self) -> Result<DB> {
        Ok(DB(self.db_pool.get()?))
    }

    pub fn jwt_secret(&self) -> &JWTSecret {
        &self.jwt_secret
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

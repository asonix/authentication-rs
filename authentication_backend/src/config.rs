use std::env;
use dotenv::dotenv;
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

pub struct Config {
    jwt_secret: String,
    db_pool: ConnectionPool,
}

impl Config {
    pub fn initialize() -> Self {
        Config {
            jwt_secret: get_jwt_secret(),
            db_pool: create_db_pool(),
        }
    }

    pub fn db(&self) -> Result<DB> {
        Ok(DB(self.db_pool.get()?))
    }

    pub fn jwt_secret(&self) -> &str {
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

fn get_jwt_secret() -> String {
    dotenv().ok();

    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

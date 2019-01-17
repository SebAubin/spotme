const DB_CONNECTION_TIMEOUT_MS: i64 = 5000;
const DATABASE_NAME: &'static str = "spotme";

mod r2d2_mongo {
    use r2d2;
    use mongodb;
    use mongodb::{Client, ThreadedClient};
    use mongodb::ClientOptions;
    use mongodb::connstring::{self, ConnectionString};
    use mongodb::db::ThreadedDatabase;
    use std::error;
    use std::error::Error as _StdError;
    use std::fmt;
    use mongodb::CommandType;
    use crate::mongo_connection::DATABASE_NAME;

    /// A unified enum of errors returned by redis::Client
    #[derive(Debug)]
    pub enum Error {
        Other(String),
    }

    impl fmt::Display for Error {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self.cause() {
                Some(cause) => write!(fmt, "{}: {}", self.description(), cause),
                None => write!(fmt, "{}", self.description()),
            }
        }
    }

    impl error::Error for Error {
        fn description(&self) -> &str {
            match *self {
                Error::Other(ref err) => err.as_str()
            }
        }

        fn cause(&self) -> Option<&error::Error> {
            match *self {
                Error::Other(ref _err) => None
            }
        }
    }

    #[derive(Debug)]
    pub struct MongoConnectionManager {
        parsed_conn_string: ConnectionString,
    }

    impl MongoConnectionManager {
        pub fn new(connection_str: &str) -> Result<MongoConnectionManager, Error> {
            if let Ok(parsed_conn_string) = connstring::parse(connection_str) {
                return Ok(MongoConnectionManager {
                    parsed_conn_string,
                });

            }

            Err(Error::Other("Bad connection uri".to_string()))
        }
    }

    impl r2d2::ManageConnection for MongoConnectionManager {
        type Connection = mongodb::db::Database;
        type Error = Error;

        fn connect(&self) -> Result<Self::Connection, Error> {
            let host = self.parsed_conn_string.hosts[0].host_name.clone();
            let port = self.parsed_conn_string.hosts[0].port.clone();
            let user = self.parsed_conn_string.user.clone();
            let pass = self.parsed_conn_string.password.clone();
            let db_name = DATABASE_NAME.to_string();

            let client_options = ClientOptions::new();

            let client = Client::connect_with_options(&host, port, client_options).expect("Bad client");

            if let (Some(username), Some(password)) = (user, pass) {
                let auth_db = client.db("admin");

                auth_db.auth(
                    &username,
                    &password,
                ).map_err(|e| Error::Other(e.to_string()))?;
            }

            let db = client.db(db_name.as_str());

            let coll = db.collection("connection_test");

            let doc = doc! { "test": 1 };

            if let Ok(_) = coll.insert_one(doc, None) {
                return Ok(db);
            } else {
                return Err(Error::Other("Unable to set up client properly".to_string()));
            }
        }

        fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Error> {
            let cmd = doc! { "ping": 1 };
            conn.command(cmd, CommandType::Suppressed, None).map_err(|e| { Error::Other(e.to_string()) }).map(|_| { () })
        }

        fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
            false
        }
    }
}

use r2d2;
use r2d2::Pool;
use mongodb::CommandType;
use mongodb::db::ThreadedDatabase;

pub struct MongoConnection {
    pool: Pool<r2d2_mongo::MongoConnectionManager>
}

impl MongoConnection {
    pub fn new(mongo_url: &str) -> Result<Self, String> {
        if let Ok(manager) = r2d2_mongo::MongoConnectionManager::new(mongo_url) {
            if let Ok(pool) = r2d2::Pool::builder()
                .max_size(15)
                .min_idle(Some(0))
                .connection_timeout(::std::time::Duration::from_millis(DB_CONNECTION_TIMEOUT_MS as u64))
                .build(manager) {
                return Ok(MongoConnection {
                    pool,
                });
            }
        }

        Err("Unable to connect to mongo db".to_string())
    }

    pub fn get(&self) -> Result<r2d2::PooledConnection<r2d2_mongo::MongoConnectionManager>, String> {
        self.pool.get().map_err(|e| { e.to_string() })
    }

    pub fn ping(&self) -> Result<(), String> {
        let cmd = doc! { "ping": 1 };
        self.get()?.command(cmd, CommandType::Suppressed, None).map_err(|e| { e.to_string() }).map(|_| { () })
    }
}

impl Clone for MongoConnection {
    fn clone(&self) -> Self {
        MongoConnection {
            pool: self.pool.clone(),
        }
    }
}

unsafe impl Send for MongoConnection {}

unsafe impl Sync for MongoConnection {}

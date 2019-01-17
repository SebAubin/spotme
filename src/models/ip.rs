use crate::models::{Repository, RepositoryError};
use crate::mongo_connection::MongoConnection;
use mongodb::db::ThreadedDatabase;
use mongodb::coll::Collection;
use bson::oid::ObjectId;

fn default_bson_id() -> ObjectId {
    ObjectId::new().unwrap()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ip {
    #[serde(rename = "_id")]
    #[serde(default = "default_bson_id")]
    pub id: ObjectId,
    pub network: String,
    pub geoname_id: String,
    pub latitude: String,
    pub longitude: String,
    pub accuracy_radius: String,
    pub net: i32,
    pub sub: i32,
    pub sub2: i32,
    pub netmask: String
}

impl Ip {
    pub fn new() -> Self{
        Ip {
            id: ObjectId::new().unwrap(),
            network: String::new(),
            geoname_id: String::new(),
            latitude: String::new(),
            longitude: String::new(),
            accuracy_radius: String::new(),
            net: 0,
            sub: 0,
            sub2: 0,
            netmask: String::new()
        }
    }
}

pub struct IpRepository {
    db_instance: Option<MongoConnection>,
}

impl Default for IpRepository {
    fn default() -> Self {
        IpRepository {
            db_instance: None,
        }
    }
}

impl Clone for IpRepository {
    fn clone(&self) -> Self {
        if let Some(ref db) = self.db_instance {
            IpRepository {
                db_instance: Some(db.clone()),
            }
        } else {
            IpRepository {
                db_instance: None,
            }
        }
    }
}

impl Repository for IpRepository {
    type Model = Ip;

    fn init(&mut self, db_instance: MongoConnection) -> Result<(), RepositoryError> {
        self.db_instance = Some(db_instance);
        Ok(())
    }

    fn get_collection(&self) -> Result<Collection, RepositoryError> {
        if let Some(ref db) = self.db_instance {
            Ok(db.get()?.collection("ip"))
        } else {
            Err(RepositoryError::UninitializedRepoError)
        }
    }
}
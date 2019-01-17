use crate::models::{Repository, RepositoryError};
use crate::mongo_connection::MongoConnection;
use mongodb::db::ThreadedDatabase;
use mongodb::coll::Collection;
use bson::oid::ObjectId;

fn default_bson_id() -> ObjectId {
    ObjectId::new().unwrap()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    #[serde(rename = "_id")]
    #[serde(default = "default_bson_id")]
    pub id: ObjectId,
    pub geoname_id: String,
    pub continent_name: String,
    pub country_name: String,
    pub subdivision_1_name: String,
    pub subdivision_2_name: String,
    pub city_name: String,
    pub time_zone: String,
}

impl Location {
    pub fn new() -> Self{
        Location{
            id: ObjectId::new().unwrap(),
            geoname_id: String::new(),
            continent_name: String::new(),
            country_name: String::new(),
            subdivision_1_name: String::new(),
            subdivision_2_name: String::new(),
            city_name: String::new(),
            time_zone: String::new()
        }
    }
}

pub struct LocationRepository {
    db_instance: Option<MongoConnection>,
}

impl Default for LocationRepository {
    fn default() -> Self {
        LocationRepository {
            db_instance: None,
        }
    }
}

impl Clone for LocationRepository {
    fn clone(&self) -> Self {
        if let Some(ref db) = self.db_instance {
            LocationRepository {
                db_instance: Some(db.clone()),
            }
        } else {
            LocationRepository {
                db_instance: None,
            }
        }
    }
}

impl Repository for LocationRepository {
    type Model = Location;

    fn init(&mut self, db_instance: MongoConnection) -> Result<(), RepositoryError> {
        self.db_instance = Some(db_instance);
        Ok(())
    }

    fn get_collection(&self) -> Result<Collection, RepositoryError> {
        if let Some(ref db) = self.db_instance {
            Ok(db.get()?.collection("location"))
        } else {
            Err(RepositoryError::UninitializedRepoError)
        }
    }
}
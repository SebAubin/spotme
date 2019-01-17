pub mod location;
pub mod ip;

use bson::Bson;
use bson::Document;
use bson::{to_bson, from_bson};
use mongodb::coll::results::InsertManyResult;
use bson::oid::ObjectId;
use mongodb::coll::options::AggregateOptions;
use mongodb::coll::results::UpdateResult;
use mongodb::coll::options::FindOptions;

#[allow(dead_code)]
#[derive(Debug)]
pub enum RepositoryError {
    BsonEncodeError(::bson::EncoderError),
    BsonDecodeError(::bson::DecoderError),
    MongoError(::mongodb::Error),
    UninitializedRepoError,
    InsertError,
    UpdateError,
    Other(String),
}

impl From<String> for RepositoryError {
    fn from(e: String) -> Self {
        RepositoryError::Other(e)
    }
}

impl From<::bson::EncoderError> for RepositoryError {
    fn from(e: ::bson::EncoderError) -> Self {
        RepositoryError::BsonEncodeError(e)
    }
}

impl From<::bson::DecoderError> for RepositoryError {
    fn from(e: ::bson::DecoderError) -> Self {
        RepositoryError::BsonDecodeError(e)
    }
}

impl From<::mongodb::Error> for RepositoryError {
    fn from(e: ::mongodb::Error) -> Self {
        RepositoryError::MongoError(e)
    }
}

#[derive(Clone)]
pub struct RepositoryCollection {
    db_instance: crate::mongo_connection::MongoConnection,
    pub ip: ip::IpRepository,
    pub location: location::LocationRepository,
}

impl RepositoryCollection {
    pub fn new(db: crate::mongo_connection::MongoConnection) -> Self {
        RepositoryCollection {
            db_instance: db,
            ip: Default::default(),
            location: Default::default()
        }
    }

    pub fn load_repositories(&mut self) -> Result<(), RepositoryError>{
        self.ip.init(self.db_instance.clone())?;
        self.location.init(self.db_instance.clone())?;
        Ok(())
    }
}

pub mod oid{
    use bson::oid::ObjectId;
    use serde::{Deserialize, Serializer, Deserializer};

    pub fn serialize<S>(
        id: &ObjectId,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        //let s = format!("{}", id.to_hex());
        serializer.serialize_some(&id)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<ObjectId, D::Error> where D: Deserializer<'de>,
    {
        let s = ObjectId::deserialize(deserializer)?;
        Ok(s)
    }
}

pub trait Repository {
    type Model;

    fn init(&mut self, db_instance: crate::mongo_connection::MongoConnection) -> Result<(), RepositoryError>;
    fn get_collection(&self) -> Result<::mongodb::coll::Collection, RepositoryError>;

    fn insert(&self, model: <Self as Repository>::Model) -> Result<Option<Bson>, RepositoryError> where <Self as Repository>::Model: ::serde::Serialize {
        let serialized_model = to_bson(&model)?;

        if let Bson::Document(document) = serialized_model {
            let inserted = self.get_collection()?.insert_one(document, None)?;
            Ok(inserted.inserted_id)
        } else {
            Err(RepositoryError::InsertError)
        }
    }

    fn insert_many(&self, models: Vec<<Self as Repository>::Model>) -> InsertManyResult<> where <Self as Repository>::Model: ::serde::Serialize {
        let mut documents = Vec::new();
        for model in models {
            if let Ok(serialized_model) = to_bson(&model) {
                if let Bson::Document(document) = serialized_model {
                    documents.push(document);
                }
            }
        }

        if let Ok(collection) = self.get_collection(){
            if let Ok(res) = collection.insert_many(documents, None){
                return res;
            }
        }

        InsertManyResult::new(None, None)
    }

    fn update(&self, doc: Document, model: <Self as Repository>::Model) -> Result<UpdateResult, RepositoryError> where <Self as Repository>::Model: ::serde::Serialize {
        let serialized_model = to_bson(&model)?;

        if let Bson::Document(mut document) = serialized_model {
            let _res = document.remove("_id"); // if there is an id field removes it. Replace one does not work on data targeting the id field index
            let result = self.get_collection()?.replace_one(doc, document, None)?;
            Ok(result)
        } else {
            Err(RepositoryError::UpdateError)
        }
    }

    fn update_by_id(&self, bson_id: ObjectId, model: <Self as Repository>::Model) -> Result<UpdateResult, RepositoryError> where <Self as Repository>::Model: ::serde::Serialize {
        self.update(doc! { "_id": bson_id }, model)
    }

    fn delete(&self, doc: Document) -> Result<(), RepositoryError> {
        self.get_collection()?.delete_one(doc, None)?;
        Ok(())
    }

    fn delete_by_id(&self, bson_id: ObjectId) -> Result<(), RepositoryError> {
        self.delete(doc! { "_id": bson_id })
    }

    fn count(&self, doc: Option<Document>) -> Result<i64, RepositoryError> {
        self.get_collection()?.count(doc, None).map_err(|e| e.into())
    }

    fn get(&self, doc: Document) -> Result<Option<<Self as Repository>::Model>, RepositoryError> where <Self as Repository>::Model: ::serde::Deserialize<'static> {
        let document_opt = self.get_collection()?.find_one(Some(doc), None)?;

        if let Some(doc) = document_opt {
            let model = from_bson(Bson::Document(doc))?;
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }

    fn get_by_id(&self, bson_id: ObjectId) -> Result<Option<<Self as Repository>::Model>, RepositoryError> where <Self as Repository>::Model: ::serde::Deserialize<'static> {
        self.get(doc! { "_id": bson_id })
    }

    fn get_all(&self) -> Result<Vec<<Self as Repository>::Model>, RepositoryError> where <Self as Repository>::Model: ::serde::Deserialize<'static> {
        let mut model_vec = Vec::new();
        let documents_cursor = self.get_collection()?.find(None, None)?;

        for doc_res in documents_cursor {
            if let Ok(model_document) = doc_res {
                if let Ok(model) = from_bson(Bson::Document(model_document)) {
                    model_vec.push(model);
                }
            }
        }

        Ok(model_vec)
    }

    fn find(&self, doc: Document) -> Result<Vec<<Self as Repository>::Model>, RepositoryError> where <Self as Repository>::Model: ::serde::Deserialize<'static> {
        let mut model_vec = Vec::new();
        let documents_cursor = self.get_collection()?.find(Some(doc), None)?;

        for doc_res in documents_cursor {
            if let Ok(model_document) = doc_res {
                if let Ok(model) = from_bson(Bson::Document(model_document)) {
                    model_vec.push(model);
                }
            }
        }

        Ok(model_vec)
    }

    fn find_with_options(&self, doc: Document, options: FindOptions) -> Result<Vec<<Self as Repository>::Model>, RepositoryError> where <Self as Repository>::Model: ::serde::Deserialize<'static> {
        let mut model_vec = Vec::new();
        let documents_cursor = self.get_collection()?.find(Some(doc), Some(options))?;

        for doc_res in documents_cursor {
            if let Ok(model_document) = doc_res {
                if let Ok(model) = from_bson(Bson::Document(model_document)) {
                    model_vec.push(model);
                }
            }
        }

        Ok(model_vec)
    }

    fn find_with_pipeline(&self, docs: Vec<Document>, options: Option<AggregateOptions>) -> Result<Vec<Document>, RepositoryError> where Document: ::serde::Deserialize<'static> {
        let mut model_vec = Vec::new();
        let documents_cursor = self.get_collection()?.aggregate(docs, options);

        if let Ok(mut cursor) = documents_cursor {
            if let Ok(documents) = cursor.drain_current_batch(){
                for document in documents{
                    model_vec.push(document);
                }
            }
        }

        Ok(model_vec)
    }

    fn find_models_with_pipeline(&self, docs: Vec<Document>, options: Option<AggregateOptions>) -> Result<Vec<<Self as Repository>::Model>, RepositoryError> where <Self as Repository>::Model: ::serde::Deserialize<'static> {
        let mut model_vec = Vec::new();
        let documents_cursor = self.get_collection()?.aggregate(docs, options);

        if let Ok(mut cursor) = documents_cursor {
            if let Ok(documents) = cursor.drain_current_batch(){
                for document in documents{
                    if let Ok(model) = from_bson(Bson::Document(document)) {
                        model_vec.push(model);
                    }
                }
            }
        }

        Ok(model_vec)
    }
}
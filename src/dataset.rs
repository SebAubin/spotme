use std::error::Error;
use csv::Reader;
use std::fs::File;
use std::net::SocketAddr;
use std::net::IpAddr;

lazy_static!{
    pub static ref DATASET_LOCATION: Dataset = Dataset::new("dataset/location.csv");
}

pub struct Dataset{
    reader: Option<Reader<File>>
}

impl Dataset{
    pub fn new(path: &str) -> Self{
        if let Ok(p) = Reader::from_path(path){
            Dataset{
                reader: Some(p)
            }
        }else{
            Dataset{
                reader: None
            }
        }
    }

    pub fn get_from_addr(&self, addr: SocketAddr){
        println!("Converted to u32: {}", addr.ip());
    }
}

pub fn get_location(){

}
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1}; 

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Torrent {
    pub announce: String,
    pub info: Info,
    pub comment: Option<String>,
    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,
}

impl Torrent {
    pub fn hash_info(&self) -> [u8; 20] {

        let encoded_info: Vec<u8> = serde_bencode::to_bytes(&self.info).unwrap();
        let mut hasher = Sha1::new();
        hasher.update(&encoded_info);
        let result = hasher.finalize();
      
        result.into()
    }
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Info {
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    pub pieces: ByteBuf,
    pub length: i64,
}
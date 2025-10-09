pub mod torrent;
pub mod tracker_request;

use reqwest::Client;
use std::fs;

use crate::torrent::Torrent;

#[tokio::main]
async fn main() {
    let torrent_file = fs::read("assets/ubuntu-25.04-desktop-amd64.iso.torrent").unwrap();

    let decoded: Torrent = serde_bencode::from_bytes(&torrent_file).unwrap();

    println!("Announce: {}", decoded.announce);
    println!("Name: {}", decoded.info.name);
    println!("Piece length in KiB: {}", decoded.info.piece_length / 1024);
    println!("Number of pieces: {}", decoded.info.pieces.len() / 20);
    println!("Length in MiB: {}", decoded.info.length / (1024 * 1024));

    if let Some(comment) = &decoded.comment {
        println!("Comment: {}", comment);
    }
    if let Some(creation_date) = &decoded.creation_date {
        println!("Creation date (Unix timestamp): {}", creation_date);
    }

    let hashed_info = decoded.hash_info();

    let request = tracker_request::TrackerRequest {
        info_hash: url_encode(&hashed_info),
        peer_id: "-TR2940-6wfG2wk6wWLc".to_string(),
        port: 6881,
        uploaded: 0,
        downloaded: 0,
        left: decoded.info.length as u64,
        compact: 1,
        // event: Some("started".to_string()), // Optional event
    };

    
    println!("Requesting tracker with params: {:?}", request);

    let client = Client::new();

    let response = client
        .get(&decoded.announce)
        .query(&request)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await;

    let response2 = response.unwrap();
    
    println!("Address {:?} ",response2.remote_addr().unwrap());

    let body = response2.bytes().await.unwrap();

    let content = str::from_utf8(&body).unwrap();

    print!("Response: {}", &content)
}

fn url_encode(data: &[u8]) -> String {
    data.iter().map(|byte| format!("%{:02X}", byte)).collect()
}

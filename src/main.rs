pub mod handshake;
pub mod torrent;
pub mod tracker_request;
pub mod tracker_response;

use core::hash;
use reqwest::Client;
use serde_bytes::ByteBuf;
use std::{fs, net::SocketAddrV4};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{handshake::Handshake, torrent::Torrent, tracker_response::TrackerResponse};

#[tokio::main]
async fn main() {
    let torrent_file = fs::read("assets/ubuntu-25.04-desktop-amd64.iso.torrent").unwrap();
    let torrent_metadata: Torrent = serde_bencode::from_bytes(&torrent_file).unwrap();


    let hashed_info = torrent_metadata.hash_info();

    let request = tracker_request::TrackerRequest {
        peer_id: "00112233445566778899".to_string(),
        port: 6881,
        uploaded: 0,
        downloaded: 0,
        left: torrent_metadata.info.length as u64,
        compact: 1,
    };


    let client = Client::new();

    let query_params = serde_urlencoded::to_string(&request).unwrap();
    let tracker_url = format!(
        "{}?{}&info_hash={}",
        &torrent_metadata.announce,
        query_params,
        url_encode(&hashed_info)
    );


    let response = client
        .get(&tracker_url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await;

    let body = response.unwrap().bytes().await.unwrap();

    let content: TrackerResponse = serde_bencode::from_bytes(&body).unwrap();


    println!("Found {} peers", content.peers.0.len());

    if let Some(first_peer) = content.peers.0.first() {
        let peer_id_bytes: [u8; 20] = request
            .peer_id
            .as_bytes()
            .try_into()
            .expect("peer_id must be exactly 20 bytes");

        match connect_to_peer(
            *first_peer,
            hashed_info,
            peer_id_bytes,
        )
        .await
        {
            Ok(stream) => {
                println!("Successfully connected and handshaked with {}", first_peer);
                // Keep the stream for next milestone (downloading pieces)
            }
            Err(e) => {
                println!("Failed to connect to {}: {}", first_peer, e);
            }
        }
    }
}

fn url_encode(data: &[u8]) -> String {
    data.iter().map(|byte| format!("%{:02X}", byte)).collect()
}

fn print_metadata(torrent_metadata: &Torrent) {
    // TODO: possible refactor to fmt:display
    println!("Announce: {}", torrent_metadata.announce);
    println!("Name: {}", torrent_metadata.info.name);
    println!(
        "Piece length in KiB: {}",
        torrent_metadata.info.piece_length / 1024
    );
    println!(
        "Number of pieces: {}",
        torrent_metadata.info.pieces.len() / 20
    );
    println!(
        "Length in MiB: {}",
        torrent_metadata.info.length / (1024 * 1024)
    );

    if let Some(comment) = &torrent_metadata.comment {
        println!("Comment: {}", comment);
    }
    if let Some(creation_date) = &torrent_metadata.creation_date {
        println!("Creation date (Unix timestamp): {}", creation_date);
    }
}

pub async fn connect_to_peer(
    peer: SocketAddrV4,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
) -> Result<TcpStream, Box<dyn std::error::Error>> {
    println!("Connecting to peer: {}", peer);

    // Establish TCP connection
    let mut stream = TcpStream::connect(peer).await?;

    println!("Connected! Sending handshake...");

    // Create handshake
    let handshake = Handshake::new(info_hash, peer_id);
    let handshake_bytes = handshake.to_bytes();

    // Send handshake
    stream.write_all(&handshake_bytes).await.unwrap();

    println!("Handshake sent, waiting for response...");

    // Read peer's handshake response (68 bytes)
    let mut response = vec![0u8; 68];
    stream.read_exact(&mut response).await.unwrap();

    // Parse peer's handshake
    let peer_handshake = Handshake::from_bytes(&response).unwrap();

    // Verify protocol
    if peer_handshake.protocol != *b"BitTorrent protocol" {
        return Err("Invalid protocol in peer handshake".into());
    }

    // Verify info_hash matches
    if peer_handshake.info_hash != info_hash {
        return Err("Info hash mismatch".into());
    }

    println!("Handshake successful!");
    println!(
        "Peer ID: {:?}",
        String::from_utf8_lossy(&peer_handshake.peer_id)
    );

    Ok(stream)
}

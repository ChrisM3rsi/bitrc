#[derive(Debug, Clone)]
pub struct Handshake {
    pub length: u8,
    pub protocol: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Handshake {
        Self {
            length: 19,
            protocol: *b"BitTorrent protocol",
            reserved: [0u8; 8],
            info_hash,
            peer_id,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(68);
        bytes.push(self.length);
        bytes.extend_from_slice(&self.protocol);
        bytes.extend_from_slice(&self.reserved);
        bytes.extend_from_slice(&self.info_hash);
        bytes.extend_from_slice(&self.peer_id);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 68 {
            return Err(format!("Invalid handshake length: {}", bytes.len()));
        }

        let mut handshake = Self {
            length: bytes[0],
            protocol: [0u8; 19],
            reserved: [0u8; 8],
            info_hash: [0u8; 20],
            peer_id: [0u8; 20],
        };

        handshake.protocol.copy_from_slice(&bytes[1..20]);
        handshake.reserved.copy_from_slice(&bytes[20..28]);
        handshake.info_hash.copy_from_slice(&bytes[28..48]);
        handshake.peer_id.copy_from_slice(&bytes[48..68]);

        Ok(handshake)
    }
}

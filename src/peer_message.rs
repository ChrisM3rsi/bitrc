#[derive(Debug, Clone)]
pub enum PeerMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have {
        piece_index: u32,
    },
    Bitfield {
        bitfield: Vec<u8>,
    },
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: Vec<u8>,
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
}

impl PeerMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Ok(PeerMessage::KeepAlive);
        }

        let id = bytes[0];
        let payload = &bytes[1..];

        match id {
            0 => Ok(PeerMessage::Choke),
            1 => Ok(PeerMessage::Unchoke),
            2 => Ok(PeerMessage::Interested),
            3 => Ok(PeerMessage::NotInterested),
            4 => {
                let piece_index = u32::from_be_bytes(payload[0..4].try_into().unwrap());
                Ok(PeerMessage::Have { piece_index })
            }
            5 => Ok(PeerMessage::Bitfield {
                bitfield: payload.to_vec(),
            }),
            6 => {
                let index = u32::from_be_bytes(payload[0..4].try_into().unwrap());
                let begin = u32::from_be_bytes(payload[4..8].try_into().unwrap());
                let length = u32::from_be_bytes(payload[8..12].try_into().unwrap());

                Ok(PeerMessage::Request {
                    index,
                    begin,
                    length,
                })
            }
            7 => {
                let index = u32::from_be_bytes(payload[0..4].try_into().unwrap());
                let begin = u32::from_be_bytes(payload[4..8].try_into().unwrap());
                let block = payload[8..].to_vec();
                Ok(PeerMessage::Piece {
                    index,
                    begin,
                    block,
                })
            }
            8 => {
                let index = u32::from_be_bytes(payload[0..4].try_into().unwrap());
                let begin = u32::from_be_bytes(payload[4..8].try_into().unwrap());
                let length = u32::from_be_bytes(payload[8..12].try_into().unwrap());
                Ok(PeerMessage::Cancel {
                    index,
                    begin,
                    length,
                })
            }
            _ => Err(format!("Unknown message ID: {}", id)),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PeerMessage::KeepAlive => vec![0, 0, 0, 0],
            PeerMessage::Choke => vec![0, 0, 0, 1, 0],
            PeerMessage::Unchoke => vec![0, 0, 0, 1, 1],
            PeerMessage::Interested => vec![0, 0, 0, 1, 2],
            PeerMessage::NotInterested => vec![0, 0, 0, 1, 3],
            PeerMessage::Request {
                index,
                begin,
                length,
            } => {
                let mut bytes = vec![0, 0, 0, 13, 6]; // length=13, id=6
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(&length.to_be_bytes());
                bytes
            }
            _ => vec![], // Implement others as needed
        }
    }

    // 0 - choke
    // 1 - unchoke
    // 2 - interested
    // 3 - not interested
    // 4 - have
    // 5 - bitfield
    // 6 - request
    // 7 - piece
    // 8 - cancel
}

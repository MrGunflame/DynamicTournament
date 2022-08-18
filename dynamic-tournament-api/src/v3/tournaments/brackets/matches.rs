use bincode::{DefaultOptions, Options};
use dynamic_tournament_core::{EntrantScore, Matches};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    compat: u8,
    command: RequestCommand,
}

impl Request {
    pub fn from_bytes(&self) {}

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.compat);
        buf.extend(self.command.to_bytes());

        buf
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestCommand {
    /// Reserved for protocol implementations.
    /// You can safely ignore this when matching the command.
    Reserved,
    Authorize(String),
    /// Synchronize the state of the bracket.
    SyncState,
    /// Update the match the given `index` using the data in the
    /// `nodes`.
    UpdateMatch {
        index: u64,
        nodes: [EntrantScore<u64>; 2],
    },
    /// Resets the match at the given `index`.
    ResetMatch {
        index: u64,
    },
}

impl RequestCommand {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            Self::Reserved => {
                buf.push(0);
            }
            Self::Authorize(token) => {
                buf.push(1);

                buf.extend(token.len().to_le_bytes());
                buf.extend(token.as_bytes());
            }
            Self::SyncState => {
                buf.push(2);
            }
            Self::UpdateMatch { index, nodes } => {
                buf.push(3);

                buf.extend(index.to_le_bytes());
                buf.push(2);
                buf.extend(nodes[0].score.to_le_bytes());
                buf.push(nodes[0].winner as u8);
                buf.extend(nodes[1].score.to_le_bytes());
                buf.push(nodes[1].winner as u8);
            }
            Self::ResetMatch { index } => {
                buf.push(4);

                buf.extend(index.to_le_bytes());
            }
        }

        buf
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frame {
    Reserved,
    Authorize(String),
    SyncMatchesRequest,
    SyncMatchesResponse(Matches<EntrantScore<u64>>),
    UpdateMatch {
        index: u64,
        nodes: [EntrantScore<u64>; 2],
    },
    ResetMatch {
        index: usize,
    },
}

impl Frame {
    pub fn to_bytes(&self) -> bincode::Result<Vec<u8>> {
        let options = DefaultOptions::new()
            .with_little_endian()
            .with_varint_encoding();

        options.serialize(self)
    }

    pub fn from_bytes(buf: &[u8]) -> bincode::Result<Self> {
        let options = DefaultOptions::new()
            .with_little_endian()
            .with_varint_encoding();

        options.deserialize(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::Frame;

    #[test]
    fn test_frame_to_bytes() {
        let frame = Frame::Reserved;

        assert_eq!(frame.to_bytes().unwrap(), vec![0]);

        let frame = Frame::Authorize(String::from("Hello World"));
        assert_eq!(
            frame.to_bytes().unwrap(),
            vec![
                1,
                "Hello World".as_bytes().len().try_into().unwrap(),
                b'H',
                b'e',
                b'l',
                b'l',
                b'o',
                b' ',
                b'W',
                b'o',
                b'r',
                b'l',
                b'd',
            ]
        );
    }

    #[test]
    fn test_frame_from_bytes() {
        let bytes = &[0];

        let frame = Frame::from_bytes(bytes).unwrap();
        assert_eq!(frame, Frame::Reserved);

        let bytes = &[
            1,
            "Hello World".as_bytes().len().try_into().unwrap(),
            b'H',
            b'e',
            b'l',
            b'l',
            b'o',
            b' ',
            b'W',
            b'o',
            b'r',
            b'l',
            b'd',
        ];

        let frame = Frame::from_bytes(bytes).unwrap();
        assert_eq!(frame, Frame::Authorize(String::from("Hello World")));
    }
}

use std::io::{self, Write};

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

const CONTINUE_BIT: u8 = 1 << 7;

pub trait Encode {
    fn encode<W>(&self, writer: W) -> Result<usize, io::Error>
    where
        W: Write;
}

impl Encode for u8 {
    fn encode<W>(&self, mut writer: W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        writer.write_all(&[*self])?;
        Ok(1)
    }
}

impl Encode for u16 {
    fn encode<W>(&self, mut writer: W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        let mut n = *self;

        let mut bytes_written = 0;
        loop {
            let byte = n & (u8::MAX as u16);
            let mut byte = byte as u8 & !CONTINUE_BIT;

            n >>= 7;
            if n != 0 {
                byte |= CONTINUE_BIT;
            }

            writer.write_all(&[byte])?;
            bytes_written += 1;

            if n == 0 {
                return Ok(bytes_written);
            }
        }
    }
}

impl Encode for u32 {
    fn encode<W>(&self, mut writer: W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        let mut n = *self;

        let mut bytes_written = 0;
        loop {
            let byte = n & (u8::MAX as u32);
            let mut byte = byte as u8 & !CONTINUE_BIT;

            n >>= 7;
            if n != 0 {
                byte |= CONTINUE_BIT;
            }

            writer.write_all(&[byte])?;
            bytes_written += 1;

            if n == 0 {
                return Ok(bytes_written);
            }
        }
    }
}

impl Encode for i8 {
    fn encode<W>(&self, writer: W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        let n = ((*self << 1) ^ (*self >> 7)) as u8;
        n.encode(writer)
    }
}

impl Encode for i16 {
    fn encode<W>(&self, writer: W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        let n = ((*self << 1) ^ (*self >> 15)) as u16;
        n.encode(writer)
    }
}

impl Encode for i32 {
    fn encode<W>(&self, writer: W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        let n = ((*self << 1) ^ (*self >> 31)) as u32;
        n.encode(writer)
    }
}

impl Encode for i64 {
    fn encode<W>(&self, writer: W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        let n = ((*self << 1) ^ (*self >> 63)) as u64;
        n.encode(writer)
    }
}

impl Encode for u64 {
    fn encode<W>(&self, mut writer: W) -> Result<usize, io::Error>
    where
        W: Write,
    {
        let mut n = *self;

        let mut bytes_written = 0;
        loop {
            let byte = n & (u8::MAX as u64);
            let mut byte = byte as u8 & !CONTINUE_BIT;

            n >>= 7;
            if n != 0 {
                byte |= CONTINUE_BIT;
            }

            writer.write_all(&[byte])?;
            bytes_written += 1;

            if n == 0 {
                return Ok(bytes_written);
            }
        }
    }
}

mod leb128 {
    const CONTINUE_BIT: u8 = 1 << 7;

    fn encode(mut n: u64, w: &mut Vec<u8>) -> usize {
        let mut len = 0;
        loop {
            let byte = n & (u8::MAX as u64);
            let mut byte = byte as u8 & !CONTINUE_BIT;

            n >>= 7;
            if n != 0 {
                byte |= CONTINUE_BIT;
            }

            let buf = [byte];
            w.extend(buf);
            len += 1;

            if n == 0 {
                return len;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Encode, Frame};

    #[test]
    fn test_encode_u8() {
        let mut buf = Vec::new();
        0_u8.encode(&mut buf).unwrap();
        assert_eq!(buf, [0]);
    }

    #[test]
    fn test_encode_u16() {
        let mut buf = Vec::new();
        127_u16.encode(&mut buf).unwrap();
        assert_eq!(buf, [127]);

        let mut buf = Vec::new();
        300_u16.encode(&mut buf).unwrap();
        assert_eq!(buf, [172, 2]);
    }

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

use std::io::{self, Read, Write};
use std::mem::MaybeUninit;
use std::string::FromUtf8Error;

use dynamic_tournament_core::{EntrantScore, EntrantSpot, Match, Matches, Node};
use serde::{Deserialize, Serialize};

/// An error which can occur while encoding or decoding a type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("invalid utf8: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),
    #[error("invalid bool value: {0}")]
    InvalidBool(u8),
    #[error("varint overflow")]
    VarintOverflow,
    #[error("invalid variant")]
    InvalidVariant,
    #[error("invalid sequence length")]
    InvalidLength,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
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

impl Request {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let _ = self.encode(&mut buf);
        buf
    }
}

impl Encode for Request {
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let n: u8 = match self {
            Self::Reserved => 0,
            Self::Authorize(_) => 1,
            Self::SyncState => 2,
            Self::UpdateMatch { index: _, nodes: _ } => 3,
            Self::ResetMatch { index: _ } => 4,
        };
        let mut bytes_written = n.encode(&mut writer)?;

        match self {
            Self::Reserved => (),
            Self::Authorize(token) => {
                bytes_written += token.encode(writer)?;
            }
            Self::SyncState => (),
            Self::UpdateMatch { index, nodes } => {
                bytes_written += index.encode(&mut writer)?;
                bytes_written += nodes.encode(writer)?;
            }
            Self::ResetMatch { index } => {
                bytes_written += index.encode(writer)?;
            }
        }

        Ok(bytes_written)
    }
}

impl Decode for Request {
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        match u8::decode(&mut reader)? {
            0 => Ok(Self::Reserved),
            1 => {
                let token = String::decode(reader)?;

                Ok(Self::Authorize(token))
            }
            2 => Ok(Self::SyncState),
            3 => {
                let index = u64::decode(&mut reader)?;
                let nodes = Decode::decode(reader)?;

                Ok(Self::UpdateMatch { index, nodes })
            }
            4 => {
                let index = u64::decode(&mut reader)?;

                Ok(Self::ResetMatch { index })
            }
            _ => Err(Error::InvalidVariant),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Response {
    Reserved,
    Error,
    SyncState(Matches<EntrantScore<u64>>),
    UpdateMatch {
        index: u64,
        nodes: [EntrantScore<u64>; 2],
    },
    ResetMatch {
        index: u64,
    },
}

#[derive(Clone, Debug)]
pub enum ErrorResp {
    Unauthorized,
    /// The server event queue lagged and is out sync. The client may want
    /// to synchronize again.
    Lagged,
}

impl Response {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let _ = self.encode(&mut buf);
        buf
    }
}

impl Encode for Response {
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let cmd: u8 = match self {
            Self::Reserved => 0,
            Self::Error => 1,
            Self::SyncState(_) => 2,
            Self::UpdateMatch { index: _, nodes: _ } => 3,
            Self::ResetMatch { index: _ } => 4,
        };
        let mut bytes_written = cmd.encode(&mut writer)?;

        match self {
            Self::Reserved => (),
            Self::Error => (),
            Self::SyncState(state) => {
                let slice: &[_] = state.as_ref();
                bytes_written += slice.encode(writer)?;
            }
            Self::UpdateMatch { index, nodes } => {
                bytes_written += index.encode(&mut writer)?;
                bytes_written += nodes.encode(writer)?;
            }
            Self::ResetMatch { index } => {
                bytes_written += index.encode(writer)?;
            }
        }

        Ok(bytes_written)
    }
}

impl Decode for Response {
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        match u8::decode(&mut reader)? {
            0 => Ok(Self::Reserved),
            1 => Ok(Self::Error),
            2 => {
                let matches: Vec<Match<Node<EntrantScore<u64>>>> = Decode::decode(reader)?;

                Ok(Self::SyncState(Matches::from(matches)))
            }
            3 => {
                let index = u64::decode(&mut reader)?;
                let nodes = Decode::decode(reader)?;

                Ok(Self::UpdateMatch { index, nodes })
            }
            4 => {
                let index = u64::decode(reader)?;

                Ok(Self::ResetMatch { index })
            }
            _ => Err(Error::InvalidVariant),
        }
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

const CONTINUE_BIT: u8 = 1 << 7;

/// A type that can be encoded into a byte buffer.
pub trait Encode {
    fn encode<W>(&self, writer: W) -> Result<usize, Error>
    where
        W: Write;
}

/// A type that can be decoded from a byte buffer.
pub trait Decode: Sized {
    fn decode<R>(reader: R) -> Result<Self, Error>
    where
        R: Read;
}

// ========================
// ===== Encode impls =====
// ========================

impl Encode for bool {
    fn encode<W>(&self, writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        (*self as u8).encode(writer)
    }
}

impl Encode for u8 {
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        writer.write_all(&[*self])?;
        Ok(1)
    }
}

impl Encode for u16 {
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
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
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
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
    fn encode<W>(&self, writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let n = ((*self << 1) ^ (*self >> 7)) as u8;
        n.encode(writer)
    }
}

impl Encode for i16 {
    fn encode<W>(&self, writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let n = ((*self << 1) ^ (*self >> 15)) as u16;
        n.encode(writer)
    }
}

impl Encode for i32 {
    fn encode<W>(&self, writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let n = ((*self << 1) ^ (*self >> 31)) as u32;
        n.encode(writer)
    }
}

impl Encode for i64 {
    fn encode<W>(&self, writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let n = ((*self << 1) ^ (*self >> 63)) as u64;
        n.encode(writer)
    }
}

impl Encode for u64 {
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
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

impl Encode for usize {
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let mut n = *self;

        let mut bytes_written = 0;
        loop {
            let byte = n & (u8::MAX as usize);
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

impl<T> Encode for [T]
where
    T: Encode,
{
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let len = self.len() as u64;
        let mut bytes_written = len.encode(&mut writer)?;

        for elem in self {
            bytes_written += elem.encode(&mut writer)?;
        }

        Ok(bytes_written)
    }
}

impl Encode for str {
    fn encode<W>(&self, writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        self.as_bytes().encode(writer)
    }
}

// =======================
// ===== Decode impl =====
// =======================

impl Decode for bool {
    fn decode<R>(reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        match u8::decode(reader)? {
            0 => Ok(false),
            1 => Ok(true),
            n => Err(Error::InvalidBool(n)),
        }
    }
}

impl Decode for u8 {
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl Decode for u64 {
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let mut n = 0;
        let mut shift = 0;

        loop {
            let mut buf = [0];
            reader.read_exact(&mut buf)?;

            if shift == u64::BITS - 1 {
                consume_trail(reader)?;
                panic!("ULEB-128 overflow");
            }

            // Remove the continue bit.
            n += ((buf[0] & !CONTINUE_BIT) as u64) << shift;

            // Continue bit not set. This is the end of the integer.
            if buf[0] & CONTINUE_BIT == 0 {
                return Ok(n);
            }

            shift += 7;
        }
    }
}

impl Decode for usize {
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let mut n = 0;
        let mut shift = 0;

        loop {
            let mut buf = [0];
            reader.read_exact(&mut buf)?;

            if shift > usize::BITS {
                consume_trail(reader)?;
                return Err(Error::VarintOverflow);
            }

            // Remove the continue bit.
            n += ((buf[0] & !CONTINUE_BIT) as usize) << shift;

            // Continue bit not set. This is the end of the integer.
            if buf[0] & CONTINUE_BIT == 0 {
                return Ok(n);
            }

            shift += 7;
        }
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode,
{
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let len = usize::decode(&mut reader)?;

        let mut buf = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(T::decode(&mut reader)?);
        }

        Ok(buf)
    }
}

impl<T, const N: usize> Decode for [T; N]
where
    T: Decode,
{
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let len = usize::decode(&mut reader)?;
        if len != N {
            return Err(Error::InvalidLength);
        }

        // SAFETY: An uninitialized `[MaybeUninit<_>; N]` is always valid.
        let mut buf: [MaybeUninit<T>; N] =
            unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };

        let mut elems = 0;

        // Read all elements from the reader. If reading a single value fails we
        // return an error. This means we need to drop all previously initialized
        // elements. The `elems` variable keeps track of how many values have been
        // initialized.
        for index in 0..len {
            match T::decode(&mut reader) {
                Ok(val) => {
                    buf[index].write(val);
                    elems += 1;
                }
                Err(err) => {
                    // Drop all previously initialized elements.
                    for mut elem in buf.into_iter().take(elems) {
                        // SAFETY: All fields until `elems` are initialized.
                        unsafe {
                            elem.assume_init_drop();
                        }
                    }

                    return Err(err);
                }
            }
        }

        // Transmute [MaybeUninit<T>; N] into [T; N].
        // SAFETY: All fields are initialized.
        let buf = unsafe { (&buf as *const _ as *const [T; N]).read() };

        Ok(buf)
    }
}

impl Decode for String {
    fn decode<R>(reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let buf = Vec::decode(reader)?;
        String::from_utf8(buf).map_err(|err| err.into())
    }
}

// =========================================
// ===== dynamic-tournament-core impls =====
// =========================================

impl<T> Encode for Match<T>
where
    T: Encode,
{
    fn encode<W>(&self, writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        self.entrants.encode(writer)
    }
}

impl<T> Encode for EntrantSpot<T>
where
    T: Encode,
{
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let n: u8 = match self {
            Self::Empty => 0,
            Self::TBD => 1,
            Self::Entrant(_) => 2,
        };
        let mut bytes_written = n.encode(&mut writer)?;

        if let Self::Entrant(entrant) = self {
            bytes_written += entrant.encode(writer)?;
        }

        Ok(bytes_written)
    }
}

impl<T> Encode for Node<T>
where
    T: Encode,
{
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let mut bytes_written = (self.index as u64).encode(&mut writer)?;
        bytes_written += self.data.encode(writer)?;

        Ok(bytes_written)
    }
}

impl<T> Encode for EntrantScore<T>
where
    T: Encode,
{
    fn encode<W>(&self, mut writer: W) -> Result<usize, Error>
    where
        W: Write,
    {
        let mut bytes_written = self.score.encode(&mut writer)?;
        bytes_written += self.winner.encode(writer)?;

        Ok(bytes_written)
    }
}

impl<T> Decode for Match<T>
where
    T: Decode,
{
    fn decode<R>(reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        Ok(Match::new(Decode::decode(reader)?))
    }
}

impl<T> Decode for EntrantSpot<T>
where
    T: Decode,
{
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        match u8::decode(&mut reader)? {
            0 => Ok(Self::Empty),
            1 => Ok(Self::TBD),
            2 => Ok(Self::Entrant(T::decode(reader)?)),
            _ => Err(Error::InvalidVariant),
        }
    }
}

impl<T> Decode for Node<T>
where
    T: Decode,
{
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let index = usize::decode(&mut reader)?;
        let data = T::decode(reader)?;

        Ok(Node::new_with_data(index, data))
    }
}

impl<T> Decode for EntrantScore<T>
where
    T: Decode,
{
    fn decode<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let score = T::decode(&mut reader)?;
        let winner = bool::decode(reader)?;

        Ok(Self { score, winner })
    }
}

/// Consumes the ULEB-128 encoded integer from the `reader` without storing them. This is mostly
/// useful if you want to handle an integer overflow.
///
/// # Errors
///
/// This function will read until the ULEB-128 encoded integer is terminated properly. If the
/// reader is empty without terminating the integer an [`io::Error`] is returned. This function
/// also returns an [`io::Error`] reading from the reader fails for any other reason.
fn consume_trail<R>(mut reader: R) -> Result<(), io::Error>
where
    R: Read,
{
    let mut buf = [0];
    loop {
        reader.read_exact(&mut buf)?;

        // Reached the end of the integer.
        if buf[0] & CONTINUE_BIT == 0 {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{Decode, Encode, EntrantScore, EntrantSpot, Error, Match, Node};

    #[test]
    fn test_encode_bool() {
        let mut buf = Vec::new();
        false.encode(&mut buf).unwrap();
        assert_eq!(buf, [0]);

        true.encode(&mut buf).unwrap();
        assert_eq!(buf, [0, 1]);
    }

    #[test]
    fn test_encode_u8() {
        for i in 0..u8::MAX {
            let mut buf = Vec::with_capacity(1);
            i.encode(&mut buf).unwrap();
            assert_eq!(buf, [i]);
        }
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
    fn test_encode_u64() {
        let mut buf = Vec::new();
        127_u64.encode(&mut buf).unwrap();
        assert_eq!(buf, [127]);

        let mut buf = Vec::new();
        624485_u64.encode(&mut buf).unwrap();
        assert_eq!(buf, [0xE5, 0x8E, 0x26]);
    }

    #[test]
    fn test_decode_bool() {
        let mut buf = Cursor::new([0, 1, 2]);
        assert!(!bool::decode(&mut buf).unwrap());
        assert!(bool::decode(&mut buf).unwrap());

        matches!(bool::decode(&mut buf).unwrap_err(), Error::InvalidBool(_));
    }

    #[test]
    fn test_decode_u64() {
        let mut buf = Cursor::new([0]);
        assert_eq!(u64::decode(&mut buf).unwrap(), 0);

        let mut buf = Cursor::new([127]);
        assert_eq!(u64::decode(&mut buf).unwrap(), 127);

        let mut buf = Cursor::new([172, 2]);
        assert_eq!(u64::decode(&mut buf).unwrap(), 300);
    }

    #[test]
    fn test_decode_usize() {
        let buf = Cursor::new([0]);
        assert_eq!(usize::decode(buf).unwrap(), 0);

        let buf = Cursor::new([127]);
        assert_eq!(usize::decode(buf).unwrap(), 127);

        let buf = Cursor::new([172, 2]);
        assert_eq!(usize::decode(buf).unwrap(), 300);
    }

    #[test]
    fn test_decode_array() {
        let buf = Cursor::new([5, 1, 2, 3, 4, 5]);
        assert_eq!(<[u8; 5]>::decode(buf).unwrap(), [1, 2, 3, 4, 5]);

        // Invalid length
        let buf = Cursor::new([3, 1, 2, 3]);
        matches!(<[u8; 5]>::decode(buf).unwrap_err(), Error::InvalidLength);

        // Test internal drop implementation.
        static ACTIVE: AtomicUsize = AtomicUsize::new(3);

        #[derive(Debug)]
        struct HasDrop(u8);

        impl Decode for HasDrop {
            fn decode<R>(reader: R) -> Result<Self, Error>
            where
                R: Read,
            {
                Ok(Self(u8::decode(reader)?))
            }
        }

        impl Drop for HasDrop {
            fn drop(&mut self) {
                ACTIVE.fetch_sub(1, Ordering::SeqCst);
            }
        }

        let buf = Cursor::new([5, 1, 2, 3]);
        matches!(<[HasDrop; 5]>::decode(buf).unwrap_err(), Error::Io(_));
        assert_eq!(ACTIVE.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_encode_entrant_score() {
        let mut buf = Vec::new();
        EntrantScore {
            score: 23_u64,
            winner: false,
        }
        .encode(&mut buf)
        .unwrap();
        assert_eq!(buf, [23, 0]);

        EntrantScore {
            score: 69_u64,
            winner: true,
        }
        .encode(&mut buf)
        .unwrap();
        assert_eq!(buf, [23, 0, 69, 1]);
    }

    #[test]
    fn test_encode_node() {
        let mut buf = Vec::new();
        Node::new_with_data(127, 126_u64).encode(&mut buf).unwrap();
        assert_eq!(buf, [127, 126]);
    }

    #[test]
    fn test_encode_entrant_spot() {
        let mut buf = Vec::new();
        EntrantSpot::<u64>::Empty.encode(&mut buf).unwrap();
        assert_eq!(buf, [0]);

        EntrantSpot::<u64>::TBD.encode(&mut buf).unwrap();
        assert_eq!(buf, [0, 1]);

        EntrantSpot::Entrant(42_u64).encode(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2, 42]);
    }

    #[test]
    fn test_encode_match() {
        let mut buf = Vec::new();
        Match::new([EntrantSpot::<u64>::Empty, EntrantSpot::<u64>::Empty])
            .encode(&mut buf)
            .unwrap();
        assert_eq!(buf, [2, 0, 0]);
    }
}

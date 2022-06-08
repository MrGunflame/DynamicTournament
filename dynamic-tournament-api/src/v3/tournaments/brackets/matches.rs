use bincode::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frame {
    Reserved,
    Authorize(String),
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

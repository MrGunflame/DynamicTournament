use std::{
    collections::HashSet,
    io::{self, BufRead, BufReader, Read},
    ops::Index,
    str::from_utf8,
};

use crate::instructions::{Instruction, Operand, Pointer, Register};

use thiserror::Error;

#[derive(Debug)]
pub struct Parser<R>
where
    R: Read,
{
    reader: BufReader<R>,
    state: Vec<Token>,
    line: usize,
    labels: HashSet<Label>,
}

impl<R> Parser<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            state: Vec::new(),
            line: 0,
            labels: HashSet::new(),
        }
    }

    pub fn parse_line(&mut self) -> Result<(), Error> {
        let mut buf = Vec::new();
        self.reader.read_until(b'\n', &mut buf)?;

        // Remove all whitespaces.
        // buf.retain(|b| !b.is_ascii_whitespace());

        if buf.is_empty() {
            return Ok(());
        }

        if buf[0] == b'.' && buf[buf.len() - 1] == b':' {
            let label = parse_label(&buf)?;
            if self.labels.contains(&label) {
                return Err(Error::DuplicateLabel {
                    label,
                    line: self.line,
                });
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
struct Operands {
    buf: Vec<Operand>,
    found: usize,
    expected: usize,
}

impl Operands {
    /// Creates a new `Operands` buffer with no taken items.
    #[inline]
    fn new(buf: Vec<Operand>) -> Self {
        Self {
            buf,
            expected: 0,
            found: 0,
        }
    }

    /// Takes an [`Operand`] from the buffer. If it exists it is returned, otherwise an default
    /// [`Operand`] with the value `Operand::Const(0)` os returned.
    ///
    /// If the number of items taken mismatches the number of items in the buffer when calling
    /// [`Self::end`] an apropriate [`Error`] is returned.
    #[inline]
    fn take(&mut self) -> Operand {
        self.expected += 1;

        match self.buf.get(0) {
            Some(_) => {
                self.found += 1;
                self.buf.remove(0)
            }
            None => Operand::Const(0),
        }
    }

    /// Consumes the buffer, returning an apropriate [`Error`] if the number of items present at
    /// the start mismatches the number of items taken.
    #[inline]
    fn end(self) -> Result<(), Error> {
        if self.found != self.expected {
            Err(Error::InvalidOperands {
                found: self.found,
                expected: self.expected,
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum Token {
    Label(Label),
    Instruction(Instruction),
    Empty,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Label(Vec<u8>);

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("duplicate label {label:?} on line {line}")]
    DuplicateLabel { label: Label, line: usize },
    #[error("invalid instruction {ident:?}")]
    InvalidInstruction { ident: Vec<u8> },
    #[error("inalid token {token}")]
    InvalidToken { token: String },
    #[error("unexpected end of line")]
    UnexpectedEndOfLine { expected: &'static str },
    #[error("invalid register {0:?}")]
    InvalidRegister(Vec<u8>),
    #[error("invalid integer")]
    InvalidInteger,
    #[error("invalid number of provided operands: found {found} expected {expected}")]
    InvalidOperands { found: usize, expected: usize },
}

/// Parse a token from a single line.
fn parse_token(input: &[u8]) -> Result<Token, Error> {
    match input.get(0) {
        Some(b'.') => Ok(Token::Label(parse_label(input)?)),
        Some(_) => Ok(Token::Instruction(parse_instruction(input)?)),
        None => Ok(Token::Empty),
    }
}

fn parse_label(input: &[u8]) -> Result<Label, Error> {
    debug_assert_eq!(input[0], b'.');

    // SAFETY: Since `debug_assert_eq!` was successful, at least one item is in the slice.
    unsafe {
        if *input.last().unwrap_unchecked() != b':' {
            return Err(Error::UnexpectedEndOfLine { expected: ":" });
        }
    }

    let label = &input[1..input.len() - 1];

    Ok(Label(label.to_vec()))
}

fn parse_instruction(input: &[u8]) -> Result<Instruction, Error> {
    let mut parts = input.split(|b| b.is_ascii_whitespace());

    let instruction = parts.next().unwrap();

    let mut ops = match parts.next() {
        Some(part) => {
            let operands = part.split(|b| *b == b',');

            let mut buf = Vec::new();
            for op in operands {
                buf.push(parse_operand(op)?);
            }
            Operands::new(buf)
        }
        None => Operands::default(),
    };

    match instruction {
        b"ADD" => Instruction::ADD(ops.take()),
        b"SUB" => Instruction::SUB(ops.take()),
        b"MUL" => Instruction::MUL(ops.take()),
        b"DIV" => Instruction::DIV(ops.take()),
        b"SHL" => Instruction::SHL(ops.take()),
        b"SHR" => Instruction::SHR(ops.take()),

        b"AND" => Instruction::AND(ops.take()),
        b"OR" => Instruction::OR(ops.take()),
        b"XOR" => Instruction::XOR(ops.take()),
        b"NOT" => Instruction::NOT,

        b"MOV" => Instruction::MOV(ops.take(), ops.take()),
        b"PUSH" => Instruction::PUSH,
        b"POP" => Instruction::POP,

        b"JMP" => Instruction::JMP(ops.take()),
        b"JE" => Instruction::JE(ops.take(), ops.take(), ops.take()),
        b"JNE" => Instruction::JNE(ops.take(), ops.take(), ops.take()),
        b"JG" => Instruction::JG(ops.take(), ops.take(), ops.take()),
        b"JGE" => Instruction::JGE(ops.take(), ops.take(), ops.take()),
        b"JL" => Instruction::JL(ops.take(), ops.take(), ops.take()),
        b"JLE" => Instruction::JLE(ops.take(), ops.take(), ops.take()),

        b"ABORT" => Instruction::ABORT,

        ident => {
            return Err(Error::InvalidInstruction {
                ident: ident.to_vec(),
            });
        }
    };

    ops.end()?;

    Ok(Instruction::ABORT)
}

#[inline]
fn parse_operand(input: &[u8]) -> Result<Operand, Error> {
    match input[0] {
        // Register
        b'%' => match &input[1..] {
            b"rax" => Ok(Operand::Register(Register::RAX)),
            ident => Err(Error::InvalidRegister(ident.to_vec())),
        },
        // Address
        b'*' => {
            let int = parse_integer(&input[1..])?;
            Ok(Operand::Pointer(Pointer(int)))
        }
        _ => {
            let int = parse_integer(input)?;
            Ok(Operand::Const(int))
        }
    }
}

/// Parses a `u64` from an input. The input must not be empty and contain a valid
/// UTF-8 sequence.
///
/// `parse_integer` parses decimal integers by default, but also parses binary or hexadecimal
/// values if they are prefixed. If an unknown prefix is found, an error is returned.
#[inline]
fn parse_integer(input: &[u8]) -> Result<u64, Error> {
    let string = match from_utf8(input) {
        Ok(val) => val,
        Err(_) => return Err(Error::InvalidInteger),
    };

    let (src, radix) = match input.get(0) {
        Some(b) => match b.to_ascii_lowercase() {
            b'b' => (&string[1..], 2),
            b'x' => (&string[1..], 16),
            b if b.is_ascii_digit() => (string, 10),
            _ => return Err(Error::InvalidInteger),
        },
        None => return Err(Error::InvalidInteger),
    };

    match u64::from_str_radix(src, radix) {
        Ok(val) => Ok(val),
        Err(_) => Err(Error::InvalidInteger),
    }
}

#[cfg(test)]
mod tests {
    use crate::instructions::{Operand, Pointer, Register};

    use super::{parse_integer, parse_label, parse_operand, Error, Label};

    #[test]
    fn test_parse_label() {
        let input = b".hello_world:";
        assert_eq!(parse_label(input).unwrap(), Label(b"hello_world".to_vec()));

        let input = b".hello_world";
        let err = parse_label(input).unwrap_err();
        matches!(err, Error::UnexpectedEndOfLine { expected: ":" });
    }

    #[test]
    #[should_panic]
    fn test_parse_label_empty() {
        let input = b"";
        let _ = parse_label(input);
    }

    #[test]
    #[should_panic]
    fn test_parse_label_invalid() {
        let input = b"hello_world:";
        let _ = parse_label(input);
    }

    #[test]
    fn test_parse_instruction() {}

    #[test]
    fn test_parse_operand_const() {
        // Decimal
        let input = b"23";
        assert_eq!(parse_operand(input).unwrap(), Operand::Const(23));

        // Binary
        let input = b"b11";
        assert_eq!(parse_operand(input).unwrap(), Operand::Const(0b11));

        // Hex
        let input = b"xA1";
        assert_eq!(parse_operand(input).unwrap(), Operand::Const(0xA1));
    }

    #[test]
    fn test_parse_operand_pointer() {
        let input = b"*32";
        assert_eq!(parse_operand(input).unwrap(), Operand::Pointer(Pointer(32)));
    }

    #[test]
    fn test_parse_operand_register() {
        let input = b"%rax";
        assert_eq!(
            parse_operand(input).unwrap(),
            Operand::Register(Register::RAX)
        );

        let input = b"%abc";
        let err = parse_operand(input).unwrap_err();
        match err {
            Error::InvalidRegister(ident) => assert_eq!(ident, b"abc"),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_integer() {
        let input = b"69";
        assert_eq!(parse_integer(input).unwrap(), 69);

        let input = b"a123";
        let err = parse_integer(input).unwrap_err();
        matches!(err, Error::InvalidInteger);
    }

    #[test]
    fn test_parse_integer_bin() {
        let input = b"b1011";
        assert_eq!(parse_integer(input).unwrap(), 0b1011);

        let input = b"B10011";
        assert_eq!(parse_integer(input).unwrap(), 0b10011);
    }

    #[test]
    fn test_parse_integer_hex() {
        let input = b"xAF3";
        assert_eq!(parse_integer(input).unwrap(), 0xAF3);

        let input = b"xF222";
        assert_eq!(parse_integer(input).unwrap(), 0xF222);
    }
}

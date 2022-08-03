use std::fmt::Display;
use std::io::{self, Write};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Prompt<'a, T>
where
    T: FromStr,
    T::Err: Display,
{
    value: Option<T>,
    msg: &'a str,
}

impl<'a, T> Prompt<'a, T>
where
    T: FromStr,
    T::Err: Display,
{
    #[inline]
    pub fn new(msg: &'a str) -> Self {
        Self { value: None, msg }
    }

    pub fn read(&mut self) -> Result<T, T::Err> {
        {
            let mut writer = io::stdout();
            writer.write_all(self.msg.as_bytes()).unwrap();
            writer.write_all(b": ").unwrap();
            writer.flush().unwrap();
        }

        let mut string = String::new();
        io::stdin()
            .read_line(&mut string)
            .expect("Failed to read from stdin");
        string.truncate(string.len() - 1);

        T::from_str(&string)
    }

    /// Read until a valid element is input.
    pub fn read_valid(&mut self) -> T {
        loop {
            match self.read() {
                Ok(val) => return val,
                Err(err) => {
                    println!("Failed to parse input: {}", err)
                }
            }
        }
    }
}

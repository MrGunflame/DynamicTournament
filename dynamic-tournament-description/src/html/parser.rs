use std::io::Read;

use super::{Token, Tokens, AST};

pub trait Parse {
    fn parse();
}

pub struct Cursor {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidToken { token: char, position: usize },
}

pub fn tokenize(input: &str) -> Result<Tokens, Error> {
    let mut tokens = Tokens::new();

    let mut token = Token::Text(String::new());
    for (index, c) in input.chars().enumerate() {
        match c {
            // Start of open token.
            '<' => {
                tokens.push(token);
                token = Token::new_open_token();
            }
            '/' => match &mut token {
                // '/' followed immediately after opening '<' tag. This means this token
                // is actually a closing token.
                Token::OpenToken(string) => {
                    if string.is_empty() {
                        token = Token::new_close_token();
                    } else {
                        return Err(Error::InvalidToken {
                            token: c,
                            position: index,
                        });
                    }
                }
                Token::CloseToken(string) => {
                    return Err(Error::InvalidToken {
                        token: c,
                        position: index,
                    });
                }
                Token::Text(string) => string.push(c),
            },
            '>' => {
                tokens.push(token);
                token = Token::new_text();
            }

            _ => match &mut token {
                Token::Text(string) => string.push(c),
                Token::OpenToken(string) => string.push(c),
                Token::CloseToken(string) => string.push(c),
            },
        }
    }

    // Push trailing token.
    tokens.push(token);

    Ok(tokens)
}

pub fn parse(tokens: Tokens) -> Result<AST, Error> {
    let mut tree = AST::new();

    for token in tokens.tokens {
        match token {
            Token::OpenToken(name) => {}
            _ => unimplemented!(),
        }
    }

    Ok(tree)
}

#[cfg(test)]
mod tests {
    use crate::html::Token;

    use super::{tokenize, Error};

    #[test]
    fn test_tokenize() {
        let input = "<h1></h1>";
        assert_eq!(
            tokenize(input).unwrap(),
            [
                Token::OpenToken(String::from("h1")),
                Token::CloseToken(String::from("h1")),
            ]
        );

        let input = "<h1>Hello World</h1>";
        assert_eq!(
            tokenize(input).unwrap(),
            [
                Token::OpenToken(String::from("h1")),
                Token::Text(String::from("Hello World")),
                Token::CloseToken(String::from("h1")),
            ]
        );

        let input = "<h1><strong>test</strong></h1>";
        assert_eq!(
            tokenize(input).unwrap(),
            [
                Token::OpenToken(String::from("h1")),
                Token::OpenToken(String::from("strong")),
                Token::Text(String::from("test")),
                Token::CloseToken(String::from("strong")),
                Token::CloseToken(String::from("h1")),
            ]
        );

        let input = "<strong>123<em>456</strong>789</em>";
        assert_eq!(
            tokenize(input).unwrap(),
            [
                Token::OpenToken(String::from("strong")),
                Token::Text(String::from("123")),
                Token::OpenToken(String::from("em")),
                Token::Text(String::from("456")),
                Token::CloseToken(String::from("strong")),
                Token::Text(String::from("789")),
                Token::CloseToken(String::from("em")),
            ]
        );

        let input = "just text";
        assert_eq!(
            tokenize(input).unwrap(),
            [Token::Text(String::from("just text")),]
        );
    }

    #[test]
    fn test_tokenize_error() {
        let input = "<h1><//h1>";
        assert_eq!(
            tokenize(input).unwrap_err(),
            Error::InvalidToken {
                token: '/',
                position: 6,
            },
        );

        let input = "<h1/>";
        assert_eq!(
            tokenize(input).unwrap_err(),
            Error::InvalidToken {
                token: '/',
                position: 3,
            },
        );
    }
}

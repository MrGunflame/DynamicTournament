mod lexer;
mod parser;
mod render;

use lexer::Error;
use render::Renderer;

pub type Result<T> = std::result::Result<T, Error>;

pub fn from_str(s: &str) -> Result<String> {
    let root = parser::parse(s)?;
    Ok(Renderer::new(root).render())
}

/// Validates whether the input is valid without output any rendered result.
///
/// # Errors
///
/// Returns an [`Error`] if the given input is not valid.
pub fn validate(s: &str) -> Result<()> {
    parser::parse(s)?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    OpenToken(String),
    CloseToken(String),
    Text(String),
}

impl Token {
    pub fn new_open_token() -> Self {
        Self::OpenToken(String::new())
    }

    pub fn new_close_token() -> Self {
        Self::CloseToken(String::new())
    }

    pub fn new_text() -> Self {
        Self::Text(String::new())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tokens {
    tokens: Vec<Token>,
}

impl Tokens {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn push(&mut self, token: Token) {
        // Don't push empty strings onto the stack.
        if let Token::Text(text) = &token {
            if text.is_empty() {
                return;
            }
        }

        self.tokens.push(token);
    }

    pub fn find_closing(&self, ident: &str) -> Option<usize> {
        for (index, token) in self.tokens.iter().enumerate() {
            if let Token::CloseToken(name) = token {
                if name == ident {
                    return Some(index);
                }
            }
        }

        None
    }

    pub fn spans(&self) -> Spans<'_> {
        Spans {
            tokens: self.tokens.as_ref(),
            position: 0,
        }
    }

    pub fn as_ref(&self) -> TokensRef<'_> {
        TokensRef {
            tokens: &self.tokens,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TokensRef<'a> {
    tokens: &'a [Token],
}

impl<'a> TokensRef<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn spans(&self) -> Spans<'_> {
        Spans {
            tokens: self.tokens,
            position: 0,
        }
    }

    fn from_slice(slice: &'a [Token]) -> Self {
        Self { tokens: slice }
    }
}

impl<'a> AsRef<[Token]> for TokensRef<'a> {
    fn as_ref(&self) -> &[Token] {
        &self.tokens
    }
}

impl<T> PartialEq<T> for Tokens
where
    T: AsRef<[Token]>,
{
    fn eq(&self, other: &T) -> bool {
        self.tokens == other.as_ref()
    }
}

/// An iterator over a token branch.
#[derive(Debug)]
pub struct Spans<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Iterator for Spans<'a> {
    type Item = Span<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.tokens.len() {
            return None;
        }

        let start = self.position;

        let token = &self.tokens[start];
        match token {
            Token::OpenToken(open) => {
                // Find the next matching closing tag.
                for (index, token) in self.tokens[start..].iter().enumerate() {
                    if let Token::CloseToken(close) = token {
                        if open == close {
                            let end = start + index + 1;
                            self.position = end;

                            return Some(Span {
                                tokens: TokensRef::from_slice(&self.tokens[start..end]),
                            });
                        }
                    }
                }

                // There is no matching closing element. Return the whole buffer.
                Some(Span {
                    tokens: TokensRef::from_slice(&self.tokens[start..self.tokens.len() - start]),
                })
            }
            Token::CloseToken(_) => unreachable!(),
            Token::Text(_) => {
                self.position += 1;

                Some(Span {
                    tokens: TokensRef::from_slice(&self.tokens[start..start + 1]),
                })
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Span<'a> {
    tokens: TokensRef<'a>,
}

impl<'a> Span<'a> {
    pub fn as_tokens_ref(&self) -> TokensRef<'_> {
        self.tokens
    }

    pub fn head(&self) -> &Token {
        &self.tokens.as_ref()[0]
    }

    pub fn inner(&self) -> TokensRef<'_> {
        #[cfg(debug_assertions)]
        assert!(!self.tokens.is_empty());

        match self.head() {
            Token::Text(_) => TokensRef {
                tokens: &self.tokens.tokens,
            },
            _ => TokensRef {
                tokens: &self.tokens.tokens[1..self.tokens.len() - 1],
            },
        }
    }

    /// Returns an iterator over the `Span`s in the content of this `Span`.
    ///
    /// Note: This method differs from [`TokensRef::spans`], which returns an iterator
    /// over all tokens including the head/tail. This method is equivalent to
    /// `self.inner().spans()` and **not** `self.as_ref().spans()`.
    #[inline]
    pub fn spans(&self) -> Spans<'_> {
        Spans {
            tokens: &self.tokens.tokens[1..self.tokens.len() - 1],
            position: 0,
        }
    }
}

impl<'a> AsRef<[Token]> for Span<'a> {
    fn as_ref(&self) -> &[Token] {
        self.tokens.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::from_str;

    #[test]
    fn test_from_str() {
        let input = "<h1>Hello World!</h1>";
        assert_eq!(from_str(input).unwrap(), "<h1>Hello World!</h1>");

        let input =
            "<h1>Hello World!</h1><p>A<strong>B</strong>C<em>D<strong>E</strong>F</em>G</p>";
        assert_eq!(
            from_str(input).unwrap(),
            "<h1>Hello World!</h1><p>A<strong>B</strong>C<em>D<strong>E</strong>F</em>G</p>"
        );

        let input = "<h1>Test</h1> SomeText <em>Inline</em>";
        assert_eq!(
            from_str(input).unwrap(),
            "<h1>Test</h1> SomeText <em>Inline</em>"
        );
    }
}

mod parser;

use parser::Error;

use self::parser::tokenize;

pub fn parse(input: &str) -> Result<DocumentRoot> {
    let tokens = tokenize(input)?;
    DocumentRoot::parse(tokens.as_ref())
}

type Result<T> = std::result::Result<T, Error>;

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

pub struct DocumentRoot(Vec<Block>);

impl DocumentRoot {
    pub fn parse(tokens: TokensRef<'_>) -> Result<Self> {
        let mut blocks = Vec::new();

        for span in tokens.spans() {
            blocks.push(Block::parse(span)?);
        }

        Ok(Self(blocks))
    }
}

#[derive(Clone, Debug)]
pub enum Block {
    H1(Inline),
    H2(Inline),
    H3(Inline),
    H4(Inline),
    H5(Inline),
    H6(Inline),
    P(Inline),
    Inline(Inline),
    Text(String),
}

impl Block {
    pub fn parse(span: Span<'_>) -> Result<Self> {
        match span.head() {
            Token::OpenToken(tag) => match tag.as_str() {
                "h1" => {
                    let inline = Inline::parse(span.inner())?;

                    Ok(Self::H1(inline))
                }
                tag => Err(Error::InvalidTag {
                    tag: tag.to_owned(),
                    position: 0,
                }),
            },
            Token::CloseToken(_) => unreachable!(),
            Token::Text(text) => Ok(Self::Text(text.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inline(Vec<InlineBlock>);

impl Inline {
    pub fn parse(tokens: TokensRef<'_>) -> Result<Self> {
        let mut elements = Vec::new();

        for span in tokens.spans() {
            elements.push(InlineBlock::parse(span)?);
        }

        Ok(Self(elements))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlineBlock {
    Strong(Inline),
    Emphasis(Inline),
    Text(String),
}

impl InlineBlock {
    pub fn parse(span: Span<'_>) -> Result<Self> {
        match span.head() {
            Token::OpenToken(tag) => match tag.as_str() {
                "strong" => {
                    let inline = Inline::parse(span.inner())?;

                    Ok(Self::Strong(inline))
                }
                "em" => {
                    let inline = Inline::parse(span.inner())?;

                    Ok(Self::Emphasis(inline))
                }
                tag => Err(Error::InvalidTag {
                    tag: tag.to_string(),
                    position: 0,
                }),
            },
            Token::CloseToken(_) => unreachable!(),
            Token::Text(text) => Ok(Self::Text(text.to_owned())),
        }
    }
}

impl Inline {}

#[cfg(test)]
mod tests {
    use crate::html::InlineBlock;

    use super::{Inline, Token, Tokens};

    #[test]
    fn test_spans() {
        let mut tokens = Tokens::new();
        tokens.push(Token::OpenToken(String::from("h1")));
        tokens.push(Token::Text(String::from("Hello World")));
        tokens.push(Token::CloseToken(String::from("h1")));

        let mut spans = tokens.spans();

        let span = spans.next().unwrap();
        assert_eq!(*span.head(), Token::OpenToken(String::from("h1")));
        assert_eq!(
            span.inner().as_ref(),
            [Token::Text(String::from("Hello World"))]
        );

        let mut tokens = Tokens::new();
        tokens.push(Token::OpenToken(String::from("h1")));
        tokens.push(Token::OpenToken(String::from("strong")));
        tokens.push(Token::Text(String::from("Strong text")));
        tokens.push(Token::CloseToken(String::from("strong")));
        tokens.push(Token::Text(String::from("Normal text")));
        tokens.push(Token::CloseToken(String::from("h1")));

        let mut spans = tokens.spans();

        let span = spans.next().unwrap();
        assert_eq!(*span.head(), Token::OpenToken(String::from("h1")));
        assert_eq!(
            span.inner().as_ref(),
            [
                Token::OpenToken(String::from("strong")),
                Token::Text(String::from("Strong text")),
                Token::CloseToken(String::from("strong")),
                Token::Text(String::from("Normal text")),
            ]
        );

        {
            let mut spans = span.spans();

            let span = spans.next().unwrap();
            assert_eq!(*span.head(), Token::OpenToken(String::from("strong")));
            assert_eq!(
                span.inner().as_ref(),
                [Token::Text(String::from("Strong text"))]
            );

            let span = spans.next().unwrap();
            assert_eq!(*span.head(), Token::Text(String::from("Normal text")));
            assert_eq!(
                span.inner().as_ref(),
                [Token::Text(String::from("Normal text"))]
            );

            assert!(spans.next().is_none());
        }

        assert!(spans.next().is_none());
    }

    #[test]
    fn test_spans_iter() {
        let mut tokens = Tokens::new();
        tokens.push(Token::OpenToken(String::from("h1")));
        tokens.push(Token::Text(String::from("Hello World")));
        tokens.push(Token::CloseToken(String::from("h1")));

        let mut spans = tokens.spans();

        let span = spans.next().unwrap();
        assert_eq!(
            span.as_ref(),
            [
                Token::OpenToken(String::from("h1")),
                Token::Text(String::from("Hello World")),
                Token::CloseToken(String::from("h1")),
            ]
        );

        assert!(spans.next().is_none());

        let mut tokens = Tokens::new();
        tokens.push(Token::OpenToken(String::from("h1")));
        tokens.push(Token::OpenToken(String::from("strong")));
        tokens.push(Token::Text(String::from("Strong text")));
        tokens.push(Token::CloseToken(String::from("strong")));
        tokens.push(Token::Text(String::from("Normal text")));
        tokens.push(Token::CloseToken(String::from("h1")));

        let mut spans = tokens.spans();

        let span = spans.next().unwrap();
        assert_eq!(
            span.as_ref(),
            [
                Token::OpenToken(String::from("h1")),
                Token::OpenToken(String::from("strong")),
                Token::Text(String::from("Strong text")),
                Token::CloseToken(String::from("strong")),
                Token::Text(String::from("Normal text")),
                Token::CloseToken(String::from("h1")),
            ]
        );

        {
            let mut spans = span.spans();

            let span = spans.next().unwrap();
            assert_eq!(
                span.as_ref(),
                [
                    Token::OpenToken(String::from("strong")),
                    Token::Text(String::from("Strong text")),
                    Token::CloseToken(String::from("strong")),
                ]
            );

            let span = spans.next().unwrap();
            assert_eq!(span.as_ref(), [Token::Text(String::from("Normal text"))]);

            assert!(spans.next().is_none());
        }

        assert!(spans.next().is_none());
    }

    #[test]
    fn test_inline_parse() {
        let mut tokens = Tokens::new();
        tokens.push(Token::OpenToken(String::from("strong")));
        tokens.push(Token::Text(String::from("a")));
        tokens.push(Token::CloseToken(String::from("strong")));

        let inline = Inline::parse(tokens.as_ref()).unwrap();
        assert_eq!(
            inline.0,
            [InlineBlock::Strong(Inline(vec![InlineBlock::Text(
                String::from("a")
            )]))]
        );
    }
}

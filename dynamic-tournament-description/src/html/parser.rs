use super::lexer::tokenize;
use super::{Error, Result, Span, Token, TokensRef};

pub fn parse(input: &str) -> Result<DocumentRoot> {
    let tokens = tokenize(input)?;
    DocumentRoot::parse(tokens.as_ref())
}

#[derive(Clone, Debug)]
pub struct DocumentRoot(pub Vec<Block>);

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
                "h2" => {
                    let inline = Inline::parse(span.inner())?;
                    Ok(Self::H2(inline))
                }
                "h3" => {
                    let inline = Inline::parse(span.inner())?;
                    Ok(Self::H3(inline))
                }
                "h4" => {
                    let inline = Inline::parse(span.inner())?;
                    Ok(Self::H4(inline))
                }
                "h5" => {
                    let inline = Inline::parse(span.inner())?;
                    Ok(Self::H5(inline))
                }
                "h6" => {
                    let inline = Inline::parse(span.inner())?;
                    Ok(Self::H6(inline))
                }
                "p" => {
                    let inline = Inline::parse(span.inner())?;
                    Ok(Self::P(inline))
                }
                // Give this tag to inline for parsing.
                _ => Ok(Self::Inline(Inline::parse(span.as_tokens_ref())?)),
            },
            Token::CloseToken(_) => unreachable!(),
            Token::Text(text) => Ok(Self::Text(text.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inline(pub Vec<InlineBlock>);

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
    use super::{Inline, InlineBlock, Token};
    use crate::html::Tokens;

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

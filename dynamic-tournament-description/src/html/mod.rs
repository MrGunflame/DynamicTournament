mod parser;

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
}

#[derive(Debug)]
pub struct TokensRef<'a> {
    tokens: &'a [Token],
}

impl<'a> TokensRef<'a> {
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
                            let end = start + index;
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
            Token::Text(_) => Some(Span {
                tokens: TokensRef::from_slice(&self.tokens[start..start + 1]),
            }),
        }
    }
}

pub struct Span<'a> {
    tokens: TokensRef<'a>,
}

impl<'a> AsRef<[Token]> for Span<'a> {
    fn as_ref(&self) -> &[Token] {
        self.tokens.as_ref()
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
}

pub enum Leaf {
    Text(Vec<Self>),
    Strong(Vec<Leaf>),
    Emphasis(Vec<Leaf>),
}

pub struct Node {}

#[derive(Clone, Debug)]
pub struct AST {
    root: Vec<Block>,
}

impl AST {
    pub fn new() -> Self {
        Self { root: Vec::new() }
    }

    pub fn push(&mut self, block: Block) {
        self.root.push(block);
    }
}

#[derive(Clone, Debug)]
pub enum Inline {
    Text(Vec<Self>),
    Strong(Vec<Self>),
    Emphasis(Vec<Self>),
}

impl Inline {
    pub fn parse(tokens: TokensRef<'_>) -> Self {
        let mut this = Vec::new();

        for span in tokens.spans() {
            match &span.as_ref()[0] {
                Token::OpenToken(ident) => match ident {
                    "strong" => token = Self::Strong(Vec::new()),
                },
            }
        }

        Self::Text(Default::default())
    }

    pub fn push(&mut self, item: Self) {
        match self {
            Self::Text(vec) => vec.push(item),
            Self::Strong(vec) => vec.push(item),
            Self::Emphasis(vec) => vec.push(item),
        }
    }
}

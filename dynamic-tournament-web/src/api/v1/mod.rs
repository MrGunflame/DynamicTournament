pub mod auth;
pub mod tournament;

#[derive(Clone, Debug)]
struct BadStatusCodeError {
    status: u16,
}

impl std::fmt::Display for BadStatusCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad status code: {}", self.status)
    }
}

impl std::error::Error for BadStatusCodeError {}

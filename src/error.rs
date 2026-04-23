use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    /// CLI argument / flag validation failure surfaced to the user.
    #[error("Usage: {0}")]
    Usage(String),

    /// Domain-level "looked it up, it isn't there".
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Not authenticated - run `garmin auth login`")]
    NotAuthenticated,

    /// Non-2xx response from the Garmin API.
    #[error("API error {status}: {body}")]
    Http { status: u16, body: String },

    #[error(transparent)]
    Transport(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// Machine-readable error code for structured JSON output.
    pub fn code(&self) -> &str {
        match self {
            Error::Usage(_) => "usage",
            Error::NotFound(_) => "not_found",
            Error::Auth(_) | Error::NotAuthenticated => "auth",
            Error::Http { status: 404, .. } => "not_found",
            Error::Http { status: 429, .. } => "rate_limit",
            Error::Http { .. } | Error::Transport(_) => "api",
            Error::Json(_) | Error::Io(_) | Error::Other(_) => "generic",
        }
    }
}

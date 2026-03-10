use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("API error: {0}")]
    Api(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Not authenticated — run `garmin auth login`")]
    NotAuthenticated,

    #[error("MFA required")]
    MfaRequired,

    #[error(transparent)]
    Http(#[from] reqwest::Error),

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
            Error::Auth(_) | Error::NotAuthenticated | Error::MfaRequired => "auth",
            Error::Api(msg) if msg.starts_with("404") => "not_found",
            Error::Api(msg) if msg.starts_with("429") => "rate_limit",
            Error::Api(_) => "api",
            Error::Http(_) => "api",
            Error::Json(_) => "generic",
            Error::Io(_) => "generic",
            Error::Other(_) => "generic",
        }
    }

    /// Process exit code.
    pub fn exit_code(&self) -> i32 {
        match self.code() {
            "auth" => 2,
            "not_found" => 3,
            "rate_limit" => 4,
            _ => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

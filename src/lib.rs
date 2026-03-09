pub mod auth;
pub mod client;
pub mod commands;
pub mod config;
pub mod error;
pub mod output;

pub use client::GarminClient;
pub use error::{Error, Result};
pub use output::{HumanReadable, Output};

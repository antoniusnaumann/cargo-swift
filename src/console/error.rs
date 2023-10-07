use std::fmt::Display;
use std::io::{self, stderr, Write};

#[derive(Debug)]
enum ErrorMessage {
    Simple(String),
    Stderr(Vec<u8>),
}

#[derive(Debug, thiserror::Error)]
pub struct Error {
    message: ErrorMessage,
}

impl From<Vec<u8>> for Error {
    fn from(value: Vec<u8>) -> Self {
        Self {
            message: ErrorMessage::Stderr(value),
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self::new(format!("{value:#}"))
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            ErrorMessage::Simple(msg) => f.write_str(msg),
            ErrorMessage::Stderr(buf) => f.write_str(&String::from_utf8_lossy(buf)),
        }
    }
}

impl Error {
    pub fn new<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            message: ErrorMessage::Simple(message.into()),
        }
    }

    pub fn print(&self) {
        match &self.message {
            ErrorMessage::Simple(msg) => eprintln!("{msg}"),
            ErrorMessage::Stderr(buf) => stderr().write_all(buf).unwrap(),
        }
    }
}

pub type Result<S> = std::result::Result<S, Error>;

// impl<S, E: Into<Error>> From<std::result::Result<S, E>> for Result<S> {}

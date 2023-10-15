use std::fmt::Display;
use std::io::{self, stderr, Write};
use std::ops::Deref;

use itertools::Itertools;

#[derive(Debug)]
enum ErrorMessage {
    Simple(String),
    Stderr(Vec<u8>),
}

impl ErrorMessage {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            ErrorMessage::Simple(msg) => msg.into_bytes(),
            ErrorMessage::Stderr(buf) => buf,
        }
    }
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

pub(crate) struct Errors(Vec<Error>);

impl Deref for Errors {
    type Target = Vec<Error>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Error>> for Errors {
    fn from(value: Vec<Error>) -> Self {
        Self(value)
    }
}

impl FromIterator<Error> for Errors {
    fn from_iter<T: IntoIterator<Item = Error>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl IntoIterator for Errors {
    type Item = Error;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Errors> for Result<()> {
    fn from(value: Errors) -> Self {
        if value.is_empty() {
            Ok(())
        } else {
            let message = ErrorMessage::Stderr(
                value
                    .into_iter()
                    .map(|e| e.message)
                    .map(ErrorMessage::into_bytes)
                    .concat(),
            );

            Err(Error { message })
        }
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

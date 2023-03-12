use std::{
    fmt::Display,
    io::{stderr, Write},
};

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

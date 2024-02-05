//! opzioni is a strongly typed configuration library for Rust.
//! It is designed to be easy to use and to provide a good user experience.
//! It uses serde for serialization and deserialization.
//! The currently supported formats are JSON, TOML and YAML.
#![deny(missing_docs)]

use std::{
    fmt::Display,
};

mod manager;
mod config;

#[cfg(feature = "tracing")]
#[macro_use]
extern crate tracing;

#[cfg(not(feature = "tokio"))]
type Lock<T> = std::sync::RwLock<T>;

#[cfg(feature = "tokio")]
type Lock<T> = tokio::sync::RwLock<T>;

/// The Error enum contains all possible errors that can occur while loading or saving a config file.
#[derive(Debug)]
pub enum Error {
    /// This error occurs when the config file could not be loaded. It contains an optional error message.
    ConfigLoadError(Option<String>),
    /// This error occurs when the file extension of the config file is not supported. It contains an optional error message.
    UnknownFileExtension(Option<String>),
    /// This error occurs when serializing or deserializing the config fails. It contains an optional error message.
    SerializationError(Option<String>),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::SerializationError(Some(err.to_string()))
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(Some(err.to_string()))
    }
}

#[cfg(feature = "toml")]
impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::SerializationError(Some(err.to_string()))
    }
}

#[cfg(feature = "toml")]
impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::SerializationError(Some(err.to_string()))
    }
}

#[cfg(feature = "yaml")]
impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::SerializationError(Some(err.to_string()))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConfigLoadError(Some(msg)) => write!(f, "opzioni::ConfigLoadError: {}", msg),
            Error::ConfigLoadError(None) => write!(f, "opzioni::ConfigLoadError"),
            Error::UnknownFileExtension(Some(msg)) => write!(f, "opzioni::UnknownFileExtension: {}", msg),
            Error::UnknownFileExtension(None) => write!(f, "opzioni::UnknownFileExtension"),
            Error::SerializationError(Some(msg)) => write!(f, "opzioni::SerializationError: {}", msg),
            Error::SerializationError(None) => write!(f, "opzioni::SerializationError"),
        }
    }
}

/// See [`config::sync::Config`]
#[cfg(feature = "tokio")]
pub type Config<T> = config::sync::Config<T>;

/// See [`config::std::Config`]
#[cfg(not(feature = "tokio"))]
pub type Config<T> = config::std::Config<T>;

//! opzioni is a strongly typed configuration library for Rust.
//! It is designed to be easy to use and to provide a good user experience.
//! It uses serde for serialization and deserialization.
//! The currently supported formats are JSON, TOML and YAML.
#![deny(missing_docs)]

use std::{
    fmt::Display,
    path::{self, Path},
};

mod manager;

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
            Error::ConfigLoadError(Some(msg)) => write!(f, "ConfigLoadError: {}", msg),
            Error::ConfigLoadError(None) => write!(f, "ConfigLoadError"),
            Error::UnknownFileExtension(Some(msg)) => write!(f, "UnknownFileExtension: {}", msg),
            Error::UnknownFileExtension(None) => write!(f, "UnknownFileExtension"),
            Error::SerializationError(Some(msg)) => write!(f, "SerializationError: {}", msg),
            Error::SerializationError(None) => write!(f, "SerializationError"),
        }
    }
}

/// The Config struct is the main entry point for the library.
#[derive(Debug, Default)]
pub struct Config<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync,
{
    config: Lock<T>,
    path: Option<path::PathBuf>,
}

impl<T> Config<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync,
{
    /// Creates a new Config struct.
    /// The config struct uses the default values of the given type T.
    /// It is better to directly load a config file with the [`Config::configure`] method, because empty will panic if the save method is called.
    ///
    /// # Example
    /// ```
    /// use opzioni::Config;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Default, Clone)]
    /// struct MyConfig {
    ///   name: String,
    ///   age: u8,
    /// }
    ///
    /// let config = Config::<MyConfig>::empty();
    /// ```
    pub fn empty() -> Self {
        #[cfg(feature = "tracing")]
        trace!("created empty config");
        Self {
            config: Lock::new(T::default()),
            path: None,
        }
    }

    /// Access the `Lock` of the config used to read and write the config.
    /// To save the config to file use the [`Config::save`] method.
    ///
    /// # Example
    /// ```
    /// use opzioni::Config;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Default, Clone)]
    /// struct MyConfig {
    ///   name: String,
    ///   age: u8,
    /// }
    ///
    /// let config = Config::<MyConfig>::empty();
    /// let config_lock = config.get();
    /// let mut config = config_lock.write().unwrap();
    /// config.name = "John".to_string();
    /// config.age = 42;
    /// ```
    pub fn get(&self) -> &Lock<T> {
        &self.config
    }

    /// Configure returns a new [`ConfigBuilder`] to load a config file from disk. See [`ConfigBuilder::load`] for more information.
    ///
    /// # Example
    /// ```
    /// use opzioni::Config;
    /// use serde::{Serialize, Deserialize};
    /// use std::path::Path;
    ///
    /// #[derive(Serialize, Deserialize, Default, Clone)]
    /// struct MyConfig {
    ///   name: String,
    ///   age: u8,
    /// }
    ///
    /// let config: Config<MyConfig> = Config::<MyConfig>::configure().load(Path::new("testconfig.json")).unwrap();
    /// ```
    pub fn configure() -> ConfigBuilder {
        ConfigBuilder {
            use_default_on_error: false,
        }
    }

    /// Saves the config to file. The file extension of the config file determines the format of the config file.
    /// The currently supported formats are JSON, TOML and YAML.
    /// The config file is overwritten.
    /// If the config file could not be saved, an error is returned.
    /// If the config file was loaded from disk, the config is saved to the same file.
    /// If the config file was created with [`Config::empty`], the method returns an error.
    ///
    /// # Example
    /// ```
    /// use opzioni::Config;
    /// use serde::{Serialize, Deserialize};
    /// use std::path::Path;
    ///
    /// #[derive(Serialize, Deserialize, Default, Clone)]
    /// struct MyConfig {
    ///  name: String,
    ///  age: u8,
    /// }
    ///
    /// let config: Config<MyConfig> = Config::<MyConfig>::configure().load(Path::new("testconfig.json")).unwrap();
    /// config.get().write().unwrap().name = "John".to_string();
    /// config.get().write().unwrap().age = 42;
    /// config.save().unwrap();
    /// ```
    #[cfg(not(feature = "tokio"))]
    pub fn save(&self) -> Result<(), Error> {
        match &self.path {
            Some(path) => match manager::for_file::<T>(path) {
                Ok(loader) => loader.save(&self.config.read().unwrap()),
                Err(err) => Err(err),
            },
            None => Err(Error::ConfigLoadError(None)),
        }
    }

    /// Saves the config to file. The file extension of the config file determines the format of the config file.
    /// The currently supported formats are JSON, TOML and YAML.
    /// The config file is overwritten.
    /// If the config file could not be saved, an error is returned.
    /// If the config file was loaded from disk, the config is saved to the same file.
    /// If the config file was created with [`Config::empty`], the method returns an error.
    ///
    /// # Example
    /// ```
    /// use opzioni::Config;
    /// use serde::{Serialize, Deserialize};
    /// use std::path::Path;
    ///
    /// #[derive(Serialize, Deserialize, Default, Clone)]
    /// struct MyConfig {
    ///  name: String,
    ///  age: u8,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let config: Config<MyConfig> = Config::<MyConfig>::configure().load(Path::new("testconfig.json")).unwrap();
    /// config.get().write().unwrap().name = "John".to_string();
    /// config.get().write().unwrap().age = 42;
    /// config.save().unwrap();
    /// # }
    /// ```
    #[cfg(feature = "tokio")]
    pub async fn save(&self) -> Result<(), Error> {
        match &self.path {
            Some(path) => match manager::for_file::<T>(path) {
                Ok(loader) => {
                    let cfg = self.config.read().await.clone();
                    loader.save(&cfg)
                }
                Err(err) => Err(err),
            },
            None => Err(Error::ConfigLoadError(None)),
        }
    }
}

/// The ConfigBuilder struct is used to load a config file from disk. See [`ConfigBuilder::load`] for more information.
pub struct ConfigBuilder {
    use_default_on_error: bool,
}

impl ConfigBuilder {
    fn handle_load_err<T>(&self, err: Error, path: &Path) -> Result<Config<T>, Error>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync,
    {
        if !self.use_default_on_error {
            return Err(err);
        }
        #[cfg(feature = "tracing")]
        trace!(
            error = err.to_string(),
            "using default config because of error"
        );
        return Ok(Config {
            config: Lock::new(T::default()),
            path: Some(path.to_path_buf()),
        });
    }

    /// If this method is called, the config will use the default values of the given type `T` if an error occurs while loading the config file.
    /// It causes the [`ConfigBuilder::load`] method to return a [`Config`] struct with the default values of the given type `T` instead of an error.
    ///
    /// # Example
    /// ```
    /// use opzioni::Config;
    /// use serde::{Serialize, Deserialize};
    /// use std::path::Path;
    ///
    /// #[derive(Serialize, Deserialize, Clone)]
    /// struct MyConfig {
    ///   name: String,
    ///   age: u8,
    /// }
    ///
    /// impl Default for MyConfig {
    ///   fn default() -> Self {
    ///     Self {
    ///       name: "John".to_string(),
    ///       age: 42,
    ///     }
    ///   }
    /// }
    ///
    /// let config: Config<MyConfig> = Config::<MyConfig>::configure().use_default_on_error().load(Path::new("testconfig.json")).unwrap();
    /// ```
    pub fn use_default_on_error(&mut self) -> &mut Self {
        self.use_default_on_error = true;
        self
    }

    /// Loads a config file from disk. The file extension of the config file determines the format of the config file.
    /// The currently supported formats are JSON, TOML and YAML.
    /// The config file must contain a valid config of the given type `T`.
    /// If the config file does not exist or is invalid, an error is returned. To use the default values of the given type `T` instead of an error, set [`ConfigBuilder::use_default_on_error`].
    ///
    /// # Example
    /// ```
    /// use opzioni::Config;
    /// use serde::{Serialize, Deserialize};
    /// use std::path::Path;
    ///
    /// #[derive(Serialize, Deserialize, Default, Clone)]
    /// struct MyConfig {
    ///    name: String,
    ///    age: u8,
    /// }
    ///
    /// let config: Config<MyConfig> = Config::<MyConfig>::configure().load(Path::new("testconfig.json")).unwrap();
    /// ```
    pub fn load<T>(&mut self, path: &Path) -> Result<Config<T>, Error>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync,
    {
        match manager::for_file(path) {
            Ok(loader) => match loader.load() {
                Ok(config) => Ok(Config {
                    config: Lock::new(config),
                    path: Some(path.to_path_buf()),
                }),
                Err(err) => self.handle_load_err(err, &path),
            },
            Err(err) => self.handle_load_err(err, &path),
        }
    }
}

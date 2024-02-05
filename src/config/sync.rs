use std::path;
use std::path::{Path, PathBuf};
use crate::{Error, Lock, manager};
use crate::manager::ConfigManager;

#[derive(Debug)]
pub struct Config<T>
    where T: serde::ser::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync {
    pub(crate) config: Lock<T>,
    pub(crate) path: Option<path::PathBuf>,
}

impl<T> Config<T>
    where T: serde::ser::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync {
    /// Creates a new Config struct.
    /// The config struct uses the default values of the given type T.
    /// It is better to directly load a config file with the [`crate::Config::configure`] method, because empty will panic if the save method is called.
    ///
    /// # Example
    /// ```
    /// use std::path::PathBuf;
    /// use opzioni::Config;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Default, Clone)]
    /// struct MyConfig {
    ///   name: String,
    ///   age: u8,
    /// }
    ///
    /// let config = Config::<MyConfig>::new(MyConfig::default(), PathBuf::new());
    /// ```
    pub fn new(config: T, path: PathBuf) -> Self {
        Self {
            config: Lock::new(config),
            path: Some(path),
        }
    }

    /// Access the `Lock` of the config used to read and write the config.
    /// To save the config to file use the [`crate::Config::save`] method.
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
    pub fn configure() -> ConfigBuilder<T> {
        ConfigBuilder {
            use_default_on_error: false,
        }
    }

    /// Saves the config to file. The file extension of the config file determines the format of the config file.
    /// The currently supported formats are JSON, TOML and YAML.
    /// The config file is overwritten.
    /// If the config file could not be saved, an error is returned.
    /// If the config file was loaded from disk, the config is saved to the same file.
    /// If the config file was created with [`crate::Config::empty`], the method returns an error.
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

impl<T> Default for Config<T>
    where T: serde::ser::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync {
    fn default() -> Self {
        Self {
            path: None,
            config: Lock::new(T::default()),
        }
    }
}

/// The ConfigBuilder struct is used to load a config file from disk. See [`ConfigBuilder::load`] for more information.
pub struct ConfigBuilder<T> where T: serde::ser::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync {
    use_default_on_error: bool,
}

impl<T> ConfigBuilder<T> where T: serde::ser::Serialize + serde::de::DeserializeOwned + Default + Clone + Send + Sync {
    fn handle_load_err(&self, err: Error, path: &Path) -> Result<crate::Config<T>, Error>
    {
        if !self.use_default_on_error {
            return Err(err);
        }
        #[cfg(feature = "tracing")]
        trace!(
            error = err.to_string(),
            "using default config because of error"
        );
        return Ok(crate::Config {
            config: Lock::new(T::default()),
            path: Some(path.to_path_buf()),
        });
    }

    /// If this method is called, the config will use the default values of the given type `T` if an error occurs while loading the config file.
    /// It causes the [`ConfigBuilder::load`] method to return a [`crate::Config`] struct with the default values of the given type `T` instead of an error.
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
    pub fn load(&mut self, path: &Path) -> Result<crate::Config<T>, Error>
    {
        match manager::for_file(path) {
            Ok(loader) => match loader.load() {
                Ok(config) => Ok(crate::Config {
                    config: Lock::new(config),
                    path: Some(path.to_path_buf()),
                }),
                Err(err) => self.handle_load_err(err, &path),
            },
            Err(err) => self.handle_load_err(err, &path),
        }
    }
}
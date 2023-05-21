use std::path::Path;

use crate::Error;

pub(crate) fn for_file<T>(path: &Path) -> Result<Box<dyn ConfigManager<T>>, Error>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Default,
{
    match path.extension() {
        Some(ext) => match ext.to_str() {
            #[cfg(feature = "json")]
            Some("json") => Ok(Box::new(json::JsonLoader::new(path))),
            #[cfg(feature = "toml")]
            Some("toml") => Ok(Box::new(toml::TomlLoader::new(path))),
            #[cfg(feature = "yaml")]
            Some("yaml") | Some("yml") => Ok(Box::new(yaml::YamlLoader::new(path))),
            _ => Err(Error::UnknownFileExtension(Some(
                ext.to_str().unwrap().to_string(),
            ))),
        },
        None => Err(Error::UnknownFileExtension(None)),
    }
}

pub(crate) trait ConfigManager<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Default,
{
    fn load(&self) -> Result<T, Error>;
    fn save(&self, config: &T) -> Result<(), Error>;
}

#[cfg(feature = "json")]
mod json {
    pub(crate) struct JsonLoader {
        path: std::path::PathBuf,
    }

    impl JsonLoader {
        pub(crate) fn new(path: &std::path::Path) -> Self {
            Self {
                path: path.to_path_buf(),
            }
        }
    }

    impl<T> super::ConfigManager<T> for JsonLoader
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Default,
    {
        fn load(&self) -> Result<T, super::Error> {
            trace!(file = ?self.path, "loading config");
            let data = std::fs::read_to_string(&self.path)?;
            let config: T = serde_json::from_str(&data)?;
            debug!(file = ?self.path, config = data, "loaded config");
            Ok(config)
        }

        fn save(&self, config: &T) -> Result<(), super::Error> {
            trace!(file = ?self.path, "saving config");
            let data = serde_json::to_string_pretty(config)?;
            std::fs::write(&self.path, &data)?;
            debug!(file = ?self.path, config = data, "saved config");
            Ok(())
        }
    }
}

#[cfg(feature = "toml")]
mod toml {
    pub(crate) struct TomlLoader {
        path: std::path::PathBuf,
    }

    impl TomlLoader {
        pub(crate) fn new(path: &std::path::Path) -> Self {
            Self {
                path: path.to_path_buf(),
            }
        }
    }

    impl<T> super::ConfigManager<T> for TomlLoader
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Default,
    {
        fn load(&self) -> Result<T, super::Error> {
            trace!(file = ?self.path, "loading config");
            let data = std::fs::read_to_string(&self.path)?;
            let config: T = toml::from_str(&data)?;
            debug!(file = ?self.path, config = data, "loaded config");
            Ok(config)
        }

        fn save(&self, config: &T) -> Result<(), super::Error> {
            trace!(file = ?self.path, "saving config");
            let data = toml::to_string_pretty(config)?;
            std::fs::write(&self.path, &data)?;
            debug!(file = ?self.path, config = data, "saved config");
            Ok(())
        }
    }
}

#[cfg(feature = "yaml")]
mod yaml {
    pub(crate) struct YamlLoader {
        path: std::path::PathBuf,
    }

    impl YamlLoader {
        pub(crate) fn new(path: &std::path::Path) -> Self {
            Self {
                path: path.to_path_buf(),
            }
        }
    }

    impl<T> super::ConfigManager<T> for YamlLoader
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Default,
    {
        fn load(&self) -> Result<T, super::Error> {
            trace!(file = ?self.path, "loading config");
            let data = std::fs::read_to_string(&self.path)?;
            let config: T = serde_yaml::from_str(&data)?;
            debug!(file = ?self.path, config = data, "loaded config");
            Ok(config)
        }

        fn save(&self, config: &T) -> Result<(), super::Error> {
            trace!(file = ?self.path, "saving config");
            let data = serde_yaml::to_string(config)?;
            std::fs::write(&self.path, &data)?;
            debug!(file = ?self.path, config = data, "saved config");
            Ok(())
        }
    }
}

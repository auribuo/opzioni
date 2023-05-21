# ⚙️ opzioni

A simple and fast configuration library for Rust.

# Installation

Add the library to your project via cargo:

`cargo add opzioni`

## Features
By default all features are enabled. This allows to work with JSON, TOML and YAML configs.

If you want to only use a subset run:

`cargo add opzioni --no-default-features --features json`

Replace json with the features you want to enable.
The available features are

- json
- yaml
- toml

# Usage

First create a struct implementing `Serialize`, `Deserialize` and `Default`

```rust
use serde::{Serialize, Deserialize}

#[derive(Serialize, Deserialize, Default)]
struct MyConfig {
    name: String,
    age: u8
}
```

Then just load the config file using opzioni:

```rust
let config = opzioni::Config::<MyConfig>::configure().load(std::path::Path::new("myconfig.yml")).unwrap();
```

opzioni exposes a `RwLock` which can be used to modify the config data:

```rust
let lock = config.get();
let data = lock.read().unwrap();
// or
let mut data = lock.write().unwrap();
```

Once you are done working with the config you can save the changes to disk by calling `save`:

```rust
config.save().unwrap();
```


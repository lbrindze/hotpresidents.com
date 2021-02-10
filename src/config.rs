use serde::{self, Deserialize};

#[derive(Default, Deserialize, Debug)]
pub struct Config {
    pub data_load_uri: String,
    pub api_key: String,
    pub save_file: String,
}

pub fn from_envvar() -> Config {
    match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{:#?}", error);
            Default::default()
        }
    }
}

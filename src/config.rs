use serde::{self, Deserialize};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub api_key: String,
    #[serde(default)]
    pub data_load_uri: String,
    #[serde(default)]
    pub save_file: String,
    #[serde(default)]
    pub save_timeout: usize,
    #[serde(default)]
    pub host_address: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            data_load_uri:
                "https://api.airtable.com/v0/appWPQd75Wh8IVPa0/Table%201?view=Grid%20view"
                    .to_string(),
            api_key: "".to_string(),
            save_file: "president_votes_state.data".to_string(),
            host_address: "0.0.0.0:8080".to_string(),
            save_timeout: 30,
        }
    }
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

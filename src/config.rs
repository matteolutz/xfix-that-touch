use std::fs::File;

use serde_derive::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &'static str = ".xfixtouch.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct XFixTouchscreen {
    pub vendor: String,
    pub id_path: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct XFixConfig {
    pub touchscreens: Vec<XFixTouchscreen>,
}

impl XFixConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let home_dir_buf = home::home_dir();
        if home_dir_buf.is_none() {
            return Err("Could not find home directory".into());
        }

        let config_file_path = home::home_dir().unwrap().as_path().join(CONFIG_FILE_NAME);

        if !config_file_path.exists() {
            println!(
                "[xfix] Config file not found, creating a new one at {}",
                config_file_path.display()
            );

            let default_config = Self::default();

            let f = File::create(config_file_path)?;
            serde_json::to_writer(f, &default_config)?;

            return Ok(default_config);
        }

        let file = File::open(config_file_path)?;
        let config: Self = serde_json::from_reader(file)?;

        Ok(config)
    }
}

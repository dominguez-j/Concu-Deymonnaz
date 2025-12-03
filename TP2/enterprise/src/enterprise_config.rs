use config::{Config, ConfigError, File, FileFormat};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CardConfig {
    pub id: u32,
    pub limit: u32,
}

#[derive(Debug, Clone)]
pub struct EnterpriseConfig {
    pub id: u32,
    pub balance: u32,
    pub cards: Vec<CardConfig>,
}

impl EnterpriseConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(File::new(path, FileFormat::Toml).required(true))
            .build()?;
        Ok(Self {
            id: settings.get::<u32>("id").unwrap(),
            balance: settings.get::<u32>("balance").unwrap(),
            cards: settings
                .get::<Vec<HashMap<String, u32>>>("cards")
                .unwrap()
                .into_iter()
                .map(|card| CardConfig {
                    id: card.get("id").unwrap().to_string().parse::<u32>().unwrap(),
                    limit: card
                        .get("limit")
                        .unwrap()
                        .to_string()
                        .parse::<u32>()
                        .unwrap(),
                })
                .collect(),
        })
    }
}

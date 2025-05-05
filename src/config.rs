use std::path::{PathBuf, Path};
use serde::Deserialize;
use eyre::Result;
use url::Url;
use alloy::primitives::{U256, uint};
use crate::protocols::{UniV2, UniV3, BoxedProtocol};


#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ProtocolType {
    UniV3(UniV3),
    UniV2(UniV2),
}

impl ProtocolType {

    pub fn into_boxed(self) -> BoxedProtocol {
        match self {
            Self::UniV3(protocol) => Box::new(protocol),
            Self::UniV2(protocol) => Box::new(protocol),
        }
    }

}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub chain_configs: Vec<ChainConfig>,
    pub precision: u8,
}

#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub rpc_url: Url,
    pub reth_db_path: PathBuf,
    pub default_start_block: u64,
    pub default_end_block: u64,
    pub price_sources: Vec<PriceSource>,
}

#[derive(Debug, Deserialize)]
pub struct PriceSource {
    pub name: String,
    pub inverse_it: bool,
    pub protocol: ProtocolType,
}

impl Config {

    pub fn try_from_file(path: &Path) -> Result<Self> {
        let config_str_raw = std::fs::read_to_string(path)
            .map_err(|e| eyre::eyre!("Failed to read config file: {}", e))?;
        let config_str_rendered = envsubst::substitute(
            config_str_raw, 
            &dotenv::vars().collect()
        )?;
        let config: Config = toml::from_str(&config_str_rendered)
            .map_err(|e| eyre::eyre!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

}

use std::collections::HashSet;
use std::ops::Range;
use std::path::{Path, PathBuf};

use eyre::{Result, eyre};
use rayon::prelude::*;
use fxhash::FxHashMap;
use serde::{Serialize, Deserialize};
use futures::future;
use url::Url;

use alloy::primitives::{Address, U256};
use alloy::providers::RootProvider;

use crate::config::PriceSource;
use crate::protocols::{self, BoxedProtocol, TokenInfo};
use crate::reth_utils::{self, LocalProviderFactory};


#[derive(Default)]
pub struct PriceFetcherBuilder {
    precision: u8,
    reth_db_path: Option<PathBuf>,
    rpc_url: Option<Url>,
    price_sources: Option<Vec<PriceSource>>,
}

impl PriceFetcherBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn precision(mut self, precision: u8) -> Self {
        self.precision = precision;
        self
    }

    pub fn reth_db_path(mut self, path: impl AsRef<Path>) -> Self {
        self.reth_db_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn rpc_url(mut self, url: Url) -> Self {
        self.rpc_url = Some(url);
        self
    }

    pub fn price_sources(mut self, sources: Vec<PriceSource>) -> Self {
        self.price_sources = Some(sources);
        self
    }

    pub async fn build(self) -> Result<PriceFetcher> {
        let reth_path = self.reth_db_path.ok_or_else(|| eyre!("reth_db_path not provided"))?;
        let rpc_url = self.rpc_url.ok_or_else(|| eyre!("rpc_url not provided"))?;
        let price_sources = self.price_sources.ok_or_else(|| eyre!("price_sources not provided"))?;

        let provider_factory = reth_utils::build_provider_factory(&reth_path)?;
        let rpc_provider = RootProvider::new_http(rpc_url);

        let parsed_price_sources = Self::parse_price_sources(&rpc_provider, price_sources).await?;
        let token_infos = Self::fetch_token_infos(&rpc_provider, &parsed_price_sources).await?;
        let precision_factor = U256::from(10u64).pow(U256::from(self.precision));

        Ok(PriceFetcher {
            precision_factor,
            token_infos,
            price_sources: parsed_price_sources,
            provider_factory,
        })
    }

    async fn parse_price_sources(
        provider: &RootProvider,
        price_sources: Vec<PriceSource>,
    ) -> Result<Vec<ParsedPriceSource>> {
        let futs = price_sources
            .into_iter()
            .map(|source| {
                let provider = provider.clone();
                async move {
                    let protocol = source.protocol.into_boxed();
                    let tokens = protocol.fetch_tokens(&provider).await?;
                    Ok::<_, eyre::Report>(ParsedPriceSource {
                        inverse_it: source.inverse_it,
                        protocol,
                        tokens,
                    })
                }
            });
        Ok(future::try_join_all(futs).await?)
    }

    async fn fetch_token_infos(
        provider: &RootProvider,
        price_sources: &[ParsedPriceSource],
    ) -> Result<FxHashMap<Address, TokenInfo>> {
        let futs = price_sources
            .iter()
            .flat_map(|ps| ps.tokens.iter())
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|&token| {
                let provider = provider.clone();
                async move {
                    let info = protocols::fetch_token_info(&provider, token).await?;
                    Ok::<_, eyre::Report>((token, info))
                }
            });
        Ok(future::try_join_all(futs).await?.into_iter().collect())
    }
}

pub struct PriceFetcher {
    token_infos: FxHashMap<Address, TokenInfo>,
    price_sources: Vec<ParsedPriceSource>,
    provider_factory: LocalProviderFactory,
    precision_factor: U256,
}

impl PriceFetcher {
    pub fn fetch_prices(&self, block_range: Range<u64>) -> Result<Vec<PriceFetcherResult>> {
        Ok(block_range
            .into_par_iter()
            .map(|block| self.fetch_prices_for_block(block))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect()
        )
    }

    pub fn fetch_prices_for_block(&self, block_num: u64) -> Result<Vec<PriceFetcherResult>> {
        let hist_provider = self.provider_factory.history_by_block_number(block_num)?;
        self.price_sources
            .iter()
            .map(|ps| {
                let slot = ps.protocol.storage_slot();
                let target = ps.protocol.storage_target();
                let storage = hist_provider
                    .storage(target, slot)?
                    .ok_or_else(|| eyre!("storage slot {slot} empty at block {block_num}"))?;

                let price = ps.protocol.retrieve_price_from_storage(
                    storage,
                    ps.inverse_it,
                    [
                        self.token_infos[&ps.tokens[0]].dec_denom,
                        self.token_infos[&ps.tokens[1]].dec_denom,
                    ],
                    self.precision_factor,
                )?;

                let (base_token, quote_token) =
                    if ps.inverse_it {
                        (ps.tokens[1], ps.tokens[0])
                    } else {
                        (ps.tokens[0], ps.tokens[1])
                    };
                Ok(PriceFetcherResult {
                    block_num,
                    source: ps.protocol.name(),
                    price,
                    quote_token,
                    base_token,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFetcherResult {
    pub block_num: u64,
    pub source: String,
    #[serde(serialize_with = "serialize_u256_to_dec")]
    pub price: U256,
    pub quote_token: Address,
    pub base_token: Address,
}

struct ParsedPriceSource {
    inverse_it: bool,
    protocol: BoxedProtocol,
    tokens: [Address; 2],
}

fn serialize_u256_to_dec<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

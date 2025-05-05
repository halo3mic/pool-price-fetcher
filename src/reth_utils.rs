use std::path::Path;
use std::sync::Arc;
use eyre::Result;

use reth_ethereum::node::{api::NodeTypesWithDBAdapter, EthereumNode};
use reth_ethereum::chainspec::ChainSpecBuilder;
use reth_ethereum::provider::{
    db::{mdbx::DatabaseArguments, open_db_read_only, ClientVersion, DatabaseEnv},
    providers::StaticFileProvider,
    HeaderProvider,
    ProviderFactory,
};

pub type LocalProviderFactory = ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>;

pub fn build_provider_factory(db_path: &Path) -> Result<LocalProviderFactory> {
    let db_path = Path::new(&db_path);
    let db = Arc::new(open_db_read_only(
        db_path.join("db").as_path(),
        DatabaseArguments::new(ClientVersion::default()),
    )?);
    let spec = Arc::new(ChainSpecBuilder::mainnet().build());
    let factory = ProviderFactory::<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>::new(
        db.clone(),
        spec.clone(),
        StaticFileProvider::read_only(db_path.join("static_files"), true)?,
    );
    Ok(factory)
}

pub fn block_num_to_timestamp(
    provider: &LocalProviderFactory,
    block_num: u64,
) -> Result<u64> {
    provider
        .header_by_number(block_num)?
        .map(|h| h.timestamp)
        .ok_or_else(|| eyre::eyre!("Header not found for block number {}", block_num))
}


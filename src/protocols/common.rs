use alloy::primitives::{B256, U256, Address};
use eyre::Result;


#[async_trait::async_trait]
pub trait Protocol {

    fn name(&self) -> String;

    async fn fetch_tokens(
        &self,
        provider: &alloy::providers::RootProvider,
    ) -> Result<[Address; 2]>;

    fn storage_slot(&self) -> B256;

    fn storage_target(&self) -> Address;

    fn retrieve_price_from_storage(
        &self,
        storage: U256,
        inverse_it: bool,
        dec_denoms: [U256; 2],
        precision_factor: U256,
    ) -> Result<U256>;

}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    symbol: String,
    decimals: u8,
    pub dec_denom: U256,
}


use alloy::providers::RootProvider;
use alloy::sol;

sol!{
    #[sol(rpc)]
    interface IUniswapPool {
        function token0() external view returns (address);
        function token1() external view returns (address);
    }

    #[sol(rpc)]
    interface IERC20 {
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
    }
}

pub async fn uniswap_pool_tokens(
    provider: &RootProvider,
    pool: Address,
) -> Result<[Address; 2]> {
    let pool_contract = IUniswapPool::new(pool, provider);
    let token0 = pool_contract.token0().call().await?;
    let token1 = pool_contract.token1().call().await?;
    Ok([token0, token1])
}

pub async fn fetch_token_info(
    provider: &RootProvider,
    token: Address,
) -> Result<TokenInfo> {
    let token_contract = IERC20::new(token, provider);
    let symbol = token_contract.symbol().call().await?;
    let decimals = token_contract.decimals().call().await?;
    let dec_denom = U256::from(10u64).pow(U256::from(decimals));
    Ok(TokenInfo { symbol, decimals, dec_denom })
}
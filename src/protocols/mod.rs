mod common;
mod univ2;
mod univ3;

pub use common::fetch_token_info;
pub use common::{Protocol, TokenInfo};
pub use univ2::UniV2;
pub use univ3::UniV3;
pub type BoxedProtocol = Box<dyn Protocol + Send + Sync>;
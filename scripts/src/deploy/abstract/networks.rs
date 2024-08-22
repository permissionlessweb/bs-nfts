use cw_orch::environment::{ChainKind, NetworkInfo};
//////////////// SUPPORTED NETWORK CONFIGS ////////////////
/// Add more chains in SUPPORTED_CHAINS to include in account framework instance.
use cw_orch::prelude::{networks::UNI_6, *};

pub const SUPPORTED_CHAINS: &[ChainInfo] = &[UNI_6, BITSONG_MAINNET];
pub const GAS_TO_DEPLOY: u64 = 60_000_000;

/// Bitsong: <https://github.com/cosmos/chain-registry/blob/master/bitsong/chain.json>
pub const BITSONG_NETWORK: NetworkInfo = NetworkInfo {
    chain_name: "Bitsong",
    pub_address_prefix: "bitsong",
    coin_type: 639u32,
};
pub const BITSONG_MAINNET: ChainInfo = ChainInfo {
    kind: ChainKind::Mainnet,
    chain_id: "bitsong-2b",
    gas_denom: "ubtsg",
    gas_price: 0.025,
    grpc_urls: &["http://grpc"],
    network_info: BITSONG_NETWORK,
    lcd_url: None,
    fcd_url: None,
};
// pub const BITSONG_TESTNET: ChainInfo = ChainInfo {
//     kind: ChainKind::Testnet,
//     chain_id: "bobnet",
//     gas_denom: "ubtsg",
//     gas_price: 0.025,
//     grpc_urls: &["http://grpc"],
//     network_info: BITSONG_NETWORK,
//     lcd_url: None,
//     fcd_url: None,
// };

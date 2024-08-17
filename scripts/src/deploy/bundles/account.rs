use crate::BtsgAccountTestSuite;
use bs721_account_minter::msg::InstantiateMsg as AccountMinterInitMsg;
use btsg_account::market::{ExecuteMsgFns as _, InstantiateMsg as AccountMarketInitMsg};
use cosmwasm_std::Decimal;
use cw_orch::prelude::*;

// Bitsong Accounts `Deploy` Suite
impl<Chain: CwEnv> cw_orch::contract::Deploy<Chain> for BtsgAccountTestSuite<Chain> {
    // We don't have a custom error type
    type Error = CwOrchError;
    type DeployData = Addr;

    fn store_on(chain: Chain) -> Result<Self, Self::Error> {
        let suite = BtsgAccountTestSuite::new(chain.clone());
        suite.upload()?;
        Ok(suite)
    }

    fn deployed_state_file_path() -> Option<String> {
        None
    }

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
        vec![Box::new(&mut self.account)]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let suite = Self::new(chain.clone());
        Ok(suite)
    }

    fn deploy_on(chain: Chain, _data: Self::DeployData) -> Result<Self, Self::Error> {
        // ########### Upload ##############
        let mut suite: BtsgAccountTestSuite<Chain> = BtsgAccountTestSuite::store_on(chain.clone())?;

        // ########## Instantiate #############
        // account marketplace
        suite.market.instantiate(
            &AccountMarketInitMsg {
                trading_fee_bps: 0u64,
                min_price: 100u128.into(),
                ask_interval: 30u64,
                max_renewals_per_block: 10u32,
                valid_bid_query_limit: 100u32,
                renew_window: 1000u64,
                renewal_bid_percentage: Decimal::one(),
                operator: chain.sender_addr().to_string(),
            },
            None,
            None,
        )?;
        // Account Minter
        // On instantitate, bs721-account contract is created by minter contract.
        // We grab this contract addr from response events, and set address in internal test suite state.
        let bs721_account = suite
            .minter
            .instantiate(
                &AccountMinterInitMsg {
                    admin: None,
                    verifier: None,
                    collection_code_id: suite.account.code_id()?,
                    marketplace_addr: suite.market.addr_str()?,
                    min_name_length: 3u32,
                    max_name_length: 128u32,
                    base_price: 10u128.into(),
                },
                None,
                None,
            )?
            .event_attr_value("wasm", "bs721_account_address")?;

        suite
            .account
            .set_default_address(&Addr::unchecked(bs721_account));

        // Provide marketplace with collection and minter contracts.
        suite
            .market
            .setup(suite.account.address()?, suite.minter.address()?)?;

        Ok(suite)
    }
}

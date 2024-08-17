use crate::BtsgAccountTestSuite;
use bs721_account_minter::msg::InstantiateMsg as AccountMinterInitMsg;
use btsg_account::market::{ExecuteMsgFns as _, InstantiateMsg as AccountMarketInitMsg};
use cosmwasm_std::Decimal;
use cw_orch::prelude::*;
use cw_orch::{anyhow, prelude::*};
// Library for various combinations (bundles) of contracts in a single `Deploy` trait implementation.
pub mod account;

/// MockBech32 functions and helpers
impl BtsgAccountTestSuite<MockBech32> {
    pub fn default_setup(&mut self, mock: MockBech32) -> anyhow::Result<()> {
        self.upload()?;
        // instantiate

        self.market.instantiate(
            &AccountMarketInitMsg {
                trading_fee_bps: 0u64,
                min_price: 100u128.into(),
                ask_interval: 30u64,
                max_renewals_per_block: 10u32,
                valid_bid_query_limit: 100u32,
                renew_window: 1000u64,
                renewal_bid_percentage: Decimal::one(),
                operator: mock.sender_addr().to_string(),
            },
            None,
            None,
        )?;
        // Account Minter
        // On instantitate, bs721-account contract is created by minter contract.
        // We grab this contract addr from response events, and set address in internal test suite state.
        let bs721_account = self
            .minter
            .instantiate(
                &AccountMinterInitMsg {
                    admin: None,
                    verifier: None,
                    collection_code_id: self.account.code_id()?,
                    marketplace_addr: self.market.addr_str()?,
                    min_name_length: 3u32,
                    max_name_length: 128u32,
                    base_price: 10u128.into(),
                },
                None,
                None,
            )?
            .event_attr_value("wasm", "bs721_account_address")?;

        self.account
            .set_default_address(&Addr::unchecked(bs721_account));

        // Provide marketplace with collection and minter contracts.
        self.market
            .setup(self.account.address()?, self.minter.address()?)?;

        Ok(())
    }
}

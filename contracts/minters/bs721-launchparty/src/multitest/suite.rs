use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, Addr, Coin, Empty, StdError, Timestamp, Uint128, Uint256};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use derivative::Derivative;

use bs721_base::{msg::QueryMsg as Bs721BaseQueryMsg, InstantiateMsg as Bs721InitMsg};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MaxPerAddressResponse, PartyType, QueryMsg},
    state::Config,
};
use easy_addr::addr;
pub const CREATOR: &str = addr!("creator");
pub const PAYMENT_RECIPIENT: &str = addr!("contract2");
pub const BS721_ADMIN: &str = addr!("bs721_admin");
pub const ADDRESS1: &str = addr!("address1");
pub const ADDRESS2: &str = addr!("address2");
pub const REFERRAL: &str = addr!("referral");

/// Helper function to create a wrapper around the bs721 base contract
pub fn contract_bs721_base() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(
        bs721_base::entry::execute,
        bs721_base::entry::instantiate,
        bs721_base::entry::query,
    ))
}

/// Helper function to create a wrapper around the launchparty contract
pub fn contract_launchparty() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ))
}

// -------------------------------------------------------------------------------------------------
// TestSuiteBuilder
// -------------------------------------------------------------------------------------------------

/// Stores genesis configuration for tests. This strcture is used initialize a TestSuite.
#[derive(Derivative, Debug)]
#[derivative(Default(new = "true"))]
pub struct TestSuiteBuilder {
    /// Creator of the collection. If not provided it will be the sender.
    #[derivative(Default(value = "String::from(CREATOR)"))]
    pub creator: String,
    /// BS721 token symbol.
    #[derivative(Default(value = "String::from(\"album\")"))]
    pub symbol: String,
    #[derivative(Default(value = "String::from(\"name\")"))]
    pub name: String,
    #[derivative(Default(value = "String::from(\"uri\")"))]
    pub uri: String,
    /// Price of single nft minting.
    pub price: Coin,
    /// Maximum numer of tokens an address can mint
    #[derivative(Default(value = "None"))]
    pub max_per_address: Option<u32>,
    /// BS721 token uri.
    pub seller_fee_bps: u16,
    pub referral_fee_bps: u16,
    pub protocol_fee_bps: u16,
    /// Start time of the launchparty.
    #[derivative(Default(value = "Timestamp::from_seconds(1571797419)"))]
    pub start_time: Timestamp,
    /// End condition of the collection launchparty.
    #[derivative(Default(value = "PartyType::MaxEdition(1)"))]
    pub party_type: PartyType,
    pub init_funds: Vec<(Addr, Vec<Coin>)>,
}

impl TestSuiteBuilder {
    /// Helper function to define the price of the bs721 collection.
    pub fn with_price(mut self, price: Coin) -> Self {
        self.price = price;
        self
    }

    /// Helper function to set referral fee bp.
    pub fn with_referral_fee_bps(mut self, referral_fee_bps: u16) -> Self {
        self.referral_fee_bps = referral_fee_bps;
        self
    }

    /// Helper function to define the end condition of the launchparty.
    pub fn with_party_type(mut self, party_type: PartyType) -> Self {
        self.party_type = party_type;
        self
    }

    /// Helper function to initialize the bank module with funds associated to particular addresses.
    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.init_funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    /// Helper function to define maximum number of tokens an address can mint.
    pub fn with_max_per_address(mut self, max_per_address: u32) -> Self {
        self.max_per_address = Some(max_per_address);
        self
    }

    /// Helper function to instantiate the launchparty contract with parameters defined by the TestSuiteBuilder
    pub fn instantiate_launchparty(&self, app: &mut App, code_id: u64, bs721_code_id: u64) -> Addr {
        // could we also use mem to optimize code and avoid clone
        let init_msg = InstantiateMsg {
            symbol: self.symbol.clone(),
            name: self.name.clone(),
            uri: self.uri.clone(),
            price: match self.price.amount == Uint256::zero() {
                true => coin(1, "ubtsg"),
                false => self.price.clone(),
            },
            max_per_address: self.max_per_address,
            seller_fee_bps: self.seller_fee_bps,
            referral_fee_bps: self.referral_fee_bps,
            protocol_fee_bps: self.protocol_fee_bps,
            start_time: self.start_time,
            party_type: self.party_type.clone(),
            bs721_code_id,
            payment_address: PAYMENT_RECIPIENT.to_string(),
            bs721_admin: BS721_ADMIN.to_string(),
        };

        app.instantiate_contract(
            code_id,
            Addr::unchecked(self.creator.clone()),
            &init_msg,
            &[],
            "Launchparty",
            None,
        )
        .unwrap()
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app: App = App::default();

        app.update_block(|block| block.time = Timestamp::from_seconds(1571797419));

        let bs721_base_code_id = app.store_code(contract_bs721_base());
        let launchparty_code_id = app.store_code(contract_launchparty());

        let contract_address =
            self.instantiate_launchparty(&mut app, launchparty_code_id, bs721_base_code_id);

        app.init_modules(|router, _, storage| -> AnyResult<()> {
            for (addr, coin) in self.init_funds {
                router.bank.init_balance(storage, &addr, coin).unwrap();
            }
            Ok(())
        })
        .unwrap();

        let bs721_addr = app
            .instantiate_contract(
                bs721_base_code_id,
                contract_address.clone(),
                &Bs721InitMsg {
                    name: self.name.clone(),
                    symbol: self.symbol.clone(),
                    uri: Some(self.uri.clone()),
                    minter: contract_address.to_string(),
                },
                &[],
                "Bs721-Contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked(self.creator.clone()),
            contract_address.clone(),
            &ExecuteMsg::SetNftAddress {
                nft_addr: bs721_addr.to_string(),
            },
            &[],
        )
        .unwrap();
        Suite {
            app,
            contract_address,
            bs721_addr,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
    /// Address of the launchparty contract.
    contract_address: Addr,
    /// address of bs721 nft address we manually created and then set. workaround since v2 cosmwasmstd
    bs721_addr: Addr,
}

impl Suite {
    /// Returns the contract address.
    fn contract_address(&self) -> Addr {
        self.contract_address.clone()
    }
    /// Returns the contract address.
    pub fn replaced_cw721_addr(&self) -> Addr {
        self.bs721_addr.clone()
    }

    /// Helper function to mint a bs721 token. The sender is defined as a const.
    pub fn mint(
        &mut self,
        sender: impl ToString,
        referral: Option<String>,
        amount: u32,
        funds: Option<Coin>,
    ) -> Result<AppResponse, StdError> {
        let msg = ExecuteMsg::Mint { referral, amount };

        let send_funds: Vec<Coin> = funds.map_or_else(Vec::new, |sent_coin| vec![sent_coin]);

        self.app.execute_contract(
            Addr::unchecked(sender.to_string()),
            self.contract_address(),
            &msg,
            &send_funds,
        )
    }

    /// Helper function to query launchparty contract configuration.
    pub fn query_config(&self) -> Config {
        self.app
            .wrap()
            .query_wasm_smart(self.contract_address(), &QueryMsg::GetConfig {})
            .unwrap()
    }

    /// Helper function to query the max number of tokens an address can mint.
    pub fn query_max_per_address(&self, address: impl Into<String>) -> MaxPerAddressResponse {
        self.app
            .wrap()
            .query_wasm_smart(
                self.contract_address(),
                &QueryMsg::MaxPerAddress {
                    address: address.into(),
                },
            )
            .unwrap()
    }

    /// Helper function to query the balance of a specific address.
    pub fn query_address_balance(
        &self,
        address: impl Into<String>,
        denom: impl Into<String>,
    ) -> Coin {
        self.app
            .wrap()
            .query_balance(address.into(), denom.into())
            .unwrap()
    }

    pub fn query_nft_token(
        &self,
        bs721_address: impl Into<String>,
        owner: impl Into<String>,
    ) -> Vec<String> {
        let query: Bs721BaseQueryMsg<Empty> = Bs721BaseQueryMsg::Tokens {
            owner: owner.into(),
            start_after: None,
            limit: None,
        };
        let resp: bs721::TokensResponse = self
            .app
            .wrap()
            .query_wasm_smart(bs721_address.into(), &query)
            .unwrap();
        resp.tokens
    }
}

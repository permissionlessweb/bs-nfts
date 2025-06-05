use crate::base::Bs721Base;
use crate::factory::Bs721Factory;
use crate::launchparty::Btsg721Launchparty;
use crate::wavs::BtsgWavsAuthenticator;
use bs721_base::InstantiateMsg;
use bs721_launchparty::msg::{InstantiateMsg as Bs721LaunchInitMsg, PartyType};

use cosmwasm_std::{StdError, Timestamp};
use cw_orch::prelude::*;
pub struct BtsgNftSuite<Chain>
where
    Chain: cw_orch::prelude::CwEnv,
{
    pub bs721base: Bs721Base<Chain, Empty>,
    pub bs721launchparty: Btsg721Launchparty<Chain>,
    pub wavs: BtsgWavsAuthenticator<Chain>,
}

pub const BLS_PUBKEY: &str = "";

impl<Chain: CwEnv> BtsgNftSuite<Chain> {
    pub fn new(chain: Chain) -> BtsgNftSuite<Chain> {
        BtsgNftSuite::<Chain> {
            wavs: BtsgWavsAuthenticator::new("btsg_wavs", chain.clone()),
            bs721base: Bs721Base::new("bs721_base", chain.clone()),
            bs721launchparty: Btsg721Launchparty::new("bs721_launchparty", chain.clone()),
        }
    }

    pub fn upload(&self) -> Result<(), CwOrchError> {
        let _wavs = self.wavs.upload()?.uploaded_code_id()?;
        let _bs721base = self.bs721base.upload()?.uploaded_code_id()?;
        let _launchpad = self.bs721launchparty.upload()?.uploaded_code_id()?;

        // println!("account collection code-id: {}", _acc);
        // println!("account minter code-id: {}", _minter);
        // println!("account minter code-id: {}", _market);
        Ok(())
    }
}

// Bitsong Accounts `Deploy` Suite
impl<Chain: CwEnv> cw_orch::contract::Deploy<Chain> for BtsgNftSuite<Chain> {
    // We don't have a custom error type
    type Error = CwOrchError;
    type DeployData = Addr;

    fn store_on(chain: Chain) -> Result<Self, Self::Error> {
        let suite = BtsgNftSuite::new(chain.clone());
        suite.upload()?;
        Ok(suite)
    }

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
        vec![
            Box::new(&mut self.wavs),
            Box::new(&mut self.bs721launchparty),
            Box::new(&mut self.bs721base),
        ]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let suite = Self::new(chain.clone());
        Ok(suite)
    }

    fn deploy_on(chain: Chain, data: Self::DeployData) -> Result<Self, Self::Error> {
        // ########### Upload ##############
        let abstrct = Self::store_on(chain.clone())?;

        // ########### Upload ##############
        let mut suite: BtsgNftSuite<Chain> = BtsgNftSuite::store_on(chain.clone())?;
        let start_time = chain
            .node_querier()
            .latest_block()
            .map_err(|e| StdError::generic_err(e.to_string()))?
            .time
            .plus_seconds(60u64);

        suite.bs721launchparty.instantiate(
            &Bs721LaunchInitMsg {
                symbol: "MONK".into(),
                name: "monk on iron mountain".into(),
                uri: "ipfs://".into(),
                price: Coin::new(100u128, "ubtsg"),
                max_per_address: Some(10u32),
                payment_address: chain.sender_addr().to_string(),
                seller_fee_bps: 2,
                referral_fee_bps: 2,
                protocol_fee_bps: 2,
                start_time,
                party_type: PartyType::MaxEdition(100),
                bs721_code_id: suite.bs721base.code_id()?,
                bs721_admin: chain.sender_addr().to_string(),
            },
            None,
            &[],
        )?;

        Ok(suite)
    }
}

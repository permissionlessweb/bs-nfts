use crate::curve::Bs721Curve;
use crate::launchparty::Btsg721Launchparty;
use crate::royalties::Bs721Royalties;
use crate::wavs::BtsgWavsAuthenticator;
use crate::{base::Bs721Base, factory::Bs721Factory};
use bs721_launchparty::msg::{InstantiateMsg as Bs721LaunchInitMsg, PartyType};

use cosmwasm_std::StdError;
use cw_orch::prelude::*;
pub struct BtsgNftSuite<Chain>
where
    Chain: cw_orch::prelude::CwEnv,
{
    pub base: Bs721Base<Chain, Empty, Empty>,
    pub curve: Bs721Curve<Chain>,
    pub royalties: Bs721Royalties<Chain>,
    pub launchparty: Btsg721Launchparty<Chain>,
    pub factory: Bs721Factory<Chain>,
    pub wavs: BtsgWavsAuthenticator<Chain>,
}

pub const BLS_PUBKEY: &str = "";

impl<Chain: CwEnv> BtsgNftSuite<Chain> {
    pub fn new(chain: Chain) -> BtsgNftSuite<Chain> {
        BtsgNftSuite::<Chain> {
            wavs: BtsgWavsAuthenticator::new("btsg_wavs", chain.clone()),
            base: Bs721Base::new("bs721_base", chain.clone()),
            curve: Bs721Curve::new("bs721_curve", chain.clone()),
            launchparty: Btsg721Launchparty::new("bs721_launchparty", chain.clone()),
            factory: Bs721Factory::new("bs721_factory", chain.clone()),
            royalties: Bs721Royalties::new("bs721_royalties", chain.clone()),
        }
    }

    pub fn upload(&self) -> Result<(), CwOrchError> {
        let _wavs = self.wavs.upload()?.uploaded_code_id()?;
        let _bs721base = self.base.upload()?.uploaded_code_id()?;
        let _launchpad = self.launchparty.upload()?.uploaded_code_id()?;
        let factory = self.factory.upload()?.uploaded_code_id()?;
        let royalties = self.royalties.upload()?.uploaded_code_id()?;
        let curve = self.curve.upload()?.uploaded_code_id()?;

        println!("bs721 code-id: {}", _bs721base);
        println!("launchpad code-id: {}", _launchpad);
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
            Box::new(&mut self.launchparty),
            Box::new(&mut self.base),
            Box::new(&mut self.curve),
            Box::new(&mut self.factory),
            Box::new(&mut self.royalties),
        ]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let suite = Self::new(chain.clone());
        Ok(suite)
    }

    fn deploy_on(chain: Chain, data: Self::DeployData) -> Result<Self, Self::Error> {
        // ########### Upload ##############
        let mut suite: BtsgNftSuite<Chain> = BtsgNftSuite::store_on(chain.clone())?;
        let start_time = chain
            .node_querier()
            .latest_block()
            .map_err(|e| StdError::generic_err(e.to_string()))?
            .time
            .plus_seconds(60u64);

        // LAUNCHPARTY
        let res = suite.launchparty.instantiate(
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
                bs721_code_id: suite.base.code_id()?,
                bs721_admin: chain.sender_addr().to_string(),
            },
            None,
            &[],
        )?;

        let bs721 = res.event_attr_value("wasm", "bs721_address")?;
        println!(" {:#?}!", bs721);
        suite.base.set_address(&Addr::unchecked(bs721));

        let binding = chain.clone().state().get_all_addresses()?;
        let res: Vec<(&String, &Addr)> = binding.iter().map(|a| a).collect();
        println!(" {:#?}!", res);

        Ok(suite)
    }
}

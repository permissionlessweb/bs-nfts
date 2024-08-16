use crate::Bs721CollectionSuite;
use cw_orch::prelude::*;

// Bitsong NFT Suite
impl<Chain: CwEnv> cw_orch::contract::Deploy<Chain> for Bs721CollectionSuite<Chain> {
    // We don't have a custom error type
    type Error = CwOrchError;
    type DeployData = Addr;

    fn store_on(chain: Chain) -> Result<Self, Self::Error> {
        let suite = Bs721CollectionSuite::new(chain.clone());
        suite.upload()?;
        Ok(suite)
    }

    fn deployed_state_file_path() -> Option<String> {
        None
    }

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
        vec![
            Box::new(&mut self.account),
        ]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let suite = Self::new(chain.clone());
        Ok(suite)
    }

    fn deploy_on(chain: Chain, _data: Self::DeployData) -> Result<Self, Self::Error> {
        // ########### Upload ##############
        let suite: Bs721CollectionSuite<Chain> = Bs721CollectionSuite::store_on(chain.clone())?;
        Ok(suite)
    }
}

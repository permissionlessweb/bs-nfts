use btsg_account::Metadata;
use btsg_cw_orch::*;
use cw_orch::prelude::*;

// cw-funds-distributor
pub struct Bs721CollectionSuite<Chain> {
    pub account: BitsongAccountCollection<Chain, Metadata>,
}

impl<Chain: CwEnv> Bs721CollectionSuite<Chain> {
    pub fn new(chain: Chain) -> Bs721CollectionSuite<Chain> {
        Bs721CollectionSuite::<Chain> {
            account: BitsongAccountCollection::new("bs721_account", chain.clone()),
        }
    }

    pub fn upload(&self) -> Result<(), CwOrchError> {
        self.account.upload()?;
        Ok(())
    }
}

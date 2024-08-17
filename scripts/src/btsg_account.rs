use btsg_account::Metadata;
use btsg_cw_orch::*;
use cw_orch::prelude::*;

// Bitsong Accounts Collection Framework Testing Suite.
pub struct BtsgAccountTestSuite<Chain> {
    pub account: BitsongAccountCollection<Chain, Metadata>,
    pub market: BitsongAccountMarketplace<Chain>,
    pub minter: BitsongAccountMinter<Chain>,
}

impl<Chain: CwEnv> BtsgAccountTestSuite<Chain> {
    pub fn new(chain: Chain) -> BtsgAccountTestSuite<Chain> {
        BtsgAccountTestSuite::<Chain> {
            account: BitsongAccountCollection::new("bs721_account", chain.clone()),
            market: BitsongAccountMarketplace::new("bs721_account_market", chain.clone()),
            minter: BitsongAccountMinter::new("bs721_account_minter", chain.clone()),
        }
    }

    pub fn upload(&self) -> Result<(), CwOrchError> {
        self.account.upload()?;
        self.market.upload()?;
        self.minter.upload()?;
        Ok(())
    }
}

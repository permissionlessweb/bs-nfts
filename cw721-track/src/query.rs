use cosmwasm_std::{Binary, Deps, Empty, Env};
use cw721_base::QueryMsg;

use crate::{error::ContractError, state::Cw721TrackContract};

impl<'a> Cw721TrackContract<'a> {
    pub fn query(
        &self,
        deps: Deps,
        env: Env,
        msg: QueryMsg<Empty>,
    ) -> Result<Binary, ContractError> {
        Ok(self.cw721_contract.query(deps, env, msg)?)
    }
}

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw721::Cw721Execute;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg},
    state::{Config, Cw721TrackContract},
    Extension,
};
use cw721_base::InstantiateMsg as Cw721InstantiateMsg;

impl<'a> Cw721TrackContract<'a> {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        let config = Config {
            next_token_id: 1,
            seller_fee_bps: Some(0),
            royalties_address: None,
        };
        self.config.save(deps.storage, &config)?;

        Ok(self.cw721_contract.instantiate(
            deps,
            env,
            info,
            Cw721InstantiateMsg {
                name: msg.name,
                symbol: msg.symbol,
                minter: msg.minter,
            },
        )?)
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<Extension>,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Mint(msg) => self.mint(
                deps,
                env,
                info,
                msg.token_id,
                msg.owner,
                msg.token_uri,
                msg.extension,
            ),

            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => {
                let res = self
                    .cw721_contract
                    .approve(deps, env, info, spender, token_id, expires)?;
                Ok(res)
            }

            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => {
                let res = self
                    .cw721_contract
                    .transfer_nft(deps, env, info, recipient, token_id)?;
                Ok(res)
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                let res = self
                    .cw721_contract
                    .approve_all(deps, env, info, operator, expires)?;
                Ok(res)
            }
            ExecuteMsg::Revoke { spender, token_id } => {
                let res = self
                    .cw721_contract
                    .revoke(deps, env, info, spender, token_id)?;
                Ok(res)
            }
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => {
                let res = self
                    .cw721_contract
                    .send_nft(deps, env, info, contract, token_id, msg)?;
                Ok(res)
            }
            ExecuteMsg::RevokeAll { operator } => {
                let res = self.cw721_contract.revoke_all(deps, env, info, operator)?;
                Ok(res)
            }
            ExecuteMsg::Burn { token_id } => {
                let res = self.cw721_contract.burn(deps, env, info, token_id)?;
                Ok(res)
            }
        }
    }
}

impl<'a> Cw721TrackContract<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn mint(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        token_id: String,
        owner: String,
        token_uri: Option<String>,
        extension: Extension,
    ) -> Result<Response, ContractError> {
        let res = self
            .cw721_contract
            .mint(deps, info, token_id, owner, token_uri, extension)?;
        Ok(res.add_attribute("track", "yes"))
    }
}

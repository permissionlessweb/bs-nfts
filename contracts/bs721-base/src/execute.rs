use serde::de::DeserializeOwned;
use serde::Serialize;

use cosmwasm_std::{
    Binary, CustomMsg, Decimal, Deps, DepsMut, Env, Event, MessageInfo, Response,
};

use bs721::{
    Bs721Execute, Bs721ReceiveMsg, CollectionInfo, Expiration, RoyaltyInfo,
    RoyaltyInfoResponse, UpdateCollectionInfoMsg,
};
use cw721::ContractInfoResponse as CW721ContractInfoResponse;
use cw_utils::maybe_addr;
use url::Url;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MintMsg};
use crate::state::{self, Approval, Bs721Contract, TokenInfo};

const MAX_SELLER_FEE: u16 = 10000; // mean 100%
const MAX_DESCRIPTION_LENGTH: u32 = 512;
const MAX_SHARE_DELTA_PCT: u64 = 2;
const MAX_ROYALTY_SHARE_PCT: u64 = 10;

impl<'a, T, C, E, Q> Bs721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        // no funds should be sent to this contract
        // nonpayable(&info)?;

        //set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let info = CW721ContractInfoResponse {
            name: msg.name,
            symbol: msg.symbol,
        };
        self.parent.contract_info.save(deps.storage, &info)?;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.minter))?;

        let minter = deps.api.addr_validate(&msg.minter)?;
        self.minter.save(deps.storage, &minter)?;

        // bs721 instantiation
        if msg.collection_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
            return Err(ContractError::DescriptionTooLong {});
        }
        let image = Url::parse(&msg.collection_info.image)?;

        if let Some(ref external_link) = msg.collection_info.external_link {
            Url::parse(external_link)?;
        }

        let royalty_info: Option<RoyaltyInfo> = match msg.collection_info.royalty_info {
            Some(royalty_info) => Some(RoyaltyInfo {
                payment_address: deps.api.addr_validate(&royalty_info.payment_address)?,
                share: state::Bs721Contract::<'a, T, C, E, Q>::share_validate(royalty_info.share)?,
                payment_denom: royalty_info.payment_denom,
            }),
            None => None,
        };

        deps.api.addr_validate(&msg.collection_info.creator)?;

        let collection_info = CollectionInfo {
            creator: msg.collection_info.creator,
            description: msg.collection_info.description,
            image: msg.collection_info.image,
            external_link: msg.collection_info.external_link,
            explicit_content: msg.collection_info.explicit_content,
            start_trading_time: msg.collection_info.start_trading_time,
            royalty_info,
        };

        self.collection_info.save(deps.storage, &collection_info)?;

        self.royalty_updated_at
            .save(deps.storage, &env.block.time)?;

        Ok(Response::default()
            .add_attribute("action", "instantiate")
            .add_attribute("collection_name", info.name)
            .add_attribute("collection_symbol", info.symbol)
            .add_attribute("collection_creator", collection_info.creator)
            .add_attribute("minter", msg.minter)
            .add_attribute("image", image.to_string()))
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<T, E>,
    ) -> Result<Response<C>, ContractError> {
        match msg {
            ExecuteMsg::SetMinter { new_minter } => self.set_minter(deps, env, info, new_minter),
            ExecuteMsg::Mint(msg) => self.mint(deps, env, info, msg),
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => self.approve(deps, env, info, spender, token_id, expires),
            ExecuteMsg::Revoke { spender, token_id } => {
                self.revoke(deps, env, info, spender, token_id)
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                self.approve_all(deps, env, info, operator, expires)
            }
            ExecuteMsg::RevokeAll { operator } => self.revoke_all(deps, env, info, operator),
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => self.transfer_nft(deps, env, info, recipient, token_id),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => self.send_nft(deps, env, info, contract, token_id, msg),
            ExecuteMsg::Burn { token_id } => self.burn(deps, env, info, token_id),
            ExecuteMsg::Extension { msg: _ } => Ok(Response::default()),
            ExecuteMsg::UpdateCollectionInfo {
                new_collection_info,
            } =>self.update_collection_info(deps, env, info, new_collection_info)
        }
    }
}

// TODO pull this into some sort of trait extension??
impl<'a, T, C, E, Q> Bs721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn update_collection_info(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        new_collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    ) -> Result<Response<C>, ContractError> {
        let mut collection = self.collection_info.load(deps.storage)?;

        // if self.frozen_collection_info.load(deps.storage)? {
        //     return Err(ContractError::CollectionInfoFrozen {});
        // }

        // only creator can update collection info
        if collection.creator != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(new_creator) = new_collection_info.creator {
            deps.api.addr_validate(&new_creator)?;
            collection.creator = new_creator;
        }

        collection.description = new_collection_info
            .description
            .unwrap_or_else(|| collection.description.to_string());
        if collection.description.len() > MAX_DESCRIPTION_LENGTH as usize {
            return Err(ContractError::DescriptionTooLong {});
        }

        collection.image = new_collection_info
            .image
            .unwrap_or_else(|| collection.image.to_string());
        Url::parse(&collection.image)?;

        collection.external_link = new_collection_info
            .external_link
            .unwrap_or_else(|| collection.external_link.as_ref().map(|s| s.to_string()));
        if collection.external_link.as_ref().is_some() {
            Url::parse(collection.external_link.as_ref().unwrap())?;
        }

        collection.explicit_content = new_collection_info.explicit_content;

        if let Some(Some(new_royalty_info_response)) = new_collection_info.royalty_info {
            let last_royalty_update = self.royalty_updated_at.load(deps.storage)?;
            if last_royalty_update.plus_seconds(24 * 60 * 60) > env.block.time {
                return Err(ContractError::InvalidRoyalties(
                    "Royalties can only be updated once per day".to_string(),
                ));
            }

            let new_royalty_info = RoyaltyInfo {
                payment_address: deps
                    .api
                    .addr_validate(&new_royalty_info_response.payment_address)?,
                share: state::Bs721Contract::<'a, T, C, E, Q>::share_validate(
                    new_royalty_info_response.share,
                )?,
                payment_denom: new_royalty_info_response.payment_denom,
            };

            if let Some(old_royalty_info) = collection.royalty_info {
                if old_royalty_info.share < new_royalty_info.share {
                    let share_delta = new_royalty_info.share.abs_diff(old_royalty_info.share);

                    if share_delta > Decimal::percent(MAX_SHARE_DELTA_PCT) {
                        return Err(ContractError::InvalidRoyalties(format!(
                            "Share increase cannot be greater than {MAX_SHARE_DELTA_PCT}%"
                        )));
                    }
                    if new_royalty_info.share > Decimal::percent(MAX_ROYALTY_SHARE_PCT) {
                        return Err(ContractError::InvalidRoyalties(format!(
                            "Share cannot be greater than {MAX_ROYALTY_SHARE_PCT}%"
                        )));
                    }
                }
            }

            collection.royalty_info = Some(new_royalty_info);
            self.royalty_updated_at
                .save(deps.storage, &env.block.time)?;
        }

        self.collection_info.save(deps.storage, &collection)?;

        let event = Event::new("update_collection_info").add_attribute("sender", info.sender);
        Ok(Response::new().add_event(event))
    }

    pub fn set_minter(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        new_minter: String,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;
        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }

        let new_minter_addr = deps.api.addr_validate(&new_minter)?;
        self.minter.save(deps.storage, &new_minter_addr)?;

        Ok(Response::new()
            .add_attribute("action", "set_minter")
            .add_attribute("new_minter", new_minter))
    }

    pub fn mint(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: MintMsg<T>,
    ) -> Result<Response<C>, ContractError> {
        let minter = self.minter.load(deps.storage)?;

        if info.sender != minter {
            return Err(ContractError::Unauthorized {});
        }

        // seller fee and payment address are optional, if one is set, both must be set
        if (msg.seller_fee_bps.is_some() && msg.payment_addr.is_none())
            || (msg.seller_fee_bps.is_none() && msg.payment_addr.is_some())
        {
            return Err(ContractError::InvalidSellerFee {});
        }

        // seller fee must be between 0 and 100%
        if let Some(fee) = msg.seller_fee_bps {
            if fee > MAX_SELLER_FEE {
                return Err(ContractError::MaxSellerFeeExceeded {});
            }
        }

        // create the token
        let token = TokenInfo {
            owner: deps.api.addr_validate(&msg.owner)?,
            approvals: vec![],
            token_uri: msg.token_uri,
            extension: msg.extension,
            seller_fee_bps: msg.seller_fee_bps,
            payment_addr: maybe_addr(deps.api, msg.payment_addr)?,
        };
        self.tokens
            .update(deps.storage, &msg.token_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(token),
            })?;

        self.increment_tokens(deps.storage)?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", info.sender)
            .add_attribute("owner", msg.owner)
            .add_attribute("token_id", msg.token_id))
    }
}

impl<'a, T, C, E, Q> Bs721Execute<T, C> for Bs721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    type Err = ContractError;

    fn transfer_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

        Ok(Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", recipient)
            .add_attribute("token_id", token_id))
    }

    fn send_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response<C>, ContractError> {
        // Transfer token
        self._transfer_nft(deps, &env, &info, &contract, &token_id)?;

        let send = Bs721ReceiveMsg {
            sender: info.sender.to_string(),
            token_id: token_id.clone(),
            msg,
        };

        // Send message
        Ok(Response::new()
            .add_message(send.into_cosmos_msg(contract.clone())?)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", contract)
            .add_attribute("token_id", token_id))
    }

    fn approve(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn revoke(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

        Ok(Response::new()
            .add_attribute("action", "revoke")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn approve_all(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        operator: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        // set the operator for us
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }

    fn revoke_all(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: String,
    ) -> Result<Response<C>, ContractError> {
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .remove(deps.storage, (&info.sender, &operator_addr));

        Ok(Response::new()
            .add_attribute("action", "revoke_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }

    fn burn(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        let token = self.tokens.load(deps.storage, &token_id)?;
        self.check_can_send(deps.as_ref(), &env, &info, &token)?;

        self.tokens.remove(deps.storage, &token_id)?;
        self.decrement_tokens(deps.storage)?;

        Ok(Response::new()
            .add_attribute("action", "burn")
            .add_attribute("sender", info.sender)
            .add_attribute("token_id", token_id))
    }
}

// helpers
impl<'a, T, C, E, Q> Bs721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn _transfer_nft(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        recipient: &str,
        token_id: &str,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        // ensure we have permissions
        self.check_can_send(deps.as_ref(), env, info, &token)?;
        // set owner and remove existing approvals
        token.owner = deps.api.addr_validate(recipient)?;
        token.approvals = vec![];
        self.tokens.save(deps.storage, token_id, &token)?;
        Ok(token)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn _update_approvals(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        spender: &str,
        token_id: &str,
        // if add == false, remove. if add == true, remove then set with this expiration
        add: bool,
        expires: Option<Expiration>,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        // ensure we have permissions
        self.check_can_approve(deps.as_ref(), env, info, &token)?;

        // update the approval list (remove any for the same spender before adding)
        let spender_addr = deps.api.addr_validate(spender)?;
        token.approvals.retain(|apr| apr.spender != spender_addr);

        // only difference between approve and revoke
        if add {
            // reject expired data as invalid
            let expires = expires.unwrap_or_default();
            if expires.is_expired(&env.block) {
                return Err(ContractError::Expired {});
            }
            let approval = Approval {
                spender: spender_addr,
                expires,
            };
            token.approvals.push(approval);
        }

        self.tokens.save(deps.storage, token_id, &token)?;

        Ok(token)
    }

    /// returns true iff the sender can execute approve or reject on the contract
    pub fn check_can_approve(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        // owner can approve
        if token.owner == info.sender {
            return Ok(());
        }
        // operator can approve
        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::Unauthorized {})
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::Unauthorized {}),
        }
    }

    /// returns true iff the sender can transfer ownership of the token
    pub fn check_can_send(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        // owner can send
        if token.owner == info.sender {
            return Ok(());
        }

        // any non-expired token approval can send
        if token
            .approvals
            .iter()
            .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
        {
            return Ok(());
        }

        // operator can send
        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::Unauthorized {})
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::Unauthorized {}),
        }
    }
}

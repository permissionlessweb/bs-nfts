use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, self, PartyType};
use crate::state::{Config, CONFIG};

use bs721_base::{Extension, MintMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn,
    Response, StdResult, SubMsg, Uint128, WasmMsg, Timestamp,
};
use cw2::set_contract_version;

use bs721_base::msg::{ExecuteMsg as Bs721ExecuteMsg, InstantiateMsg as Bs721InstantiateMsg};
use cw4::Member;
use cw4_group::msg::InstantiateMsg as Cw4InstantiateMsg;
use cw_utils::{may_pay, parse_reply_instantiate_data};

const CONTRACT_NAME: &str = "crates.io:launchparty-fixed";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;
const INSTANTIATE_ROYALTY_REPLY_ID: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.party_type.validate()?;

    let config = Config {
        creator: deps.api.addr_validate(&msg.creator.unwrap_or_else(||info.sender.to_string()))?,
        name: msg.name.clone(),
        symbol: msg.symbol.clone(),
        base_token_uri: msg.base_token_uri.clone(),
        price: msg.price,
        bs721_address: None,
        next_token_id: 1,
        seller_fee_bps: msg.seller_fee_bps,
        royalty_address: None,
        start_time: msg.start_time,
        party_type: msg.party_type,
    };

    CONFIG.save(deps.storage, &config)?;

    // create submessages to instantiate token and royalties contracts
    let sub_msgs: Vec<SubMsg> = vec![
        SubMsg {
            id: INSTANTIATE_TOKEN_REPLY_ID,
            msg: WasmMsg::Instantiate {
                code_id: BS721_CODE_ID,
                msg: to_binary(&Bs721InstantiateMsg {
                    name: msg.name.clone(),
                    symbol: msg.symbol.clone(),
                    minter: env.contract.address.to_string(),
                    uri: Some(msg.collection_uri.to_string()),
                })?,
                label: "Launchparty fixed contract".to_string(),
                admin: None,
                funds: vec![],
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        },
        SubMsg {
            id: INSTANTIATE_ROYALTY_REPLY_ID,
            msg: WasmMsg::Instantiate {
                code_id: CW4_GROUP_CODE_ID,
                msg: to_binary(&Cw4InstantiateMsg {
                    admin: None,
                    members: msg
                        .contributors
                        .into_iter()
                        .map(|c| Member {
                            addr: c.addr,
                            weight: c.weight,
                        })
                        .collect(),
                })?,
                label: "Launchparty royalty contract".to_string(),
                admin: None,
                funds: vec![],
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        },
    ];

    Ok(Response::new().add_submessages(sub_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    let mut res = Response::new();

    let reply_res = parse_reply_instantiate_data(reply.clone()).unwrap();

    match reply.id {
        INSTANTIATE_TOKEN_REPLY_ID => {
            if config.bs721_address.is_some() {
                return Err(ContractError::Bs721AlreadyLinked {});
            }

            config.bs721_address = Addr::unchecked(reply_res.contract_address.clone()).into();

            res = res
                .add_attribute("action", "bs721_reply")
                .add_attribute("contract_address", reply_res.contract_address)
        }
        INSTANTIATE_ROYALTY_REPLY_ID => {
            if config.royalty_address.is_some() {
                return Err(ContractError::RoyaltyAlreadyLinked {});
            }

            config.royalty_address = Addr::unchecked(reply_res.contract_address.clone()).into();

            res = res
                .add_attribute("action", "royalty_reply")
                .add_attribute("contract_address", reply_res.contract_address)
        }
        _ => return Err(ContractError::UnknownReplyId {}),
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {} => execute_mint(deps, env, info),
    }
}

fn execute_mint(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    if config.start_time > env.block.time {
        return Err(ContractError::NotStarted {});
    }

    if config.bs721_address.is_none() {
        return Err(ContractError::Bs721NotLinked {});
    }

    if !party_is_active(&env, &config) {
        return Err(ContractError::PartyEnded {})
    }

    let payment = may_pay(&info, &"ubtsg")?;
    if payment != config.price {
        return Err(ContractError::InvalidPaymentAmount(
            coin(payment.u128(), "ubtsg"),
            coin(config.price.u128(), "ubtsg"),
        ));
    }

    let mut res = Response::new();

    let mint_msg = Bs721ExecuteMsg::<Extension, Empty>::Mint(MintMsg::<Extension> {
        owner: info.sender.to_string(),
        token_id: config.next_token_id.to_string(),
        token_uri: Some(format!(
            "{}/{}.json",
            config.base_token_uri.to_string(),
            config.next_token_id.to_string()
        )),
        extension: None,
        payment_addr: Some(config.royalty_address.clone().unwrap().to_string()),
        seller_fee_bps: Some(config.seller_fee_bps),
    });

    let msg = WasmMsg::Execute {
        contract_addr: config.bs721_address.clone().unwrap().to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    };

    res = res.add_message(msg);

    if config.price > Uint128::zero() {
        let bank_msg = BankMsg::Send {
            to_address: config.royalty_address.clone().unwrap().to_string(),
            amount: vec![coin(config.price.u128(), "ubtsg")],
        };
        res = res.add_message(bank_msg);
    }

    config.next_token_id += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(res
        .add_attribute("action", "nft_minted")
        .add_attribute("token_id", config.next_token_id.to_string())
        .add_attribute("price", config.price.to_string())
        .add_attribute("creator", config.creator.to_string())
        .add_attribute("recipient", info.sender.to_string()))
}

/// Returns true if the launcharty is still active, false otherwise.
pub fn party_is_active(env: &Env, config: &Config) -> bool {
    match config.party_type {
        PartyType::MaxEdition(number) => {
            if config.next_token_id - 1 >= number {
                return false
            }
        }
        PartyType::EndTime(duration) => {
            if config.start_time.plus_seconds(duration as u64) < env.block.time {
                return false
            }
        }
    }
    true
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        creator: config.creator,
        bs721_address: config.bs721_address,
        price: config.price,
        name: config.name,
        symbol: config.symbol,
        base_token_uri: config.base_token_uri,
        next_token_id: config.next_token_id,
        seller_fee_bps: config.seller_fee_bps,
        royalty_address: config.royalty_address,
        start_time: config.start_time,
        party_type: config.party_type,
    })
}

// -------------------------------------------------------------------------------------------------
// Unit test
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::msg::Contributor;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{from_binary, to_binary, SubMsgResponse, SubMsgResult, Timestamp};
    use prost::Message;

    const NFT_CONTRACT_ADDR: &str = "nftcontract";
    const ROYALTY_CONTRACT_ADDR: &str = "royaltycontract";

    // Type for replies to contract instantiate messes
    #[derive(Clone, PartialEq, Message)]
    struct MsgInstantiateContractResponse {
        #[prost(string, tag = "1")]
        pub contract_address: ::prost::alloc::string::String,
        #[prost(bytes, tag = "2")]
        pub data: ::prost::alloc::vec::Vec<u8>,
    }

    #[test]
    fn initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            name: "Launchparty".to_string(),
            price: Uint128::new(1),
            creator: Some(String::from("creator")),
            symbol: "LP".to_string(),
            base_token_uri: "ipfs://Qm......".to_string(),
            collection_uri: "ipfs://Qm......".to_string(),
            seller_fee_bps: 100,
            contributors: vec![Contributor {
                addr: "contributor".to_string(),
                weight: 100,
            }],
            start_time: Timestamp::from_seconds(0),
            party_type: PartyType::MaxEdition(1)
        };

        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        assert_eq!(
            res.messages,
            vec![
                SubMsg {
                    msg: WasmMsg::Instantiate {
                        code_id: BS721_CODE_ID,
                        msg: to_binary(&Bs721InstantiateMsg {
                            name: msg.name.clone(),
                            symbol: msg.symbol.clone(),
                            minter: MOCK_CONTRACT_ADDR.to_string(),
                            uri: Some(msg.collection_uri.to_string()),
                        })
                        .unwrap(),
                        funds: vec![],
                        admin: None,
                        label: String::from("Launchparty fixed contract"),
                    }
                    .into(),
                    id: INSTANTIATE_TOKEN_REPLY_ID,
                    gas_limit: None,
                    reply_on: ReplyOn::Success,
                },
                SubMsg {
                    msg: WasmMsg::Instantiate {
                        code_id: CW4_GROUP_CODE_ID,
                        msg: to_binary(&Cw4InstantiateMsg {
                            admin: None,
                            members: msg
                                .contributors
                                .into_iter()
                                .map(|c| Member {
                                    addr: c.addr,
                                    weight: c.weight,
                                })
                                .collect(),
                        })
                        .unwrap(),
                        label: "Launchparty royalty contract".to_string(),
                        admin: None,
                        funds: vec![],
                    }
                    .into(),
                    id: INSTANTIATE_ROYALTY_REPLY_ID,
                    gas_limit: None,
                    reply_on: ReplyOn::Success,
                },
            ]
        );

        let instantiate_reply_bs721 = MsgInstantiateContractResponse {
            contract_address: NFT_CONTRACT_ADDR.to_string(),
            data: vec![2u8; 32769],
        };

        let mut encoded_instantiate_reply_bs721 =
            Vec::<u8>::with_capacity(instantiate_reply_bs721.encoded_len());
        instantiate_reply_bs721
            .encode(&mut encoded_instantiate_reply_bs721)
            .unwrap();

        let reply_msg_bs721 = Reply {
            id: INSTANTIATE_TOKEN_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(encoded_instantiate_reply_bs721.into()),
            }),
        };

        reply(deps.as_mut(), mock_env(), reply_msg_bs721).unwrap();

        let instantiate_reply_royalty = MsgInstantiateContractResponse {
            contract_address: ROYALTY_CONTRACT_ADDR.to_string(),
            data: vec![2u8; 32769],
        };

        let mut encoded_instantiate_reply_royalty =
            Vec::<u8>::with_capacity(instantiate_reply_royalty.encoded_len());
        instantiate_reply_royalty
            .encode(&mut encoded_instantiate_reply_royalty)
            .unwrap();

        let reply_msg_royalty = Reply {
            id: INSTANTIATE_ROYALTY_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(encoded_instantiate_reply_royalty.into()),
            }),
        };

        reply(deps.as_mut(), mock_env(), reply_msg_royalty).unwrap();

        let query_msg = QueryMsg::GetConfig {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: Config = from_binary(&res).unwrap();

        assert_eq!(
            config,
            Config {
                creator: Addr::unchecked("creator"),
                name: "Launchparty".to_string(),
                symbol: "LP".to_string(),
                base_token_uri: "ipfs://Qm......".to_string(),
                price: Uint128::new(1),
                bs721_address: Some(Addr::unchecked(NFT_CONTRACT_ADDR)),
                next_token_id: 1,
                seller_fee_bps: 100,
                royalty_address: Some(Addr::unchecked(ROYALTY_CONTRACT_ADDR)),
                start_time: Timestamp::from_nanos(0),
                party_type: PartyType::MaxEdition(1)
            }
        );
    }

    #[test]
    fn mint() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            name: "Launchparty".to_string(),
            price: Uint128::new(1),
            creator: Some(String::from("creator")),
            symbol: "LP".to_string(),
            base_token_uri: "ipfs://Qm......".to_string(),
            collection_uri: "ipfs://Qm......".to_string(),
            seller_fee_bps: 100,
            contributors: vec![Contributor {
                addr: "contributor".to_string(),
                weight: 100,
            }],
            start_time: Timestamp::from_nanos(0),
            party_type: PartyType::MaxEdition(1),
        };

        let info = mock_info("creator", &[coin(1, "ubtsg")]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        let instantiate_reply_bs721 = MsgInstantiateContractResponse {
            contract_address: NFT_CONTRACT_ADDR.to_string(),
            data: vec![2u8; 32769],
        };

        let mut encoded_instantiate_reply_bs721 =
            Vec::<u8>::with_capacity(instantiate_reply_bs721.encoded_len());
        instantiate_reply_bs721
            .encode(&mut encoded_instantiate_reply_bs721)
            .unwrap();

        let reply_msg_bs721 = Reply {
            id: INSTANTIATE_TOKEN_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(encoded_instantiate_reply_bs721.into()),
            }),
        };

        reply(deps.as_mut(), mock_env(), reply_msg_bs721).unwrap();

        let instantiate_reply_royalty = MsgInstantiateContractResponse {
            contract_address: ROYALTY_CONTRACT_ADDR.to_string(),
            data: vec![2u8; 32769],
        };

        let mut encoded_instantiate_reply_royalty =
            Vec::<u8>::with_capacity(instantiate_reply_royalty.encoded_len());
        instantiate_reply_royalty
            .encode(&mut encoded_instantiate_reply_royalty)
            .unwrap();

        let reply_msg_royalty = Reply {
            id: INSTANTIATE_ROYALTY_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(encoded_instantiate_reply_royalty.into()),
            }),
        };

        reply(deps.as_mut(), mock_env(), reply_msg_royalty).unwrap();

        let msg = ExecuteMsg::Mint {};
        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(1, "ubtsg")]);

        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        let mint_msg = Bs721ExecuteMsg::<Extension, Empty>::Mint(MintMsg::<Extension> {
            token_id: "1".to_string(),
            extension: None,
            owner: info.sender.to_string(),
            payment_addr: Some(ROYALTY_CONTRACT_ADDR.to_string()),
            seller_fee_bps: Some(100),
            token_uri: Some("ipfs://Qm....../1.json".to_string()),
        });

        assert_eq!(
            res.messages[0],
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: NFT_CONTRACT_ADDR.to_string(),
                    funds: vec![],
                    msg: to_binary(&mint_msg).unwrap(),
                }
                .into(),
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
            }
        );
    }
}

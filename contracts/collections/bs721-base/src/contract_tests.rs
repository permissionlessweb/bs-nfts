#![cfg(test)]

use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{from_json, to_json_binary, Addr, CosmosMsg, DepsMut, Empty, Response, WasmMsg};

use bs721::{
    Approval, ApprovalResponse, Bs721Query, Bs721ReceiveMsg, ContractInfoResponse, Expiration,
    NftInfoResponse, OperatorsResponse, OwnerOfResponse,
};

use crate::{Bs721Contract, ContractError, ExecuteMsg, Extension, InstantiateMsg, QueryMsg};

const CONTRACT_NAME: &str = "Magic Power";
const SYMBOL: &str = "MGK";
const URI: &str = "";

fn setup_contract(
    deps: DepsMut<'_>,
    creator: Addr,
    minter: Addr,
) -> Bs721Contract<'static, Extension, Empty, Empty, Empty> {
    let contract = Bs721Contract::default();

    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        uri: Some(URI.to_string()),
        minter: minter.to_string(),
    };
    let info = message_info(&creator, &[]);
    let res = contract.instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    contract
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies();
    let contract = Bs721Contract::<Extension, Empty, Empty, Empty>::default();
    let creator = deps.api.addr_make("creator");
    let minter = deps.api.addr_make("minter");
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        uri: Some(URI.to_string()),
        minter: minter.to_string(),
    };
    let info = message_info(&creator.clone(), &[]);

    // we can just call .unwrap() to assert this was a success
    let res = contract
        .instantiate(deps.as_mut(), mock_env(), info, msg)
        .unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = contract.minter(deps.as_ref()).unwrap();
    assert_eq!(minter.to_string(), res.minter);
    let info = contract.contract_info(deps.as_ref()).unwrap();
    assert_eq!(
        info,
        ContractInfoResponse {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
            uri: Some(URI.to_string()),
        }
    );

    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(0, count.count);
}

#[test]
fn minting() {
    let mut deps = mock_dependencies();
    let creator = deps.api.addr_make("creator");
    let minter = deps.api.addr_make("minter");
    let medusa = deps.api.addr_make("medusa");
    let hercules = deps.api.addr_make("hercules");
    let paymemt = deps.api.addr_make("paymemt");

    let contract = setup_contract(deps.as_mut(), creator.clone(), minter.clone());

    let token_id = "petrify".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: medusa.to_string(),
        token_uri: Some(token_uri.clone()),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(paymemt.to_string()),
        extension: None,
    };

    // random cannot mint
    let random = message_info(&creator, &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // minter can mint
    let allowed = message_info(&minter, &[]);
    let _ = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg)
        .unwrap();

    // ensure num tokens increases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(1, count.count);

    // unknown nft returns error
    let _ = contract
        .nft_info(deps.as_ref(), "unknown".to_string())
        .unwrap_err();

    // this nft info is correct
    let info = contract.nft_info(deps.as_ref(), token_id.clone()).unwrap();
    assert_eq!(
        info,
        NftInfoResponse::<Extension> {
            token_uri: Some(token_uri),
            seller_fee_bps: Option::from(100u16),
            payment_addr: Option::from(paymemt.clone()),
            extension: None,
        }
    );

    // owner info is correct
    let owner = contract
        .owner_of(deps.as_ref(), mock_env(), token_id.clone(), true)
        .unwrap();
    assert_eq!(
        owner,
        OwnerOfResponse {
            owner: medusa.to_string(),
            approvals: vec![],
        }
    );

    // Cannot mint same token_id again
    let mint_msg2 = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: hercules.to_string(),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(paymemt.to_string()),
        token_uri: None,
        extension: None,
    };

    let allowed = message_info(&minter, &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg2)
        .unwrap_err();
    assert_eq!(err, ContractError::Claimed {});

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id], tokens.tokens);
}

#[test]
fn burning() {
    let mut deps = mock_dependencies();
    let creator = deps.api.addr_make("creator");
    let minter = deps.api.addr_make("minter");
    let random = deps.api.addr_make("random");
    let paymemt = deps.api.addr_make("paymemt");

    let contract = setup_contract(deps.as_mut(), creator.clone(), minter.clone());

    let token_id = "petrify".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: minter.to_string(),
        token_uri: Some(token_uri),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(paymemt.to_string()),
        extension: None,
    };

    let burn_msg = ExecuteMsg::Burn { token_id };

    // mint some NFT
    let allowed = message_info(&minter, &[]);
    let _ = contract
        .execute(deps.as_mut(), mock_env(), allowed.clone(), mint_msg)
        .unwrap();

    // random not allowed to burn
    let random_msg = message_info(&random, &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random_msg, burn_msg.clone())
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});

    let _ = contract
        .execute(deps.as_mut(), mock_env(), allowed, burn_msg)
        .unwrap();

    // ensure num tokens decreases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(0, count.count);

    // trying to get nft returns error
    let _ = contract
        .nft_info(deps.as_ref(), "petrify".to_string())
        .unwrap_err();

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert!(tokens.tokens.is_empty());
}

#[test]
fn transferring_nft() {
    let mut deps = mock_dependencies();
    let creator = deps.api.addr_make("creator");
    let minter = deps.api.addr_make("minter");
    let random = deps.api.addr_make("random");
    let venus = deps.api.addr_make("venus");
    let paymment = deps.api.addr_make("paymment");

    let contract = setup_contract(deps.as_mut(), creator.clone(), minter.clone());

    // Mint a token
    let token_id = "melt".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/melt".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: venus.to_string(),
        token_uri: Some(token_uri),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(paymment.to_string()),
        extension: None,
    };

    let minter = message_info(&minter, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // random cannot transfer
    let random_msg = message_info(&random, &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: random.to_string(),
        token_id: token_id.clone(),
    };

    let err = contract
        .execute(deps.as_mut(), mock_env(), random_msg, transfer_msg)
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // owner can
    let random_msg = message_info(&venus, &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: random.to_string(),
        token_id: token_id.clone(),
    };

    let res = contract
        .execute(deps.as_mut(), mock_env(), random_msg, transfer_msg)
        .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", venus.to_string())
            .add_attribute("recipient", random.to_string())
            .add_attribute("token_id", token_id)
    );
}

#[test]
fn sending_nft() {
    let mut deps = mock_dependencies();
    let creator = deps.api.addr_make("creator");
    let minter = deps.api.addr_make("minter");
    let random = deps.api.addr_make("random");
    let venus = deps.api.addr_make("venus");
    let mars = deps.api.addr_make("mars");
    let payment = deps.api.addr_make("payment");
    let contract = setup_contract(deps.as_mut(), creator.clone(), minter.clone());

    // Mint a token
    let token_id = "melt".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/melt".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: venus.to_string(),
        token_uri: Some(token_uri),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(payment.to_string()),
        extension: None,
    };

    let minter = message_info(&minter, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    let msg = to_json_binary("You now have the melting power").unwrap();
    let target = mars.to_string();
    let send_msg = ExecuteMsg::SendNft {
        contract: target.clone(),
        token_id: token_id.clone(),
        msg: msg.clone(),
    };

    let random = message_info(&random, &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, send_msg.clone())
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // but owner can
    let random = message_info(&venus, &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), random, send_msg)
        .unwrap();

    let payload = Bs721ReceiveMsg {
        sender: venus.to_string(),
        token_id: token_id.clone(),
        msg,
    };
    let expected = payload.into_cosmos_msg(target.clone()).unwrap();
    // ensure expected serializes as we think it should
    match &expected {
        CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, .. }) => {
            assert_eq!(contract_addr, &target)
        }
        m => panic!("Unexpected message type: {:?}", m),
    }
    // and make sure this is the request sent by the contract
    assert_eq!(
        res,
        Response::new()
            .add_message(expected)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", venus.to_string())
            .add_attribute("recipient", mars.to_string())
            .add_attribute("token_id", token_id)
    );
}

#[test]
fn approving_revoking() {
    let mut deps = mock_dependencies();
    let creator = deps.api.addr_make("creator");
    let minter = deps.api.addr_make("minter");
    let random = deps.api.addr_make("random");
    let payment = deps.api.addr_make("payment");
    let demeter = deps.api.addr_make("demeter");
    let contract = setup_contract(deps.as_mut(), creator.clone(), minter.clone());

    // Mint a token
    let token_id = "grow".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/grow".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: demeter.to_string(),
        token_uri: Some(token_uri),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(payment.to_string()),
        extension: None,
    };

    let minter = message_info(&minter, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // token owner shows in approval query
    let res = contract
        .approval(
            deps.as_ref(),
            mock_env(),
            token_id.clone(),
            demeter.to_string(),
            false,
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovalResponse {
            approval: Approval {
                spender: demeter.to_string(),
                expires: Expiration::Never {}
            }
        }
    );

    // Give random transferring power
    let approve_msg = ExecuteMsg::Approve {
        spender: random.to_string(),
        token_id: token_id.clone(),
        expires: None,
    };
    let owner = message_info(&demeter, &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), owner, approve_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", demeter.to_string())
            .add_attribute("spender", random.to_string())
            .add_attribute("token_id", token_id.clone())
    );

    // test approval query
    let res = contract
        .approval(
            deps.as_ref(),
            mock_env(),
            token_id.clone(),
            random.to_string(),
            true,
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovalResponse {
            approval: Approval {
                spender: random.to_string(),
                expires: Expiration::Never {}
            }
        }
    );

    // random can now transfer
    let random_msg = message_info(&random, &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: demeter.to_string(),
        token_id: token_id.clone(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), random_msg, transfer_msg)
        .unwrap();

    // Approvals are removed / cleared
    let query_msg = QueryMsg::OwnerOf {
        token_id: token_id.clone(),
        include_expired: None,
    };
    let res: OwnerOfResponse = from_json(
        &contract
            .query(deps.as_ref(), mock_env(), query_msg.clone())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: demeter.to_string(),
            approvals: vec![],
        }
    );

    // Approve, revoke, and check for empty, to test revoke
    let approve_msg = ExecuteMsg::Approve {
        spender: random.to_string(),
        token_id: token_id.clone(),
        expires: None,
    };
    let owner = message_info(&demeter, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner.clone(), approve_msg)
        .unwrap();

    let revoke_msg = ExecuteMsg::Revoke {
        spender: random.to_string(),
        token_id,
    };
    contract
        .execute(deps.as_mut(), mock_env(), owner, revoke_msg)
        .unwrap();

    // Approvals are now removed / cleared
    let res: OwnerOfResponse = from_json(
        &contract
            .query(deps.as_ref(), mock_env(), query_msg)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: demeter.to_string(),
            approvals: vec![],
        }
    );
}

#[test]
fn approving_all_revoking_all() {
    let mut deps = mock_dependencies();
    let creator = deps.api.addr_make("creator");
    let minter = deps.api.addr_make("minter");
    let random = deps.api.addr_make("random");
    let person = deps.api.addr_make("person");
    let demeter = deps.api.addr_make("demeter");
    let operator = deps.api.addr_make("operator");
    let payment = deps.api.addr_make("payment");
    let randcontract = deps.api.addr_make("randcontract");
    let contract = setup_contract(deps.as_mut(), creator.clone(), minter.clone());

    // Mint a couple tokens (from the same owner)
    let token_id1 = "grow1".to_string();
    let token_uri1 = "https://www.merriam-webster.com/dictionary/grow1".to_string();

    let token_id2 = "grow2".to_string();
    let token_uri2 = "https://www.merriam-webster.com/dictionary/grow2".to_string();

    let mint_msg1 = ExecuteMsg::Mint {
        token_id: token_id1.clone(),
        owner: demeter.to_string(),
        token_uri: Some(token_uri1),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(payment.to_string()),
        extension: None,
    };

    let minter = message_info(&minter, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg1)
        .unwrap();

    let mint_msg2 = ExecuteMsg::Mint {
        token_id: token_id2.clone(),
        owner: demeter.to_string(),
        token_uri: Some(token_uri2),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(payment.to_string()),
        extension: None,
    };

    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg2)
        .unwrap();

    // paginate the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, Some(1)).unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id1.clone()], tokens.tokens);
    let tokens = contract
        .all_tokens(deps.as_ref(), Some(token_id1.clone()), Some(3))
        .unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id2.clone()], tokens.tokens);

    // demeter gives random full (operator) power over her tokens
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: random.to_string(),
        expires: None,
    };
    let owner_msg = message_info(&demeter, &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), owner_msg, approve_all_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", demeter.to_string())
            .add_attribute("operator", random.to_string())
    );

    // random can now transfer
    let random = message_info(&random, &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: person.to_string(),
        token_id: token_id1,
    };
    contract
        .execute(deps.as_mut(), mock_env(), random.clone(), transfer_msg)
        .unwrap();

    // random can now send
    let inner_msg = WasmMsg::Execute {
        contract_addr: randcontract.to_string(),
        msg: to_json_binary("You now also have the growing power").unwrap(),
        funds: vec![],
    };
    let msg: CosmosMsg = CosmosMsg::Wasm(inner_msg);

    let send_msg = ExecuteMsg::SendNft {
        contract: randcontract.to_string(),
        token_id: token_id2,
        msg: to_json_binary(&msg).unwrap(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), random, send_msg)
        .unwrap();

    // Approve_all, revoke_all, and check for empty, to test revoke_all
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: operator.to_string(),
        expires: None,
    };
    // person is now the owner of the tokens
    let owner = message_info(&person, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner, approve_all_msg)
        .unwrap();

    let res = contract
        .operators(
            deps.as_ref(),
            mock_env(),
            person.to_string(),
            true,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![bs721::Approval {
                spender: operator.to_string(),
                expires: Expiration::Never {}
            }]
        }
    );
    let buddy = deps.api.addr_make("buddy");
    // second approval
    let buddy_expires = Expiration::AtHeight(1234567);
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: buddy.to_string(),
        expires: Some(buddy_expires),
    };
    let owner = message_info(&person, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner.clone(), approve_all_msg)
        .unwrap();

    // and paginate queries
    let res = contract
        .operators(
            deps.as_ref(),
            mock_env(),
            person.to_string(),
            true,
            None,
            Some(2),
        )
        .unwrap();
    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![
                bs721::Approval {
                    spender: operator.to_string(),
                    expires: Expiration::Never {}
                },
                bs721::Approval {
                    spender: buddy.to_string(),
                    expires: buddy_expires,
                }
            ]
        }
    );
    let res = contract
        .operators(
            deps.as_ref(),
            mock_env(),
            person.to_string(),
            true,
            Some(operator.to_string()),
            Some(1),
        )
        .unwrap();
    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![bs721::Approval {
                spender: buddy.to_string(),
                expires: buddy_expires,
            }]
        }
    );

    let revoke_all_msg = ExecuteMsg::RevokeAll {
        operator: operator.to_string(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), owner, revoke_all_msg)
        .unwrap();

    // Approvals are removed / cleared without affecting others
    let res = contract
        .operators(
            deps.as_ref(),
            mock_env(),
            person.to_string(),
            false,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![bs721::Approval {
                spender: buddy.to_string(),
                expires: buddy_expires,
            }]
        }
    );

    // ensure the filter works (nothing should be here
    let mut late_env = mock_env();
    late_env.block.height = 1234568; //expired
    let res = contract
        .operators(
            deps.as_ref(),
            late_env,
            person.to_string(),
            false,
            None,
            None,
        )
        .unwrap();
    assert_eq!(0, res.operators.len());
}

#[test]
fn query_tokens_by_owner() {
    let mut deps = mock_dependencies();
    let minter = deps.api.addr_make("minter");
    let creator = deps.api.addr_make("creator");
    let minter = deps.api.addr_make("minter");
    let demeter = deps.api.addr_make("demeter");
    let ceres = deps.api.addr_make("ceres");
    let payment = deps.api.addr_make("payment");

    let contract = setup_contract(deps.as_mut(), creator.clone(), minter.clone());
    let minter = message_info(&minter, &[]);

    // Mint a couple tokens (from the same owner)
    let token_id1 = "grow1".to_string();
    let token_id2 = "grow2".to_string();
    let token_id3 = "sing".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id1.clone(),
        owner: demeter.to_string(),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(payment.to_string()),
        token_uri: None,
        extension: None,
    };
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id2.clone(),
        owner: ceres.to_string(),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(payment.to_string()),
        token_uri: None,
        extension: None,
    };
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id3.clone(),
        owner: demeter.to_string(),
        seller_fee_bps: Option::from(100u16),
        payment_addr: Option::from(payment.to_string()),
        token_uri: None,
        extension: None,
    };
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // get all tokens in order:
    let expected = vec![token_id1.clone(), token_id2.clone(), token_id3.clone()];
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(&expected, &tokens.tokens);
    // paginate
    let tokens = contract.all_tokens(deps.as_ref(), None, Some(2)).unwrap();
    assert_eq!(&expected[..2], &tokens.tokens[..]);
    let tokens = contract
        .all_tokens(deps.as_ref(), Some(expected[1].clone()), None)
        .unwrap();
    assert_eq!(&expected[2..], &tokens.tokens[..]);

    // get by owner
    let by_ceres = vec![token_id2];
    let by_demeter = vec![token_id1, token_id3];
    // all tokens by owner
    let tokens = contract
        .tokens(deps.as_ref(), demeter.to_string(), None, None)
        .unwrap();
    assert_eq!(&by_demeter, &tokens.tokens);
    let tokens = contract
        .tokens(deps.as_ref(), ceres.to_string(), None, None)
        .unwrap();
    assert_eq!(&by_ceres, &tokens.tokens);

    // paginate for demeter
    let tokens = contract
        .tokens(deps.as_ref(), demeter.to_string(), None, Some(1))
        .unwrap();
    assert_eq!(&by_demeter[..1], &tokens.tokens[..]);
    let tokens = contract
        .tokens(
            deps.as_ref(),
            demeter.to_string(),
            Some(by_demeter[0].clone()),
            Some(3),
        )
        .unwrap();
    assert_eq!(&by_demeter[1..], &tokens.tokens[..]);
}

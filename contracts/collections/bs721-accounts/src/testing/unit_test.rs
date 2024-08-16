// use bs721_base::ContractError::Unauthorized;
// use bs721_base::{ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg as Bs721InstantiateMsg};
// use btsg_account::{Metadata, TextRecord, NFT};
// use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
// use cosmwasm_std::{
//     from_json, to_json_binary, Addr, ContractInfoResponse, ContractResult, Empty, OwnedDeps,
//     Querier, QuerierResult, QueryRequest, StdError, SystemError, SystemResult, WasmQuery,
// };
// use cw721::Cw721Query;
// use cw_orch::mock::MockBech32;
// use std::marker::PhantomData;

// use super::super::commands::{
//     query_is_twitter_verified, query_name, query_text_records, transcode,
// };
// use super::super::SudoParams;
// use super::super::{execute, instantiate, query};
// use crate::msg::{Bs721AccountsQueryMsg, InstantiateMsg};
// use crate::{ContractError, ExecuteMsg, QueryMsg};
// pub type Btsg721AccountContract<'a> =
//     bs721_base::Bs721Contract<'a, Metadata, Empty, Empty, Bs721AccountsQueryMsg>;
// const CREATOR: &str = "creator";
// const IMPOSTER: &str = "imposter";

// #[test]
// fn init() {
//     // instantiate sg-names collection
//     let mock = MockBech32::new("mock");
//     let info = mock_info(CREATOR, &[]);

//     instantiate(deps.as_mut(), mock_env(), info, init_msg()).unwrap();
// }

// #[test]
// fn mint_and_update() {
//     let contract = Btsg721AccountContract::default();
//     // instantiate sg-names collection
//     let mut deps = mock_deps();
//     let info = mock_info(CREATOR, &[]);

//     instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg()).unwrap();

//     // retrieve max record count
//     let params: SudoParams =
//         from_json(query(deps.as_ref(), mock_env(), QueryMsg::Params {}).unwrap()).unwrap();
//     let max_record_count = params.max_record_count;

//     // mint token
//     let token_id = "Enterprise";

//     let exec_msg = Sg721ExecuteMsg::Mint {
//         token_id: token_id.to_string(),
//         owner: info.sender.to_string(),
//         token_uri: None,
//         seller_fee_bps: None,
//         payment_addr: None,
//         extension: Metadata::default(),
//     };
//     contract
//         .execute(deps.as_mut(), mock_env(), info.clone(), exec_msg)
//         .unwrap();

//     // check token contains correct metadata
//     let res = contract
//         .parent
//         .nft_info(deps.as_ref(), token_id.into())
//         .unwrap();
//     assert_eq!(res.token_uri, mint_msg.token_uri);
//     assert_eq!(res.extension, mint_msg.extension);

//     // update image
//     let new_nft = NFT {
//         collection: Addr::unchecked("contract"),
//         token_id: "token_id".to_string(),
//     };
//     let update_image_msg = ExecuteMsg::UpdateImageNft {
//         name: token_id.to_string(),
//         nft: Some(new_nft.clone()),
//     };
//     let res = execute(deps.as_mut(), mock_env(), info.clone(), update_image_msg).unwrap();
//     let nft_value = res.events[0].attributes[2].value.clone().into_bytes();
//     let nft: NFT = from_json(nft_value).unwrap();
//     assert_eq!(nft, new_nft);

//     // add text record
//     let new_record = TextRecord {
//         name: "test".to_string(),
//         value: "test".to_string(),
//         verified: None,
//     };
//     let update_record_msg = ExecuteMsg::UpdateTextRecord {
//         name: token_id.to_string(),
//         record: new_record.clone(),
//     };
//     let res = execute(deps.as_mut(), mock_env(), info.clone(), update_record_msg).unwrap();
//     let record_value = res.events[0].attributes[2].value.clone().into_bytes();
//     let record: TextRecord = from_json(record_value).unwrap();
//     assert_eq!(record, new_record);

//     let records = query_text_records(deps.as_ref(), token_id).unwrap();
//     assert_eq!(records.len(), 1);
//     assert_eq!(records[0].name, "test");
//     assert_eq!(records[0].value, "test");

//     let is_twitter_verified = query_is_twitter_verified(deps.as_ref(), token_id).unwrap();
//     assert!(!is_twitter_verified);

//     // trigger too many records error
//     for i in 1..=(max_record_count) {
//         let new_record = TextRecord {
//             name: format!("key{:?}", i),
//             value: "value".to_string(),
//             verified: None,
//         };
//         let update_record_msg = ExecuteMsg::UpdateTextRecord {
//             name: token_id.to_string(),
//             record: new_record.clone(),
//         };
//         if i == max_record_count {
//             let res = execute(deps.as_mut(), mock_env(), info.clone(), update_record_msg);
//             assert_eq!(
//                 res.unwrap_err().to_string(),
//                 ContractError::TooManyRecords {
//                     max: max_record_count
//                 }
//                 .to_string()
//             );
//             break;
//         } else {
//             execute(deps.as_mut(), mock_env(), info.clone(), update_record_msg).unwrap();
//         }
//     }

//     // rm text records
//     let rm_record_msg = ExecuteMsg::RemoveTextRecord {
//         name: token_id.to_string(),
//         record_name: "test".to_string(),
//     };
//     execute(deps.as_mut(), mock_env(), info.clone(), rm_record_msg).unwrap();

//     for i in 1..=(max_record_count) {
//         let record_name = format!("key{:?}", i);
//         let rm_record_msg = ExecuteMsg::RemoveTextRecord {
//             name: token_id.to_string(),
//             record_name: record_name.clone(),
//         };
//         execute(deps.as_mut(), mock_env(), info.clone(), rm_record_msg).unwrap();
//     }
//     // txt record count should be 0
//     let res = contract
//         .parent
//         .nft_info(deps.as_ref(), token_id.into())
//         .unwrap();
//     assert_eq!(res.extension.records.len(), 0);

//     // add txt record
//     let record = TextRecord {
//         name: "test".to_string(),
//         value: "test".to_string(),
//         verified: None,
//     };
//     let add_record_msg = ExecuteMsg::AddTextRecord {
//         name: token_id.to_string(),
//         record,
//     };
//     // unauthorized
//     let err = execute(
//         deps.as_mut(),
//         mock_env(),
//         mock_info(IMPOSTER, &[]),
//         add_record_msg.clone(),
//     )
//     .unwrap_err();
//     assert_eq!(
//         err.to_string(),
//         ContractError::Base(Unauthorized {}).to_string()
//     );
//     // passes
//     execute(deps.as_mut(), mock_env(), info.clone(), add_record_msg).unwrap();
//     let res = contract
//         .parent
//         .nft_info(deps.as_ref(), token_id.into())
//         .unwrap();
//     assert_eq!(res.extension.records.len(), 1);

//     // add another txt record
//     let record = TextRecord {
//         name: "twitter".to_string(),
//         value: "jackdorsey".to_string(),
//         verified: None,
//     };
//     let add_record_msg = ExecuteMsg::AddTextRecord {
//         name: token_id.to_string(),
//         record,
//     };
//     execute(deps.as_mut(), mock_env(), info.clone(), add_record_msg).unwrap();
//     let res = contract
//         .parent
//         .nft_info(deps.as_ref(), token_id.into())
//         .unwrap();
//     assert_eq!(res.extension.records.len(), 2);

//     // add duplicate record RecordNameAlreadyExist
//     let record = TextRecord {
//         name: "test".to_string(),
//         value: "testtesttest".to_string(),
//         verified: None,
//     };
//     let add_record_msg = ExecuteMsg::AddTextRecord {
//         name: token_id.to_string(),
//         record: record.clone(),
//     };
//     let err = execute(deps.as_mut(), mock_env(), info.clone(), add_record_msg).unwrap_err();
//     assert_eq!(
//         err.to_string(),
//         ContractError::RecordNameAlreadyExists {}.to_string()
//     );

//     // update txt record
//     let update_record_msg = ExecuteMsg::UpdateTextRecord {
//         name: token_id.to_string(),
//         record: record.clone(),
//     };
//     execute(deps.as_mut(), mock_env(), info.clone(), update_record_msg).unwrap();
//     let res = contract
//         .parent
//         .nft_info(deps.as_ref(), token_id.into())
//         .unwrap();
//     assert_eq!(res.extension.records.len(), 2);
//     assert_eq!(res.extension.records[1].value, record.value);

//     // rm txt record
//     let rm_record_msg = ExecuteMsg::RemoveTextRecord {
//         name: token_id.to_string(),
//         record_name: record.name,
//     };
//     execute(deps.as_mut(), mock_env(), info, rm_record_msg).unwrap();
//     let res = contract
//         .parent
//         .nft_info(deps.as_ref(), token_id.into())
//         .unwrap();
//     assert_eq!(res.extension.records.len(), 1);
// }

// #[test]
// fn query_names() {
//     let deps = mock_deps();
//     let address = "stars1y54exmx84cqtasvjnskf9f63djuuj68p2th570".to_string();
//     let err = query_name(deps.as_ref(), address.clone()).unwrap_err();
//     assert_eq!(
//         err.to_string(),
//         StdError::GenericErr {
//             msg: format!("No name associated with address {}", address)
//         }
//         .to_string()
//     );
// }

// #[test]
// fn test_transcode() {
//     let res = transcode("cosmos1y54exmx84cqtasvjnskf9f63djuuj68p7hqf47");
//     assert_eq!(res.unwrap(), "stars1y54exmx84cqtasvjnskf9f63djuuj68p2th570");
// }

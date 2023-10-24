pub use bs721_base::{ContractError, InstantiateMsg, MinterResponse};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:bs721-metadata-onchain";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
pub enum MediaType {
    Image,
    Audio,
    Video,
}

#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub media_type: Option<MediaType>,
}

pub type Extension = Option<Metadata>;

pub type Bs721MetadataContract<'a> = bs721_base::Bs721Contract<'a, Extension, Empty, Empty, Empty>;
pub type ExecuteMsg = bs721_base::ExecuteMsg<Extension, Empty>;
pub type QueryMsg = bs721_base::QueryMsg<Empty>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        // Explicitly set contract name and version, otherwise set to cw721-base info
        // set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        //     .map_err(ContractError::Std)?;
        // let res = Bs721MetadataContract::default().instantiate(deps.branch(), env, info, msg)?;
        // Ok(res)
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let res = Bs721MetadataContract::default().instantiate(deps.branch(), env, info, msg)?;
        Ok(res)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Bs721MetadataContract::default().execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Bs721MetadataContract::default().query(deps, env, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bs721::Bs721Query;
    use bs721_base::MintMsg;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    const CREATOR: &str = "creator";

    /// Make sure cw2 version info is properly initialized during instantiation,
    /// and NOT overwritten by the base contract.
    #[test]
    fn proper_cw2_initialization() {
        let mut deps = mock_dependencies();

        entry::instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("address1", &[]),
            InstantiateMsg {
                name: "".into(),
                symbol: "".into(),
                minter: "address1".into(),
                uri: None,
                cover_image: None,
                image: None,
            },
        )
        .unwrap();

        let version = cw2::get_contract_version(deps.as_ref().storage).unwrap();
        println!("version: {:?}", version);
        assert_eq!(version.contract, CONTRACT_NAME);
        assert_ne!(version.contract, bs721_base::CONTRACT_NAME);
    }

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_dependencies();
        let contract = Bs721MetadataContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "Yeah!".to_string(),
            symbol: "CRAZY".to_string(),
            minter: CREATOR.to_string(),
            uri: None,
            cover_image: None,
            image: None,
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let token_id = "1";
        //let token_uri = Some("".into());
        let extension = Some(Metadata {
            description: Some("NFT Description".into()),
            name: Some("NFT Name".to_string()),
            media_type: Some(MediaType::Image),
            ..Metadata::default()
        });
        let exec_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
            token_id: token_id.to_string(),
            owner: "address2".to_string(),
            token_uri: None,
            extension: extension.clone(),
            payment_addr: None,
            seller_fee_bps: None,
        });
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        //assert_eq!(res.token_uri, token_uri);
        assert_eq!(res.extension, extension);
    }
}

mod tests {
    use base64::engine::general_purpose::STANDARD;
    use base64_serde::base64_serde_type;
    use btsg_auth::{Any, AuthenticationRequest, SignatureData};
    use btsg_wavs::ContractError;
    use cosmwasm_std::{AnyMsg, Binary};
    base64_serde_type!(Base64Standard, STANDARD);
    use cosmwasm_std::HashFunction::Sha256;

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct WavsOperatorsPubkey(#[serde(with = "Base64Standard")] Vec<u8>);

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct WavsOperatorResolutionObjects {
        public_keys: Vec<WavsOperatorsPubkey>,
        #[serde(with = "Base64Standard")]
        message: Vec<u8>,
        #[serde(with = "Base64Standard")]
        signature: Vec<u8>,
        #[serde(with = "Base64Standard")]
        aggregate_pubkey: Vec<u8>,
    }

    // Generates a predictable BLS12-381 private key and simulates an Ethereum header block
    fn generate_bls12_wavs_operator_signature() -> Result<(), ContractError> {
        Ok(())
    }

    #[test]
    fn test_integration() {
        // Create authentication request

        // Generate the Authentication request format to send to contract

        // let auth_req = Box::new(AuthenticationRequest {
        //     signature: Binary::new(vec![]),
        //     msg: Any {
        //         type_url: todo!(),
        //         value: todo!(),
        //     },
        //     signature_data: todo!(),
        //     authenticator_id: todo!(),
        //     account: todo!(),
        //     fee_payer: todo!(),
        //     fee_granter: todo!(),
        //     fee: todo!(),
        //     msg_index: todo!(),
        //     sign_mode_tx_data: todo!(),
        //     tx_data: todo!(),
        //     simulate: todo!(),
        //     authenticator_params: todo!(),
        // });
    }
}

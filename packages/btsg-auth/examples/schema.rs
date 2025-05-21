use std::env::current_dir;
use std::fs::create_dir_all;

use btsg_auth::{
    Any, AuthenticationRequest, AuthenticatorSudoMsg, ConfirmExecutionRequest,
    OnAuthenticatorRemovedRequest, SignModeTxData, SignatureData, TrackRequest, TxData,
};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

pub fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(AuthenticatorSudoMsg), &out_dir);
    export_schema(&schema_for!(OnAuthenticatorRemovedRequest), &out_dir);
    export_schema(&schema_for!(AuthenticationRequest), &out_dir);
    export_schema(&schema_for!(TrackRequest), &out_dir);
    export_schema(&schema_for!(ConfirmExecutionRequest), &out_dir);
    export_schema(&schema_for!(SignModeTxData), &out_dir);
    export_schema(&schema_for!(TxData), &out_dir);
    export_schema(&schema_for!(SignatureData), &out_dir);
    export_schema(&schema_for!(Any), &out_dir);
}

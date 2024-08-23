use cosmwasm_std::{coins, BankMsg, Response, SubMsg, Uint128};

pub const MAX_TEXT_LENGTH: u32 = 512;
pub const NATIVE_DENOM: &str = "ubtsg";
pub const SECONDS_PER_YEAR: u64 = 31536000;

pub fn charge_fees(res: &mut Response, fee: Uint128) {
    if fee > Uint128::zero() {
        res.messages.push(SubMsg::new(BankMsg::Burn {
            amount: coins(fee.u128(), NATIVE_DENOM),
        }));
    }
}

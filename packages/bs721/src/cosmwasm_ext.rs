use cosmwasm_std::{
    ConversionOverflowError, Decimal, Decimal256, Fraction, StdError, StdResult, Uint128, Uint256,
    Uint64,
};
use std::ops;

/// Trait extension for Decimal256 to work with token precisions more accurately.
pub trait Decimal256Ext {
    fn to_uint256(&self) -> Uint256;

    fn to_uint128_with_precision(&self, precision: impl Into<u32>) -> StdResult<Uint128>;

    fn to_uint256_with_precision(&self, precision: impl Into<u32>) -> StdResult<Uint256>;

    fn from_integer(i: impl Into<Uint256>) -> Self;

    fn checked_multiply_ratio(
        &self,
        numerator: Decimal256,
        denominator: Decimal256,
    ) -> StdResult<Decimal256>;

    fn with_precision(
        value: impl Into<Uint256>,
        precision: impl Into<u32>,
    ) -> StdResult<Decimal256>;
}

impl Decimal256Ext for Decimal256 {
    fn to_uint256(&self) -> Uint256 {
        self.numerator() / self.denominator()
    }

    fn to_uint128_with_precision(&self, precision: impl Into<u32>) -> StdResult<Uint128> {
        let value = self.atomics();
        let precision = precision.into();

        value
            .checked_div(10u128.pow(self.decimal_places() - precision).into())?
            .try_into()
            .map_err(|o: ConversionOverflowError| StdError::msg(format!("Error converting {}", o)))
    }

    fn to_uint256_with_precision(&self, precision: impl Into<u32>) -> StdResult<Uint256> {
        let value = self.atomics();
        let precision = precision.into();

        value
            .checked_div(10u128.pow(self.decimal_places() - precision).into())
            .map_err(|_| StdError::msg("DivideByZeroError"))
    }

    fn from_integer(i: impl Into<Uint256>) -> Self {
        Decimal256::from_ratio(i.into(), 1u8)
    }

    fn checked_multiply_ratio(
        &self,
        numerator: Decimal256,
        denominator: Decimal256,
    ) -> StdResult<Decimal256> {
        Ok(Decimal256::new(
            self.atomics()
                .checked_multiply_ratio(numerator.atomics(), denominator.atomics())
                .map_err(|_| StdError::msg("CheckedMultiplyRatioError"))?,
        ))
    }

    fn with_precision(
        value: impl Into<Uint256>,
        precision: impl Into<u32>,
    ) -> StdResult<Decimal256> {
        Decimal256::from_atomics(value, precision.into())
            .map_err(|_| StdError::msg("Decimal256 range exceeded"))
    }
}
pub trait AbsDiff
where
    Self: Copy + PartialOrd + ops::Sub<Output = Self>,
{
    fn diff(self, rhs: Self) -> Self {
        if self > rhs {
            self - rhs
        } else {
            rhs - self
        }
    }
}

impl AbsDiff for Uint256 {}
impl AbsDiff for Uint128 {}
impl AbsDiff for Uint64 {}
impl AbsDiff for Decimal {}
impl AbsDiff for Decimal256 {}

pub trait IntegerToDecimal
where
    Self: Copy + Into<Uint128> + Into<Uint256>,
{
    fn to_decimal(self) -> Decimal {
        Decimal::from_ratio(self, 1u8)
    }

    fn to_decimal256(self, precision: impl Into<u32>) -> StdResult<Decimal256> {
        Decimal256::with_precision(self, precision)
    }
}

impl IntegerToDecimal for u64 {}
impl IntegerToDecimal for Uint128 {}

pub trait DecimalToInteger<T> {
    fn to_uint(self, precision: impl Into<u32>) -> Result<T, ConversionOverflowError>;
}

impl DecimalToInteger<Uint128> for Decimal256 {
    fn to_uint(self, precision: impl Into<u32>) -> Result<Uint128, ConversionOverflowError> {
        let multiplier = Uint256::from(10u8).pow(precision.into());
        (multiplier * self.numerator() / self.denominator()).try_into()
    }
}

pub trait ConvertInto<T>
where
    Self: Sized,
{
    type Error: Into<StdError>;
    fn conv(self) -> Result<T, Self::Error>;
}

impl ConvertInto<Decimal> for Decimal256 {
    type Error = StdError;

    fn conv(self) -> Result<Decimal, Self::Error> {
        let numerator: Uint128 = self.numerator().try_into()?;
        Decimal::from_atomics(numerator, self.decimal_places())
            .map_err(|err| StdError::msg(err.to_string()))
    }
}

use cosmwasm_std::{coin, Addr, Uint256};

use crate::multitest::suite::{ADDRESS1, ADDRESS2, PAYMENT_RECIPIENT, REFERRAL};

use super::suite::TestSuiteBuilder;

#[test]
fn instantiate() {
    let suite = TestSuiteBuilder::new().build();

    let resp = suite.query_config();

    // ensure created contract addresses are correctly saved in the state
    // commented out as suite does not give us bs721-created right now.
    assert_eq!(
        resp.bs721_address,
        suite.replaced_cw721_addr(),
        "expected bs721 base as second contract stored and saved in the state"
    );
    assert_eq!(
        resp.payment_address,
        Addr::unchecked(PAYMENT_RECIPIENT),
        "expected bs721 royalties as third contract stored and saved in the state"
    )
}

#[test]
fn mint_single_no_referral() {
    let mut suite = TestSuiteBuilder::new()
        .with_funds(ADDRESS1, &[coin(1_000, "ubtsg")])
        .with_price(coin(1, "ubtsg"))
        .build();

    suite
        .mint(ADDRESS1, None, 1, Some(coin(1, "ubtsg")))
        .unwrap();

    // retrieve royalties contract to query it
    let royalties_address = suite.query_config().payment_address;

    assert_eq!(
        suite
            .query_address_balance(royalties_address, "ubtsg")
            .amount,
        Uint256::one(),
        "expected to have the royalties contract balance equal to the price of a single NFT"
    );

    suite
        .mint(ADDRESS1, None, 1, Some(coin(1, "ubtsg")))
        .unwrap_err();
}

#[test]
fn mint_single_with_referral() {
    let mut suite = TestSuiteBuilder::new()
        .with_funds(ADDRESS1, &[coin(1_000, "ubtsg")])
        .with_referral_fee_bps(1_000)
        .with_price(coin(10, "ubtsg"))
        .build();

    let referral = Some(REFERRAL.to_string());
    suite
        .mint(ADDRESS1, referral.clone(), 1, Some(coin(10, "ubtsg")))
        .unwrap();

    // retrieve royalties contract to query it
    let royalties_address = suite.query_config().payment_address;

    assert_eq!(
        suite
            .query_address_balance(referral.unwrap(), "ubtsg")
            .amount,
        Uint256::one(),
        "expected to have the referral address balance equal to the 10% of the NFT price"
    );

    assert_eq!(
        suite
            .query_address_balance(royalties_address, "ubtsg")
            .amount,
        Uint256::new(9),
        "expected to have the royalties contract balance equal to the 90% of the NFT price"
    );
}

#[test]
fn mint_multiple() {
    let mut suite = TestSuiteBuilder::new()
        .with_funds(ADDRESS1, &[coin(1_000, "ubtsg")])
        .with_price(coin(1, "ubtsg"))
        .with_party_type(crate::msg::PartyType::MaxEdition(10))
        .build();

    suite
        .mint(ADDRESS1, None, 3, Some(coin(3, "ubtsg")))
        .unwrap();

    // retrieve royalties contract to query it
    let config = suite.query_config();
    let royalties_address = config.payment_address;

    assert_eq!(
        suite
            .query_address_balance(royalties_address, "ubtsg")
            .amount,
        Uint256::new(3),
        "expected to have the royalties contract balance equal to the price of 3 NFT"
    );

    assert_eq!(
        vec!["1", "2", "3"],
        suite.query_nft_token(config.bs721_address, ADDRESS1),
        "expected 3 nft with sequential ids starting from 1"
    );

    suite
        .mint(ADDRESS1, None, 3, Some(coin(3, "ubtsg")))
        .unwrap();

    let config = suite.query_config();
    assert_eq!(
        vec!["1", "2", "3", "4", "5", "6"],
        suite.query_nft_token(config.bs721_address, ADDRESS1),
        "expected 3 nft with sequential ids starting from 1"
    );
}

#[test]
fn max_per_address() {
    let mut suite = TestSuiteBuilder::new()
        .with_funds(ADDRESS1, &[coin(1_000, "ubtsg")])
        .with_funds(ADDRESS2, &[coin(1_000, "ubtsg")])
        .with_price(coin(1, "ubtsg"))
        .with_party_type(crate::msg::PartyType::MaxEdition(10))
        .with_max_per_address(3)
        .build();

    suite
        .mint(ADDRESS1, None, 1, Some(coin(1, "ubtsg")))
        .unwrap();

    suite
        .mint(ADDRESS1, None, 2, Some(coin(2, "ubtsg")))
        .unwrap();

    suite
        .mint(ADDRESS1, None, 1, Some(coin(1, "ubtsg")))
        .unwrap_err();

    suite
        .mint(ADDRESS2, None, 1, Some(coin(1, "ubtsg")))
        .unwrap();

    suite
        .mint(ADDRESS2, None, 2, Some(coin(2, "ubtsg")))
        .unwrap();

    suite
        .mint(ADDRESS2, None, 1, Some(coin(1, "ubtsg")))
        .unwrap_err();
}

#[test]
fn query_max_per_address() {
    let mut suite = TestSuiteBuilder::new()
        .with_funds(ADDRESS1.as_ref(), &[coin(1_000, "ubtsg")])
        .with_funds(ADDRESS2.as_ref(), &[coin(1_000, "ubtsg")])
        .with_price(coin(1, "ubtsg"))
        .with_party_type(crate::msg::PartyType::MaxEdition(10))
        .with_max_per_address(3)
        .build();

    let config = suite.query_config();

    assert_eq!(
        config.max_per_address,
        Some(3),
        "expected max per address to be 3"
    );

    let response = suite.query_max_per_address(ADDRESS1);
    if let Some(remaining) = response.remaining {
        assert_eq!(
            remaining, 3,
            "expected remaining mintable NFTs for address1 to be 3"
        );
    }

    suite
        .mint(ADDRESS1, None, 1, Some(coin(1, "ubtsg")))
        .unwrap();

    let response = suite.query_max_per_address(ADDRESS1);
    if let Some(remaining) = response.remaining {
        assert_eq!(
            remaining, 2,
            "expected remaining mintable NFTs for address1 to be 2"
        );
    }

    suite
        .mint(ADDRESS1, None, 1, Some(coin(1, "ubtsg")))
        .unwrap();

    let response = suite.query_max_per_address(ADDRESS1);
    if let Some(remaining) = response.remaining {
        assert_eq!(
            remaining, 1,
            "expected remaining mintable NFTs for address1 to be 1"
        );
    }

    suite
        .mint(ADDRESS1, None, 1, Some(coin(1, "ubtsg")))
        .unwrap();

    let response = suite.query_max_per_address(ADDRESS1);
    if let Some(remaining) = response.remaining {
        assert_eq!(
            remaining, 0,
            "expected remaining mintable NFTs for address1 to be 0"
        );
    }

    suite
        .mint(ADDRESS1, None, 1, Some(coin(1, "ubtsg")))
        .unwrap_err();

    let response = suite.query_max_per_address(ADDRESS2);
    if let Some(remaining) = response.remaining {
        assert_eq!(
            remaining, 3,
            "expected remaining mintable NFTs for address2 to be 3"
        );
    }
}

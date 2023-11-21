# LaunchParty Fixed

A smart contract to create an incredible launch party for NFTs sale. This contract act as an orchestrator for the [bs721-base](../bs721-base/) and the [bs721-royalties](../bs721-royalties/) contracts.

## Instantiate

Upon instantiation, the contract automatically creates new instances of the bs721-base and bs721-royalties contracts.

## Execute

The contract handles the following state-changing messages:

* __Mint__: Users can mint an NFT during an active party. The mint message allows users to provide a referral address. To successfully mint a token, the user must send the exact amount of tokens required by the party. The contract then utilizes the bs721-base to mint the token from the base contract.

## Tests

The contract has been designed to have most of the logic encapsulated in standalone functions. This allows to have more simple and short multi-test since single functions can be tested with unit tests.

Tests are organized in the following way:

* _unit tests_: Unit tests are placed at the bottom of each file for which the logic is tested.

* _multi tests_: Multi tests are placed in the [multitest](./src/multitest/) folder and are designed to use a suite to simplify environment creation and interaction.

## License

This project is licensed under the Apache License - see the LICENSE-APACHE file for details.

// Instantiate MSG

{
  "symbol":"FDSF",
  "payment_denom": "ubtsg",
  "max_per_address":10,  
  "collection_image":"ipfs://...",
  "metadata":{
     "name":"fdsf",
     "description":"sdf",
     "image":"ipfs://...",
     "media_type":"image",
     "attributes":[
        
     ]
  },
  "seller_fee_bps":1000,
  "referral_fee_bps":500,
  "contributors":[
     {
        "address":"bitsong1h882ezq7dyewld6gfv2e06qymvjxnu842586h2",
        "shares":1000,
        "role":"creator"
     }
  ],
  "start_time":"1698677500000000000",
  "max_edition":10,
  "bs721_metadata_code_id":26,
  "bs721_royalties_code_id":27,
  "ratio": 10  
}

// Query Buy Price
{
  "buy_price": {
    "amount": "1"
  }
}

// Query Sell Price
{
  "sell_price": {
    "amount": "1"
  }
}

// Mint NFT
// https://api.bwasmnet-1.bitsong.network/txs/7DD8DC488C9DCD177772EFD24E3EA757668C515495D123ECB9550BBB15447236
// referral - https://api.bwasmnet-1.bitsong.network/txs/F82BBCD8DC6827C4DC55D2B0CFDC6B173D9FB12F0EF97A3B48DB9F761EA002F3
{
  "mint": {
    "amount": 1
  }
}

// Burn NFT
// https://api.bwasmnet-1.bitsong.network/txs/4481EAD9C9DDD9D6033A6C72642C9B005C55579A6665BB91FB19A186FB397BF9
// referral - https://api.bwasmnet-1.bitsong.network/txs/7071A4B2A54BA7FAC1B98FE70AF81072715E42C655C6380C6FF250D0C0C0F75F
{
  "burn": {
    "token_ids": [1]
  }
}

// Get Config
{
  "get_config": {}
}

// APPROVE OPERATOR FROM NFT
{"approve_all": {"operator":"bitsong1ru3h6azjtuwh9azj08w7vrxs0r6t6d0pxssyjpv2xqfgm3zhrlhsll5rmg"}}
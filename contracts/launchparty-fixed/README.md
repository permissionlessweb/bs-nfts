# LaunchParty Fixed

A smart contract to create an incredible launch party for NFTs sale.

## Execute

The contract handles the following state-changing messages:

* `Mint`: this message allows a user to mint an NFT if the party is active. The mint message can contain a `referral` address used to [???]. The contract requires that the user send along with the tx the correct amount of tokens required by the party. The contract will create a BS721ExecuteMsg to Mint a token from the base contract.

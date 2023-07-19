# LaunchParty Fixed

A smart contract to create an incredible launch party for NFTs sale. This contract act as an orchestrator for the [bs721-base](../bs721-base/) and the [bs721-royalties](../bs721-royalties/) contracts.

## Instantiate

During instantiation, the contract will also instantiate a new bs721-base and a new bs721-royalties.

## Execute

The contract handles the following state-changing messages:

* `Mint`: This message allows a user to mint an NFT if the party is active. The mint message can contain a referral address. The contract requires the user to send the correct amount of tokens required by the party along with the transaction. The contract will then create a `BS721ExecuteMsg` to mint a token from the base contract.

## Tests

The contract has been designed such that the majority of the logic is encapsulated in standalone functions. This allows to have more simple and short multi-test since single functions can be tested with unit tests.

Tests are organized in the following way:

* _unit tests_: Unit tests are placed at the bottom of each file for which the logic is tested.

* _multi tests_: Multi tests are placed in the [multitest](./src/multitest/) folder and are designed to use a suite to simplify environment creation and interaction.

# btsg-wavs

An authentication contract validated an BLS12_381 signature, specifically for validating actions to take from transactions broadcasted by AVS operators.

 
## TODO
<!-- - verification: assert message to verify has been signed by one of the registered operator AVS keys (registered during `OnAuthenticatorAdded`) -->
<!-- - verification: assert we are accurately parsing the location where we expect the wavs operator AVS public key, signature, and message that was signed from the transaction (specifially from `AuthenticationRequest`) -->
<!-- - msgs: implement contract owner & owner migration workflow (not need, workflow will be to unregister and reregister) -->
-- integration tests

## Forming Authentication Requests Msgs

The following table describes the properties and definitions of the `AuthenticationRequest` object.
| Property | Type | Description | Definition |
| --- | --- | --- | --- |
| `account` | `Addr` | A human-readable address. | [Addr](#addr) |
| `authenticator_id` | `string` | The ID of the authenticator. | - |
| `authenticator_params` | `Binary` \| `null` | Parameters for the authenticator. | [Binary](#binary) |
| `fee` | `Coin[]` | The fee associated with the transaction. | [Coin](#coin) |
| `fee_granter` | `Addr` \| `null` | The address that granted the fee. | [Addr](#addr) |
| `fee_payer` | `Addr` | The address that pays the fee. | [Addr](#addr) |
| `msg` | `Any` | A message of any type. | [Any](#any) |
| `msg_index` | `uint64` | The index of the message. | - |
| `sign_mode_tx_data` | `SignModeTxData` | Data for signing the transaction. | [SignModeTxData](#signmodetxdata) |
| `signature` | `Binary` | The signature of the transaction. | [Binary](#binary) |
| `signature_data` | `SignatureData` | Data related to the signature. | [SignatureData](#signaturedata) |
| `simulate` | `boolean` | Whether to simulate the transaction. | - |
| `tx_data` | `TxData` | Data related to the transaction. | [TxData](#txdata) |


## Aggregated Keys
Top implement support for bls12 signature verification,  we make use of the `SignatureData`  to provide all pubkeys and all their singatures in an arrays. The message of these singatures should be passed into the msg  property, as the `Any` type.
### Definitions

#### Addr
| Property | Type | Description |
| --- | --- | --- |
| - | `string` | A human-readable address. |

#### Any
| Property | Type | Description |
| --- | --- | --- |
| `type_url` | `string` | The URL of the type. |
| `value` | `Binary` | The value associated with the type. |

#### Binary
| Property | Type | Description |
| --- | --- | --- |
| - | `string` | A binary value encoded as a base64 string. |

#### Coin
| Property | Type | Description |
| --- | --- | --- |
| `amount` | `Uint128` | The amount of the coin. |
| `denom` | `string` | The denomination of the coin. |

#### SignModeTxData
| Property | Type | Description |
| --- | --- | --- |
| `sign_mode_direct` | `Binary` | Data for direct signing mode. |
| `sign_mode_textual` | `string` \| `null` | Data for textual signing mode. |

#### SignatureData
| Property | Type | Description |
| --- | --- | --- |
| `signatures` | `Binary[]` | An array of signatures. |
| `signers` | `Addr[]` | An array of signers. |

#### TxData
| Property | Type | Description |
| --- | --- | --- |
| `account_number` | `uint64` | The account number. |
| `chain_id` | `string` | The ID of the chain. |
| `memo` | `string` | A memo associated with the transaction. |
| `msgs` | `Any[]` | An array of messages. |
| `sequence` | `uint64` | The sequence number. |
| `timeout_height` | `uint64` | The timeout height. |

#### Uint128
| Property | Type | Description |
| --- | --- | --- |
| - | `string` | A 128-bit unsigned integer encoded as a string. |
# Bs721 Royalty

This contract allows to distribute its balance to a list of contributors proportionally to their shares.

The below flowchart describes contract's components and how they interact:

![bs721-royalty](./assets/bs721-royalty.png)

## Instantiate

Contract instantiation requires to specify two types:

* `denom`: the native token denom used as contract royalties.

* `contributors`: the list of contributors that will participare in royalties distributions.

Each contributor is described by a custom type, `ContributorMsg`, composed by:

* `address`: the address of the contributor.

* `role`: the role of the contributor. This field cannot be left empty but can be passed an empty string.

* `shares`: shares associated to the contributor. This value will be used to compute the total shares as the sum of the field `shares` of each `ContributorMsg`. This value is then used to compute the percentace of royalties associated to each contributor.

### Instantiate schema

```json
  "instantiate": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "InstantiateMsg",
    "type": "object",
    "required": [
      "contributors",
      "denom"
    ],
    "properties": {
      "contributors": {
        "description": "NFT collection contibutors.",
        "type": "array",
        "items": {
          "$ref": "#/definitions/ContributorMsg"
        }
      },
      "denom": {
        "description": "Native denom distributed to contributors.",
        "type": "string"
      }
    },
    "additionalProperties": false,
    "definitions": {
      "ContributorMsg": {
        "description": "Represents a contributor to the collection.",
        "type": "object",
        "required": [
          "address",
          "role",
          "shares"
        ],
        "properties": {
          "address": {
            "description": "Contributor's address.",
            "type": "string"
          },
          "role": {
            "description": "Contributor's role",
            "type": "string"
          },
          "shares": {
            "description": "Amount of share associated to the contributor.",
            "type": "integer",
            "format": "uint32",
            "minimum": 0.0
          }
        },
        "additionalProperties": false
      }
    }
  },
```

## Execute

Two possibile state-changing messages are handled by the contract:

* `Distribute`: this messages causes the contract to distribute royalties proportionally among contributors. This logic can result in error if there are not enough tokens to be distributed.Distribtued royalties are not sent to contributors but are stored as a field inside the state. Any user can send this message.

* `Withdraw`: this message can be sent only by a contributor and results in a `BankMsg` to send royalties to the sender.

### Execute schema

```json
"execute": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "ExecuteMsg",
    "oneOf": [
      {
        "description": "Update contributors withdrawable amount by computing each contributors percentage of the total distributable contract balance. This function will consider only coins of the stored denom.",
        "type": "object",
        "required": [
          "distribute"
        ],
        "properties": {
          "distribute": {
            "type": "object",
            "additionalProperties": false
          }
        },
        "additionalProperties": false
      },
      {
        "description": "Withdraw accrued royalties. This message can only be sent by a contributor.",
        "type": "object",
        "required": [
          "withdraw"
        ],
        "properties": {
          "withdraw": {
            "type": "object",
            "additionalProperties": false
          }
        },
        "additionalProperties": false
      }
    ]
  },
```

## Query

Below the list of queries that can be resolved by the contract:

* `ListContributors`: allows to retrieve information associated to each contributor.

* `WithdrawableAmount`: returns the sum of the royalties all contributors can withdraw.

* `DistributableAmount`: returns the difference between contract balance and the amount of tokens that can be withdrawn as royalties.

### Query schema

```json
"query": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "QueryMsg",
    "oneOf": [
      {
        "description": "Retrieves the list of contributors.",
        "type": "object",
        "required": [
          "list_contributors"
        ],
        "properties": {
          "list_contributors": {
            "type": "object",
            "properties": {
              "limit": {
                "description": "Number of contributors to receive.",
                "type": [
                  "integer",
                  "null"
                ],
                "format": "uint32",
                "minimum": 0.0
              },
              "start_after": {
                "description": "Address after which contributors are retrieved.",
                "type": [
                  "string",
                  "null"
                ]
              }
            },
            "additionalProperties": false
          }
        },
        "additionalProperties": false
      },
      {
        "description": "Returns the total amount of royalties that can be withdrawn from the contract.",
        "type": "object",
        "required": [
          "withdrawable_amount"
        ],
        "properties": {
          "withdrawable_amount": {
            "type": "object",
            "additionalProperties": false
          }
        },
        "additionalProperties": false
      },
      {
        "description": "Retrieves amount of denom that can be distributed.",
        "type": "object",
        "required": [
          "distributable_amount"
        ],
        "properties": {
          "distributable_amount": {
            "type": "object",
            "additionalProperties": false
          }
        },
        "additionalProperties": false
      }
    ]
  }
```

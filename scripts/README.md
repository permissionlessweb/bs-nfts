# Bitsong NFT Scripts

## Contents
This library contains both Rust & Bash scripts for the BS-NFT repository.


| Name | Language | Version | Description |
|----------|----------|----------|----------|
| [**Testing Suite**](./src/test/mod.rs) | `Rust`   | `tbd`  | Integration test suite for all contracts.  |
| [**Cw-Orch Deployment** ](./src/deploy/mod.rs)  | `Rust`   | `tbd`   | Used for production and simulation environment contract deployment workflows.  |
| **Bitsong Account Framework Deployment**  | `Rust` |`tbd`  | Automation scripts for deployment of smart contract and IBC infrastructure that powers Bitsong Accounts.   |


## Current Orchestrator Suites
| Suite Name | Description |
|----------|----------|
| [`BtsgAccountSuite`](./src/deploy/bundles/account.rs#12)| Account Collection, Marketplace, and Minter. |



## Commands 
| Command | Description |
|----------|----------|
| `cargo test` | Run all test in codebase |
| `sh build_shema.sh` | Build cosmwasm schemas for each contract in codebase |
| `cargo run --bin manual-deploy -- --network-ids <network-id-1> <network-id-2> ... ` | Manual deployment of account framework |
| `cargo run --bin ibc-test` | Test ibc connection of bitsong accounts between 2 chains |
<!-- | `sh simulate.sh` | Simulate bs721-bonding-curve iterations | -->
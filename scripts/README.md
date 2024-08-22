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

**Changelog for Bitsong NFTS**
=====================================================

**`0.3.0-rc` -Cosmwasm-std: v2.0.0**
--------------------------------------
### Added
- **bs-controllers:** add admin item store functions (source from original cw-controllers package)
### Changed
- **bump cosmwasm deps to 2.0.0** - all cosmwasm crates have been bumped to version 2.0  
- **bump workspace tests** - bump all tests to compatible versions with v2.0 cosmwasm-std
### Fixed
### Removed
### Upgrade Notes

**`0.2.0-rc` - Cw Orchestrator & Bitsong Accounts**
--------------------------------------

### Added
* **Changelog** - basic template for manual recordkeeping.
* **Cw-Orchestrator**- advanced testing and deployment tool for CosmWasm smart-contracts https://orchestrator.abstract.money/intro.html
* **Bitsong-Controller Library** - Library for managing hooks & other contract controlling actions.
* **Scripts Library** - Library for interacting with simulation and production environments.

### Changed
* `MintMsg` enum removed from BS71-Base
* Folder reorg

### Fixed
* n/a

### Removed
* [List of features or functionalities removed in this version]
* [Brief description of each removal]

### Upgrade Notes
* Migration Logic - No test have been made on migrating v0.1.0-rc to v0.2.0-rc.


### Credits

* [Abstract Framework](https://github.com/AbstractSDK/abstract)
* [Stargaze Names](https://github.com/public-awesome/names)
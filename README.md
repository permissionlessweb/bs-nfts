# BitSong NFTs

## Overview

This repository is a fork of the original [cw-nfts](https://github.com/CosmWasm/cw-nfts) project. The purpose of this fork is to provide additional functionality to the NFT collections created with this codebase.

## Optimize

To optimize the contract, we use the following tools:

```shell
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.13
```

## Codebase

All other parts of the code remain unchanged and identical to the original cw721 project. This fork is intended to provide additional functionality for those looking to build their own NFT collections, without modifying the underlying codebase.

## Contributing

If you are working on an NFT project as well and wish to give input, please raise issues and/or PRs.
Additional maintainers can be added if they show commitment to the project.

You can also join the [BitSong Discord](https://discord.bitsong.io) server
for more interactive discussions on these themes.

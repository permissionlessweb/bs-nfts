#!/bin/sh

bs721_collection_id=22
bs721_marketplace_id=24
bs721_minter_id=25


admin_key=""
admin_addr=""
binary=bitsongd
chain_id=bobnet
gas_price="0.025ubtsg"
tx_flags="--from=$admin_key  --chain-id $chain_id --gas auto --gas-adjustment 2 --gas-prices=$gas_price -y -o json"
tx_flags_2="--from=$bidder_key --chain-id $chain_id --gas auto --gas-adjustment 2 --gas-prices=$gas_price -y -o json"

init_flags="--label="bs721-profile-marketplace" --admin=$admin_addr"
get_init_addr=""

# create marketplace first 
echo 'a. init marketplace'
init_market=$($binary tx wasm i $bs721_marketplace_id '{"trading_fee_bps": 0,"min_price":"10","ask_interval": 6, "max_renewals_per_block":10, "valid_bid_query_limit": 30,"renew_window": 60, "renewal_bid_percentage": "0.00","operator":"'$admin_addr'" }' $init_flags $tx_flags )
echo 'waiting for tx to process'
market_tx=$(echo "$init_market" | jq -r '.txhash')
sleep 6;
market_q=$($binary q tx $market_tx -o json)
# gets addr from events
bs721_marketplace_addr=$(echo "$market_q" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "marketplace addr: $bs721_marketplace_addr"

# create minter second
echo 'b. init minter'
init_minter=$($binary tx wasm i $bs721_minter_id '{"admin":"'$admin_addr'","collection_code_id":'$bs721_collection_id',"marketplace_addr":"'$bs721_marketplace_addr'","min_name_length": 3,"max_name_length": 64, "base_price":"0","fair_burn_bps":0,"whitelists":[]}'  $init_flags $tx_flags)
echo 'waiting for tx to process'
minter_tx=$(echo "$init_minter" | jq -r '.txhash')
sleep 6;
minter_q=$($binary q tx $minter_tx -o json)
echo $minter_q
# gets addr from events
bs721_minter_addr=$(echo "$minter_q" | jq -r '.logs[].events[] | select(.type == "wasm") | .attributes[] | select(.key == "names_minter_addr") | .value')
echo "minter address: $bs721_minter_addr"
# gets addr from events
bs721_collection_addr=$(echo "$minter_q" | jq -r '.logs[].events[] | select(.type == "wasm") | .attributes[] | select(.key == "bs721_profile_address") | .value')

# before minting, we need to update market config
echo 'update market config'
update_market=$($binary tx wasm e $bs721_marketplace_addr '{"setup": {"minter": "'$bs721_minter_addr'", "collection":"'$bs721_collection_addr'"}}' $tx_flags)
echo 'waiting for tx to process'
update_tx=$(echo "$update_market" | jq -r '.txhash')
sleep 6;
update_msg=$(echo "$update_tx" | jq -r '.txhash')
update_q=$($binary q tx $update_msg -o json)

# now we can mint new profile name
echo 'mint profile token'
mint=$($binary tx wasm e $bs721_minter_addr '{"mint_and_list": {"name": "test-profile-token"}}' $tx_flags)
echo 'waiting for tx to process'
mint_tx=$(echo "$mint" | jq -r '.txhash')
sleep 6;
mint_msg=$(echo "$mint_tx" | jq -r '.txhash')
mint_q=$($binary q tx $mint_msg -o json)
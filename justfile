 

fmt:
    cargo fmt --all --check

test:
    cargo test --locked

lint:
    cargo clippy --tests -- -D warnings

build:
    cargo build --release --locked --target wasm32-unknown-unknown

optimize:
    #!/usr/bin/env bash
    if [[ $(arch) == "arm64" ]]; then
      image="cosmwasm/workspace-optimizer-arm64"
      platform="linux/arm64"
    else
      image="cosmwasm/workspace-optimizer"
      platform="linux/amd64"
    fi

    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      --platform ${platform} \
      ${image}:0.16.1

schema:
    #!/usr/bin/env bash
    sh scripts/schema-codegen.sh

publish:
    #!/usr/bin/env bash
    crates=(
      bs721
      bs721-base
    )

    for crate in "${crates[@]}"; do
      cargo publish -p $crate
      echo "Sleeping before publishing the next crate..."
      sleep 30
    done
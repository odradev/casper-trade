default:
    just -l

test:
    cargo odra test
    cargo odra test -b casper

clippy:
    cargo clippy --all-targets -- -D warnings

lint:
    cargo fmt

check-lint: clippy
    cargo fmt -- --check

run-nctl:
    docker run --rm -it --name mynctl -d -p 11101:11101 -p 14101:14101 -p 18101:18101 -p 25101:25101 makesoftware/casper-nctl:v203

cli *ARGS:
    cargo run --bin casper_trade_cli -- {{ARGS}}

cli-on-nctl *args="":
    set shell := bash
    mkdir -p .node-keys
    # Extract the secret keys from the local Casper node
    docker exec mynctl /bin/bash -c "cat /home/casper/casper-nctl/assets/net-1/users/user-1/secret_key.pem" > .node-keys/secret_key.pem
    docker exec mynctl /bin/bash -c "cat  /home/casper/casper-nctl/assets/net-1/users/user-2/secret_key.pem" > .node-keys/secret_key_1.pem
    # Run the command
    ODRA_CASPER_LIVENET_SECRET_KEY_PATH=.node-keys/secret_key.pem ODRA_CASPER_LIVENET_NODE_ADDRESS=http://localhost:11101 ODRA_CASPER_LIVENET_EVENTS_URL=http://localhost:18101/events ODRA_CASPER_LIVENET_CHAIN_NAME=casper-net-1 ODRA_CASPER_LIVENET_KEY_1=.node-keys/secret_key_1.pem  cargo run --bin casper_trade_cli -- {{args}}

    rm -rf examples/.node-keys


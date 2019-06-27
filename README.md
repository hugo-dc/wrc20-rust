# WRC20 Example written in Rust

## How to compile

Run `make` to generate a `ewasm_token.wasm` file ready to be deployed to the
ewasm testnet.

Run `./compile.sh` to generate a ewasm test case `wrc20ChallengeFiller.yml`
which can be tested using `testeth`.

**Note**: This token contract cannot be deployed to the current ewasm testnet,
this is because ewasm-studio and explorer-deployer are using an older Wabt
version.



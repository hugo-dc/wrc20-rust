WAT_ARGS ?= --fold-exprs --inline-exports --generate-names

all:
	cargo build-ewasm
	wasm-opt -Oz -o target/wasm32-unknown-unknown/release/ewasm_token.wasm target/wasm32-unknown-unknown/release/ewasm_token.wasm

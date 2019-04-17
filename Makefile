WAT_ARGS ?= --fold-exprs --inline-exports --generate-names

all:
	cargo build --target=wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/ewasm_token.wasm ewasm_token.wasm
	chisel ewasm_token.wasm ewasm_token.wasm
	wasm-opt -Oz -o ewasm_token.wasm ewasm_token.wasm
	wasm-snip --snip-rust-panicking-code ewasm_token.wasm -o ewasm_token.wasm
	wasm-minify ewasm_token.wasm ewasm_token.wasm

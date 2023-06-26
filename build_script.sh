rm -rf wasm_files
mkdir wasm_files

cargo build --release --target wasm32-unknown-unknown --package file_storage --locked
mv target/wasm32-unknown-unknown/release/file_storage.wasm wasm_files/
ic-wasm wasm_files/file_storage.wasm shrink
gzip wasm_files/file_storage.wasm
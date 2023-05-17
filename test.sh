# # WILL THROW GIANT ERROR SECOND TIME, Rawerror 9 (CodeAlreadyExists)

mkdir -p .testdata;

cat .testdata/code_id || cargo run substrate -c ws://127.0.0.1:9988 -n alice tx upload --url https://github.com/CosmWasm/cw-plus/releases/download/v1.0.1/cw4_group.wasm > .testdata/output

# the tool does not error out correctly, so we need to manually check the output
P="Code ID"
grep "$P" .testdata/output && grep "$P" .testdata/output |sed s/[^0-9]*// > .testdata/code_id

cat .testdata/code_id || exit;

# WILL THROW GIANT ERROR SECOND TIME, Rawerror 11 (ContractAlreadyExists)
cat .testdata/contract_address || cargo run substrate -c ws://127.0.0.1:9988 -n alice tx instantiate --code-id $(cat code_id) --salt 0x12345 --label lol --gas 10000000000 --message '{"admin": null, "members": []}' > .testdata/output

# the tool does not error out correctly, so we need to manually check the output
P="_contract_address"
grep $P .testdata/output && grep $P .testdata/output | sed s/.*:\ // > .testdata/contract_address

cat .testdata/contract_address || exit;


# QUERY
cargo run substrate -c http://127.0.0.1:9988 -n alice rpc query --contract $(cat .testdata/contract_address) --gas 10000000000 --query '{"total_weight": {}}'

# EXECUTE
cargo run substrate -c ws://127.0.0.1:9988 -n alice tx execute --contract $(cat .testdata/contract_address) --gas 10000000000 --message '{"update_admin": { "admin": null }}'

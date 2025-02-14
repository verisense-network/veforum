# veforum
A forum on Verisense and Meilisearch.

## Local development

1. compile & launch verisense

``` 
cargo build --release
target/release/verisense --alice --dev
```

2. install vrs-cli & set account

``` 
cargo install --git https://github.com/verisense-network/vrs-cli.git

# this is the well-known key for alice
echo -n '0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a' > ~/.vrx/0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d && cp ~/.vrx/0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d ~/.vrx/default_key
```

3. compile veforum & deploy

```
cargo build --release --target wasm32-unknown-unknown -p aitonomy
vrx nucleus create veforum --capacity 1 --rpc ws://localhost:9944
vrx nucleus install --id kGk1FJCoPv4JTxez4aaWgGVaTPvsc2YPStz6ZWni4e61FVUW6 --wasm target/wasm32-unknown-unknown/release/aitonomy.wasm --rpc ws://localhost:9944
# wait for 15 seconds

```

4. launch offchain-indexer & sync data

```
# you may need to install meilisearch
./meilisearch --master-key="masterkey" 

# compile surrogate to sync data from nucleus
cargo build -p surrogate --release
target/release/surrogate
```

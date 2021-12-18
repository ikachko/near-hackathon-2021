ACCOUNT=kkdex.testnet
# near deploy $ACCOUNT res/near_orderbook.wasm new "[10,\"kstasi.testnet\",true,10,10]"
near deploy $ACCOUNT res/near_orderbook.wasm new "[\"ft.examples.testnet\", \"keku.testnet\"]"

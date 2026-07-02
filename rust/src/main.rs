#![allow(unused)]
use bitcoin::hex::DisplayHex;
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;

// Node access params
const RPC_URL: &str = "http://127.0.0.1:18443"; // Default regtest RPC port
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

// You can use calls not provided in RPC lib API using the generic `call` function.
// An example of using the `send` RPC call, which doesn't have exposed API.
// You can also use serde_json `Deserialize` derivation to capture the returned json result.
fn send(rpc: &Client, addr: &str) -> bitcoincore_rpc::Result<String> {
    let args = [
        json!([{addr : 100 }]), // recipient address
        json!(null),            // conf target
        json!(null),            // estimate mode
        json!(null),            // fee rate in sats/vb
        json!(null),            // Empty option object
    ];

    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let send_result = rpc.call::<SendResult>("send", &args)?;
    assert!(send_result.complete);
    Ok(send_result.txid)
}

fn main() -> bitcoincore_rpc::Result<()> {
    // Connect to Bitcoin Core RPC
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // Get blockchain info
    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Blockchain Info: {:?}", blockchain_info);

    // Create/Load the wallets, named 'Miner' and 'Trader'. Have logic to optionally create/load them if they do not exist or not loaded already.
    let miner_rpc = match rpc.create_wallet("Miner", None, None, None, None) {
    Ok(_) => Client::new(
        &format!("{}/wallet/Miner", RPC_URL),
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?,
    Err(_) => {
        let _ = rpc.load_wallet("Miner");
        Client::new(
            &format!("{}/wallet/Miner", RPC_URL),
            Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
        )?
    }
};


    // Generate spendable balances in the Miner wallet. How many blocks needs to be mined?
    let miner_address = miner_rpc.get_new_address(Some("Mining Reward"), None)?;
    let miner_address = miner_address.require_network(bitcoincore_rpc::bitcoin::Network::Regtest)?;

    // Mine 101 blocks to this address
    // WHY 101: Bitcoin has a rule that mining rewards cannot be spent until
    // 100 more blocks are mined on top of them. This is called "coinbase maturity".
    // So block 1 gives us the reward, blocks 2-101 make it spendable.
    rpc.generate_to_address(101, &miner_address)?;

    // Print Miner balance to confirm we have funds
    let miner_balance = miner_rpc.get_balance(None, None)?;
    println!("Miner balance: {} BTC", miner_balance);

        // Load Trader wallet and generate a new address
        let trader_rpc = match rpc.create_wallet("Trader", None, None, None, None) {
        Ok(_) => Client::new(
            &format!("{}/wallet/Trader", RPC_URL),
            Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
        )?,
        Err(_) => {
            let _ = rpc.load_wallet("Trader");
            Client::new(
                &format!("{}/wallet/Trader", RPC_URL),
                Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
            )?
        }
    };


    // Send 20 BTC from Miner to Trader
    

    // Check transaction in mempool

    // Mine 1 block to confirm the transaction

    // Extract all required transaction details

    // Write the data to ../out.txt in the specified format given in readme.md

    Ok(())
}

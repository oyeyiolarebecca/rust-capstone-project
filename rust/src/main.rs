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
    let miner_address = miner_address
        .require_network(bitcoincore_rpc::bitcoin::Network::Regtest)
        .unwrap();
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

    // This is the address Miner will send 20 BTC to
    let trader_address = trader_rpc.get_new_address(Some("Received"), None)?;
    let trader_address = trader_address
        .require_network(bitcoincore_rpc::bitcoin::Network::Regtest)
        .unwrap();

    // Send 20 BTC from Miner to Trader
    // Amount::from_btc converts the decimal 20.0 into the correct Bitcoin amount type
    let txid = miner_rpc.send_to_address(
        &trader_address,
        Amount::from_btc(20.0)?,
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    println!("Transaction sent! TXID: {}", txid);

    // Check transaction in mempool
    // The mempool is the waiting room — transactions sit here until confirmed in a block
    let mempool_entry =
        rpc.call::<serde_json::Value>("getmempoolentry", &[json!(txid.to_string())])?;
    println!("Mempool entry: {}", mempool_entry);

    // Mine 1 block to confirm the transaction
    // This moves the transaction from the mempool into the blockchain permanently
    rpc.generate_to_address(1, &miner_address)?;
    println!("1 block mined — transaction confirmed!");

    // Extract all required transaction details
    // Get full transaction details from the node
    let tx = miner_rpc.get_transaction(&txid, Some(true))?;
    let raw_tx = miner_rpc.get_raw_transaction(&txid, None)?;

    // Get the block this transaction was confirmed in
    let block_hash = tx.info.blockhash.unwrap();
    let block_info = rpc.get_block_info(&block_hash)?;
    let block_height = block_info.height;

    // Get Trader output and Miner change from raw transaction outputs
    // A Bitcoin transaction has two outputs:
    // 1. Payment to Trader (20 BTC)
    // 2. Change back to Miner (leftover BTC minus fee)
    let mut trader_output_address = String::new();
    let mut trader_output_amount = bitcoincore_rpc::bitcoin::SignedAmount::ZERO;
    let mut miner_change_address = String::new();
    let mut miner_change_amount = bitcoincore_rpc::bitcoin::SignedAmount::ZERO;

    for output in &raw_tx.output {
        // Convert the script to an address
        let addr = bitcoincore_rpc::bitcoin::Address::from_script(
            &output.script_pubkey,
            bitcoincore_rpc::bitcoin::Network::Regtest,
        );
        if let Ok(addr) = addr {
            let addr_str = addr.to_string();
            let amount =
                bitcoincore_rpc::bitcoin::SignedAmount::from_sat(output.value.to_sat() as i64);
            if addr_str == trader_address.to_string() {
                // This output went to Trader
                trader_output_address = addr_str;
                trader_output_amount = amount;
            } else {
                // This output is Miner's change
                miner_change_address = addr_str;
                miner_change_amount = amount;
            }
        }
    }

    // Transaction fee is what the miner earned for including this transaction
    let fee = tx
        .fee
        .unwrap_or(bitcoincore_rpc::bitcoin::SignedAmount::ZERO)
        .abs();

    // Write the data to ../out.txt in the specified format given in readme.md
    // Each piece of information goes on its own line as required
    let mut file = File::create("../out.txt")?;
    writeln!(file, "{}", txid)?;
    writeln!(file, "{}", miner_address)?;
    writeln!(file, "{}", miner_balance.to_btc())?;
    writeln!(file, "{}", trader_output_address)?;
    writeln!(file, "{}", trader_output_amount.to_btc())?;
    writeln!(file, "{}", miner_change_address)?;
    writeln!(file, "{}", miner_change_amount.to_btc())?;
    writeln!(file, "{}", fee.to_btc())?;
    writeln!(file, "{}", block_height)?;
    writeln!(file, "{}", block_hash)?;

    println!("Output written to out.txt successfully!");

    Ok(())
}

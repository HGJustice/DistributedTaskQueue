use task_queue::types::Operations; 
use anyhow::Result;


#[tokio::main]
async fn main() -> Result<()> {
    println!("Fetching BTC price...");
    let btc_price = Operations::get_current_btc_price().await?;
    println!("BTC Price: ${:.2}", btc_price);

    println!("Fetching ETH price...");
    let eth_price = Operations::get_current_eth_price().await?;
    println!("ETH Price: ${:.2}", eth_price);
    Ok(())
}
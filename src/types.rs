use anyhow::{anyhow, Context, Ok, Result};
use std::collections::{HashMap, BinaryHeap};
use std::fs::File;
use std::io::{Read, Write};


const MAX_TASK_RETRY: u32 = 3;

pub enum Operations {
    OpenFile,
    WriteToFile, 
    GetBTCPrice, 
    GetETHPrice,
}

impl Operations {
    pub fn open_file(path: &str) -> Result<()> {
        let mut data_file = File::open(path).context("Coudlnt open file")?;
        let mut contents = String::new();
        data_file.read_to_string(&mut contents).context("couldnt read the file")?;
        println!("The file reads: {:?}", contents);
        Ok(())
    }

    pub fn create_and_write_to_file(word: &str) -> Result<()> {
        let mut data_file = File::create("data.txt").context("Couldnt create file")?;
        data_file.write(word.as_bytes()).context("failed to write to file")?;
        println!("Created a file data.txt");
        Ok(())
    }

    pub async fn get_current_btc_price() -> Result<f64> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.coingecko.com/api/v3/simple/price")
            .query(&[
                ("ids", "bitcoin"),
                ("vs_currencies", "usd")
            ])
            .send()
            .await
            .context("Failed to send request")?;
    
        let price_data: serde_json::Value = response.json().await
            .context("Failed to parse JSON")?;
        
        let price = price_data["bitcoin"]["usd"]
            .as_f64()
            .ok_or_else(|| anyhow!("Failed to extract price"))?;
    
        Ok(price)

    }

    pub async fn get_current_eth_price() -> Result<f64> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.coingecko.com/api/v3/simple/price")
            .query(&[
                ("ids", "ethereum"),
                ("vs_currencies", "usd")
            ])
            .send()
            .await
            .context("Failed to send request")?;
    
        let price_data: serde_json::Value = response.json().await
            .context("Failed to parse JSON")?;
        
        let price = price_data["ethereum"]["usd"]
            .as_f64()
            .ok_or_else(|| anyhow!("Failed to extract price"))?;
    
        Ok(price)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum Priority {
    High(u32),
    Medium(u32),
    Low(u32)
}

pub struct Tasks {
    task_type: Operations,
    priority_level: Priority,
    retry_counter: u32,
}

impl Tasks {
    pub fn new(task_type: Operations, priority_level: Priority) -> Tasks {
        Tasks { task_type, priority_level , retry_counter: 0 }
    }
}

pub struct TaskQueue {
    priority_manager: BinaryHeap<Priority>,
    task_manager: HashMap<u32, Tasks>
}

impl TaskQueue {
    pub fn new() -> TaskQueue {
        TaskQueue { priority_manager: BinaryHeap::new() , task_manager: HashMap::new() }
    }
    
}

use anyhow::{anyhow, bail, Context, Ok, Result};
use std::collections::{HashMap, BinaryHeap};
use std::fs::File;
use std::io::{Read, Write};
use std::str::Chars;

const MAX_TASK_RETRY: u32 = 3;

#[derive(PartialEq, Debug)] 
pub enum Operations {
    OpenFile,
    WriteToFile, 
    GetBTCPrice, 
    GetETHPrice,
}

impl Operations {
    pub fn open_file() -> Result<()> {
        let mut data_file = File::open("/Users/szymonlyzwinski/Documents/Rust/distributed _task_queue/task_queue/test_file.txt").context("Coudlnt open file")?;
        let mut contents = String::new();
        data_file.read_to_string(&mut contents).context("couldnt read the file")?;
        println!("The file reads: {:?}", contents);
        Ok(())
    }

    pub fn create_and_write_to_file(word: &str) -> Result<()> {
        let mut data_file = File::create("new_file_created.txt").context("Couldnt create file")?;
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

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
pub enum Priority {
    Low(u32),
    Medium(u32),
    High(u32),
}

#[derive(PartialEq)] 
pub struct Tasks {
   pub task_type: Operations,
   pub priority_level: Priority,
   pub retry_counter: u32,
}

impl Tasks {
    pub fn new(task_type: Operations, priority_level: Priority) -> Tasks {
        Tasks { task_type, priority_level , retry_counter: 0 }
    }
}

pub struct TaskQueue {
    pub task_counter: u32,
    pub priority_manager: BinaryHeap<Priority>,
    pub task_manager: HashMap<u32, Tasks>,
    pub failed_task_manager: BinaryHeap<Priority>,
}

impl TaskQueue {
    pub fn new() -> TaskQueue {
        TaskQueue { task_counter: 1, priority_manager: BinaryHeap::new(), task_manager: HashMap::new(), failed_task_manager: BinaryHeap::new() }
    }

    pub fn insert_task(&mut self, task: Operations, priority_level: Priority) -> Result<()> {
        let counter = self.task_counter;

        let task_priority = match priority_level {
            Priority::High(_) => Priority::High(counter),
            Priority::Medium(_) => Priority::Medium(counter),
            Priority::Low(_) => Priority::Low(counter),
        };

        let new_task = Tasks::new(task, task_priority.clone());
        self.task_manager.insert(counter, new_task);
        self.priority_manager.push(task_priority);
        self.task_counter += 1;
        Ok(())
    }

    pub fn get_task(&self, task_key: u32) -> Result<&Tasks> {
        let result = self.task_manager.get(&task_key).ok_or(anyhow!("key value doesnt exisit"))?;
        Ok(result)
    }

    pub fn get_priority_task(&self) -> Result<&Priority> {
        let result = self.priority_manager.peek().ok_or(anyhow!("no priority avaiable"))?;
        Ok(result)
    }

    pub async fn execute_task(&mut self,) -> Result<()>{
        let first = self.priority_manager.pop().ok_or(anyhow!("nothing in the queue"))?;
        let task_key = match first {
            Priority::High(key) | Priority::Medium(key) | Priority::Low(key) => key
        };
        let task = self.task_manager.get_mut(&task_key).ok_or(anyhow!("task not found"))?;
        
        match task.task_type {
            Operations::OpenFile => {
                let result = Operations::open_file();
                if result.is_err(){
                    self.failed_task_manager.push(first);
                    task.retry_counter += 1;
                }
            },
            Operations::WriteToFile => {
                let result = Operations::create_and_write_to_file("Hi");
                if result.is_err(){
                    self.failed_task_manager.push(first);
                    task.retry_counter += 1;
                }
            },
            Operations::GetBTCPrice => {
                let btc_price = Operations::get_current_btc_price().await;
                if btc_price.is_err(){
                    self.failed_task_manager.push(first);
                    task.retry_counter += 1;
                }
                println!("The current BTC Price is {:?}", btc_price);
            }
            Operations::GetETHPrice => {
                let eth_price = Operations::get_current_eth_price().await;
                if eth_price.is_err(){
                    self.failed_task_manager.push(first);
                    task.retry_counter += 1;
                }
                println!("The current ETH Price is {:?}", eth_price);
            }
        }
            Ok(())
}

    pub async fn re_execute_task(&mut self,) -> Result<()> {
        let retry_priority = self.failed_task_manager.pop().ok_or(anyhow!("retry manager is empty"))?;
        let retry_key =  match retry_priority  {
            Priority::High(key) | Priority::Medium(key) | Priority::Low(key) => key
        };
        let task = self.task_manager.get_mut(&retry_key).ok_or(anyhow!("task manager empty"))?;
        if task.retry_counter >= MAX_TASK_RETRY {
            bail!("Max retry amount reached"); //code duplication ??? whole function 
        }

        match task.task_type {
            Operations::OpenFile => {
                let result = Operations::open_file();
                if result.is_err() {
                    self.failed_task_manager.push(retry_priority);
                    task.retry_counter += 1;
                }
            },
            Operations::WriteToFile => {
                let result = Operations::create_and_write_to_file("reecexuted");
                if result.is_err() {
                    self.failed_task_manager.push(retry_priority);
                    task.retry_counter += 1;
                }
            },
            Operations::GetBTCPrice => {
                let btc_price = Operations::get_current_btc_price().await;
                if btc_price.is_err(){
                    self.failed_task_manager.push(retry_priority);
                    task.retry_counter += 1;
                }
                println!("The current BTC Price is {:?}", btc_price);
            },
            Operations::GetETHPrice => {
                let eth_price = Operations::get_current_eth_price().await;
                if eth_price.is_err(){
                    self.failed_task_manager.push(retry_priority);
                    task.retry_counter += 1;
                }
                println!("The current ETH Price is {:?}", eth_price);
            }
        }
        Ok(())
    }

    pub fn create_workers(&mut self, num_workers: usize, ) -> Result<()>{
        todo!();
    }
}
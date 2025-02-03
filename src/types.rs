use anyhow::{anyhow, bail, Context, Ok, Result};
use std::collections::{HashMap, BinaryHeap};
use std::fs::File;
use std::io::{Read, Write};
use std::time::Duration;
use std::sync::Arc;  
use tokio::sync::Mutex; 

const MAX_TASK_RETRY: u32 = 3;

#[derive(PartialEq, Debug, Clone)] 
pub enum Operations {
    OpenFile,
    WriteToFile, 
    GetBTCPrice, 
    GetETHPrice,
}

impl Operations {

    pub fn sort_counter(counter: u32, shortcut: &str) -> Result<Priority> {
        match shortcut.to_lowercase().as_str() {
            "low" => Ok(Priority::Low(counter)),
            "medium" => Ok(Priority::Medium(counter)),
            "high" => Ok(Priority::High(counter)),
            _ => bail!("invalid choice, must be low, medium or high"),
        }
    }

    pub fn open_file(input: &str) -> Result<()> {
        let mut data_file = File::open(input).context("Coudlnt open file")?;
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

#[derive(PartialEq, Clone)] 
pub struct Tasks {
   pub task_type: Operations,
   pub priority_level: Priority,
   pub retry_counter: u32,
   pub delay: Duration,
}

impl Tasks {
    pub fn new(task_type: Operations, priority_level: Priority) -> Tasks {
        Tasks { task_type, priority_level , retry_counter: 0, delay: Duration::new(0, 0) }
    }
}

#[derive(Clone)]
pub struct TaskQueue {
    pub task_counter: Arc<Mutex<u32>>,
    pub priority_manager: Arc<Mutex<BinaryHeap<Priority>>>,
    pub task_manager: Arc<Mutex<HashMap<u32, Tasks>>>,
    pub failed_task_manager: Arc<Mutex<BinaryHeap<Priority>>>,
}

impl TaskQueue {
    pub fn new() -> TaskQueue {
        TaskQueue { task_counter: Arc::new(Mutex::new(1)), priority_manager: Arc::new(Mutex::new(BinaryHeap::new())), task_manager: Arc::new(Mutex::new(HashMap::new())), failed_task_manager: Arc::new(Mutex::new(BinaryHeap::new())) }
    }

    pub async fn insert_task(&mut self, task: Operations, priority_level: &str) -> Result<()> {
        let counter = *self.task_counter.lock().await;
        let task_priority = Operations::sort_counter(counter, priority_level)?;
       
        let new_task = Tasks::new(task, task_priority.clone());
        self.task_manager.lock().await.insert(counter, new_task);
        self.priority_manager.lock().await.push(task_priority);
        let mut counter_lock = self.task_counter.lock().await;
        *counter_lock += 1;
        Ok(())
    }

    pub async fn get_task(&self, task_key: u32) -> Result<Tasks> {
        let result = self.task_manager.lock().await.get(&task_key).cloned().ok_or(anyhow!("key value doesnt exisit"))?;
        Ok(result)
    }

    pub async fn get_priority_task(&self) -> Result<Priority> {
        let result = self.priority_manager.lock().await.peek().cloned().ok_or(anyhow!("no priority avaiable"))?;
        Ok(result)
    }

    pub async fn execute_task(&mut self,) -> Result<()>{
        let first = self.priority_manager.lock().await.pop().ok_or(anyhow!("nothing in the queue"))?;
        let task_key = match first {
            Priority::High(key) | Priority::Medium(key) | Priority::Low(key) => key
        };
        let mut task_manager = self.task_manager.lock().await;
        let task = task_manager.get_mut(&task_key).ok_or(anyhow!("task not found"))?;
        
        match task.task_type {
            Operations::OpenFile => {
                let result = Operations::open_file("/Users/szymonlyzwinski/Documents/Rust/distributed _task_queue/task_queue/test_file.txt");
                if result.is_err(){
                    self.failed_task_manager.lock().await.push(first);
                    task.retry_counter += 1;
                    task.delay = Duration::from_secs(2u64.pow(task.retry_counter));
                }
            },
            Operations::WriteToFile => {
                let result = Operations::create_and_write_to_file("Hi");
                if result.is_err(){
                    self.failed_task_manager.lock().await.push(first);
                    task.retry_counter += 1;
                    task.delay = Duration::from_secs(2u64.pow(task.retry_counter));
                }
            },
            Operations::GetBTCPrice => {
                let btc_price = Operations::get_current_btc_price().await;
                if btc_price.is_err(){
                    self.failed_task_manager.lock().await.push(first);
                    task.retry_counter += 1;
                    task.delay = Duration::from_secs(2u64.pow(task.retry_counter));
                }
                println!("The current BTC Price is {:?}", btc_price);
            }
            Operations::GetETHPrice => {
                let eth_price = Operations::get_current_eth_price().await;
                if eth_price.is_err(){
                    self.failed_task_manager.lock().await.push(first);
                    task.retry_counter += 1;
                    task.delay = Duration::from_secs(2u64.pow(task.retry_counter));
                }
                println!("The current ETH Price is {:?}", eth_price);
            }
        }
            Ok(())
}

    pub async fn re_execute_task(&mut self,) -> Result<()> {
        let retry_priority = self.failed_task_manager.lock().await.pop().ok_or(anyhow!("retry manager is empty"))?;
        let retry_key =  match retry_priority  {
            Priority::High(key) | Priority::Medium(key) | Priority::Low(key) => key
        };

        let mut task_manager = self.task_manager.lock().await;
        let task = task_manager.get_mut(&retry_key).ok_or(anyhow!("task not found"))?;
         //code duplication ??? whole function 
        if task.retry_counter >= MAX_TASK_RETRY {
            bail!("Max retry amount reached");
        }

        match task.task_type {
            Operations::OpenFile => {
                let result = Operations::open_file("/Users/szymonlyzwinski/Documents/Rust/distributed _task_queue/task_queue/test_file.txt");
                if result.is_err() {
                    self.failed_task_manager.lock().await.push(retry_priority);
                    task.retry_counter += 1;
                    task.delay = Duration::from_secs(2u64.pow(task.retry_counter));
                }
            },
            Operations::WriteToFile => {
                let result = Operations::create_and_write_to_file("reecexuted");
                if result.is_err() {
                    self.failed_task_manager.lock().await.push(retry_priority);
                    task.retry_counter += 1;
                    task.delay = Duration::from_secs(2u64.pow(task.retry_counter)); // 2 ** retry counter
                }
            },
            Operations::GetBTCPrice => {
                let btc_price = Operations::get_current_btc_price().await;
                if btc_price.is_err(){
                    self.failed_task_manager.lock().await.push(retry_priority);
                    task.retry_counter += 1;
                    task.delay = Duration::from_secs(2u64.pow(task.retry_counter));
                }
                println!("The current BTC Price is {:?}", btc_price);
            },
            Operations::GetETHPrice => {
                let eth_price = Operations::get_current_eth_price().await;
                if eth_price.is_err(){
                    self.failed_task_manager.lock().await.push(retry_priority);
                    task.retry_counter += 1;
                    task.delay = Duration::from_secs(2u64.pow(task.retry_counter));
                }
                println!("The current ETH Price is {:?}", eth_price);
            }
        }
        Ok(())
    }

    pub async fn start_task(mut self, ) -> Result<()> {
        loop {
            if !self.priority_manager.lock().await.is_empty() {
                self.execute_task().await?;
            }

            if !self.failed_task_manager.lock().await.is_empty(){
                let task_priority = self.failed_task_manager.lock().await.pop().ok_or(anyhow!("failed manager is empty"))?;
                let task_key = match task_priority {
                    Priority::High(key) | Priority::Medium(key) | Priority::Low(key) => key
                };
               
                let task = self.get_task(task_key).await.unwrap();
              
                tokio::time::sleep(task.delay).await;
                self.re_execute_task().await?;
            }
        }
    }

    pub async fn create_workers(&mut self, num_workers: usize, ) -> Result<()> {
        if num_workers == 0 as usize {
            bail!("Workers threads needs to be Greater then 0");
        }

        let mut handles = vec![];
      
        for _ in 0..num_workers {
            let queue = self.clone(); // expensive to clone the task queue for x number of threads?, box t?
            let handle =  tokio::spawn(async move {
                queue.start_task().await.unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
           handle.await.unwrap();
        }
        Ok(())
    }
}
use task_queue::types::*;

mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_operations_openfile() {
        let result = Operations::open_file("/Users/szymonlyzwinski/Documents/Rust/distributed _task_queue/task_queue/test_file.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_operations_create_write_file(){
        let result =  Operations::create_and_write_to_file("wag 1");
        assert!(result.is_ok());
    }

    #[tokio::test]
     async fn test_current_btc_price(){
        let result = Operations::get_current_btc_price().await;
        assert!(result.is_ok());
        let price = result.unwrap();
        assert!(price > 90_000.0 && price < 115_000.0);
    }

    #[tokio::test]
    async fn test_current_eth_price(){
        let result = Operations::get_current_eth_price().await;
        assert!(result.is_ok());
        let price = result.unwrap();
        assert!(price > 2000.0 && price < 3000.0);
    }

    #[tokio::test]
   async fn test_task_queue_new(){
        let result = TaskQueue::new();
        assert!(*result.task_counter.lock().await == 1 && result.priority_manager.lock().await.is_empty() && result.task_manager.lock().await.is_empty() && result.failed_task_manager.lock().await.is_empty());
    }

    #[tokio::test]
    async fn test_insert_task(){
        let mut queue = TaskQueue::new();
        queue.insert_task(Operations::WriteToFile, "high").await.unwrap();
        let task = queue.get_task(1).await.unwrap();
        assert_eq!(task.task_type, Operations::WriteToFile);
        assert_eq!(task.retry_counter, 0);
    }

    #[tokio::test]
    async fn test_priority_manager(){
        let mut queue = TaskQueue::new();
        queue.insert_task(Operations::GetBTCPrice, "low").await.unwrap();
        queue.insert_task(Operations::GetETHPrice, "high").await.unwrap();
        queue.insert_task(Operations::OpenFile, "medium").await.unwrap();
        queue.insert_task(Operations::WriteToFile, "high").await.unwrap();

        let result = queue.priority_manager.lock().await.pop().unwrap();
        let result2 = queue.priority_manager.lock().await.pop().unwrap();
        assert!(result == Priority::High(4) && result2 == Priority::High(2));
        let result3 = queue.priority_manager.lock().await.pop().unwrap();
        let result4 = queue.priority_manager.lock().await.pop().unwrap();
        assert!(result3 == Priority::Medium(3) && result4 == Priority::Low(1));
        assert!(queue.priority_manager.lock().await.is_empty());
    }

    #[tokio::test]
    async fn  test_execute_single_threaded(){
        let mut queue = TaskQueue::new();
        queue.insert_task(Operations::OpenFile, "low").await.unwrap();
        queue.insert_task(Operations::GetETHPrice, "high").await.unwrap();
        queue.insert_task(Operations::WriteToFile, "high").await.unwrap();
        queue.insert_task(Operations::GetBTCPrice, "medium").await.unwrap();

        queue.execute_task().await.unwrap();
        queue.execute_task().await.unwrap();
        
        let next = queue.priority_manager.lock().await.pop().unwrap();
        assert_eq!(next, Priority::Medium(4));
    }

    // #[tokio::test]
    // async fn test_rexecute_single_threaded(){
    //    let mut queue = TaskQueue::new();
    //    queue.insert_task(Operations::OpenFile, Priority::Low(0)).await.unwrap();         works when delete "test_file.txt"
    //    queue.execute_task().await.unwrap();                                              but openFile will fail in return
    //    assert!(!queue.failed_task_manager.lock().await.is_empty());
    //    assert_eq!(queue.task_manager.lock().await.get(&1).unwrap().retry_counter, 1);
    // }

    #[tokio::test]
    async fn  test_threads(){
        let mut queue = TaskQueue::new();
        queue.insert_task(Operations::GetBTCPrice, "high").await.unwrap();
        queue.insert_task(Operations::GetETHPrice, "high").await.unwrap();
        queue.insert_task(Operations::OpenFile, "medium").await.unwrap();
        queue.insert_task(Operations::OpenFile, "low").await.unwrap();
        queue.insert_task(Operations::WriteToFile, "medium").await.unwrap();

        tokio::time::timeout(Duration::from_secs(7), queue.create_workers(2)).await.unwrap_err();

        assert!(queue.priority_manager.lock().await.is_empty());
    }

}


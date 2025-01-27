use task_queue::types::*;

mod tests {
    use super::*;

    #[test]
    fn test_operations_openfile() {
        let result = Operations::open_file();
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
        assert!(price > 90_000.0 && price < 105_000.0);
    }

    #[tokio::test]
    async fn test_current_eth_price(){
        let result = Operations::get_current_eth_price().await;
        assert!(result.is_ok());
        let price = result.unwrap();
        assert!(price > 3000.0 && price < 3500.0);
    }

    #[test]
    fn test_task_queue_new(){
        let result = TaskQueue::new();
        assert!(result.task_counter == 1 && result.priority_manager.is_empty() && result.task_manager.is_empty() && result.failed_task_manager.is_empty());
    }

    #[test]
    fn test_insert_task(){
        let mut queue = TaskQueue::new();
        queue.insert_task(Operations::WriteToFile, Priority::High(0)).unwrap();
        let task = queue.get_task(1).unwrap();
        assert_eq!(task.task_type, Operations::WriteToFile);
        assert_eq!(task.retry_counter, 0);
    }

    #[test]
    fn test_priority_manager(){
        let mut queue = TaskQueue::new();
        queue.insert_task(Operations::GetBTCPrice, Priority::Low(0)).unwrap();
        queue.insert_task(Operations::GetETHPrice, Priority::High(0)).unwrap();
        queue.insert_task(Operations::OpenFile, Priority::Medium(0)).unwrap();
        queue.insert_task(Operations::WriteToFile, Priority::High(0)).unwrap();

        let result = queue.priority_manager.pop().unwrap();
        let result2 = queue.priority_manager.pop().unwrap();
        assert!(result == Priority::High(4) && result2 == Priority::High(2));
        let result3 = queue.priority_manager.pop().unwrap();
        let result4 = queue.priority_manager.pop().unwrap();
        assert!(result3 == Priority::Medium(3) && result4 == Priority::Low(1));
        assert!(queue.priority_manager.is_empty());
    }

    #[test]
    fn test_execute_single_threaded(){
        let mut queue = TaskQueue::new();
        queue.insert_task(Operations::OpenFile, Priority::Low(0)).unwrap();
        queue.insert_task(Operations::GetETHPrice, Priority::High(0)).unwrap();
        queue.insert_task(Operations::WriteToFile, Priority::High(0)).unwrap();
        queue.insert_task(Operations::GetBTCPrice, Priority::Low(0)).unwrap();

        queue.execute_task();
        queue.execute_task();
        
        // while !queue.priority_manager.is_empty() {
            
        // }
    }

    // #[test]
    // fn test_rexecute_single_threaded(){
    //     todo!();
    // }

}


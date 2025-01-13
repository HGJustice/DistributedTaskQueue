use std::collections::{HashMap, BinaryHeap};


const MAX_TASK_RETRY: u32 = 5;

enum Operations {
    OpenFile,
    WriteToFile, 
    GetBTCPrice, 
    GetETHPrice,
}

impl Operations {
    
}

enum Priority {
    Low(u32), 
    Medium(u32),
    High(u32)
}

pub struct Tasks {
    task_type: Operations,
    priority_level: Priority,
    retry_counter: u32,
}

pub struct TaskQueue{
    priority_manager: BinaryHeap<Priority>,
    task_manager: HashMap<u32, Tasks>
}
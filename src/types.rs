use std::collections::HashMap;

enum Tasks{
    OpenFile,
    WriteToFile,
    Addition,
    Subtraction,
}

enum Priority {
    Low(u32), 
    Medium(u32),
    High(u32)
}



pub struct TaskQueue{
    //binary heap
    //hashmap
}
/*
A Sender or SyncSender is used to send data to a Receiver. 
Both senders are clone-able (multi-producer) such that 
many threads can send simultaneously to one receiver (single-consumer).
*/
use std::{sync::mpsc::{Sender, channel}};

/* 
Automatically generate clone method without manual implementation
This trait can be used with #[derive] if all fields are Clone. 
The derived implementation of Clone calls clone on each field.
If every field in a struct implements Clone, then you can just call clone on each field and now you’ve cloned the whole struct.
Sender can be cloned
*/ 
#[derive(Clone)]
pub struct Executor {
    task_sender: Sender<Task>,
}

// enumeration type for tasks
pub enum Task {
    Println(String),
    Exit,
}

impl Executor {
    // constructor of Executor
    pub fn new() -> Self {
        // format: pub fn channel<T>() -> (Sender<T>, Receiver<T>)
        let (sender, receiver) = channel();
        std::thread::spawn(move || {
            loop {
                match receiver.recv() {
                    // when using match
                    // x => y conducts y when x is matched
                    Ok(task) => {
                        match task {
                            Task::Println(string) => println!("{}", string),
                            Task::Exit => return
                        }
                    },
                    Err(_) => {
                        return;
                    }
                }
            }
        });
        // return the Executor
        Executor { task_sender: sender }
    }
    // print func for Executor
    pub fn println(&self, string: String) {
        // .send() attempts to send a value on this channel, returning it back if it could not be sent.
        self.task_sender.send(Task::Println(string)).unwrap()
    }
}


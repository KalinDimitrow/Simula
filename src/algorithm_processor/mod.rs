use crossbeam::channel::*;
use std::thread;
use std::time::Duration;
pub type Data = f32;
pub type ProcessedDataHandle = Receiver<Data>;
pub enum ThreadControlMessage {
    Stop,
}

pub struct AlgorithmProcessor {
    // sender: Sender<Data>,
    worker_controller: Sender<ThreadControlMessage>,
    worker: Option<thread::JoinHandle<()>>,
}

impl AlgorithmProcessor {
    pub fn new() -> (ProcessedDataHandle, Self) {
        let (sender, receiver): (Sender<Data>, ProcessedDataHandle) = unbounded();
        let (worker_controller, controller_listener): (
            Sender<ThreadControlMessage>,
            Receiver<ThreadControlMessage>,
        ) = unbounded();

        let handle = thread::spawn(move || {
            let mut count: u64 = 0;
            loop {
                // Check for the stop signal
                if let Ok(_) = controller_listener.try_recv() {
                    println!("Received stop signal, exiting thread.");
                    break;
                }

                sender
                    .send(count as f32 * std::f32::consts::PI / 180.0)
                    .expect("Receiver is already closed");
                count += 1;
                thread::sleep(Duration::from_millis(40));
            }
        });
        (
            receiver,
            Self {
                // sender,
                worker_controller,
                worker: Some(handle),
            },
        )
    }

    fn shutdown(&mut self) {
        self.worker_controller
            .send(ThreadControlMessage::Stop)
            .expect("Channel is already closed");

        if let Some(handle) = self.worker.take() {
            handle.join().unwrap();
        }
    }
}

impl Drop for AlgorithmProcessor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

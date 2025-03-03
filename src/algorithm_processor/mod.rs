use crate::application::CustomEvent;
use crate::application::CustomEventProxy;
use crossbeam::channel::*;
use rand::Rng;
use std::thread;
use std::time::Duration;
pub type Data = Vec<f32>;
pub type ProcessedDataHandle = Receiver<Data>;
pub enum ThreadControlMessage {
    Stop,
}

pub struct AlgorithmProcessor {
    worker_controller: Sender<ThreadControlMessage>,
    worker: Option<thread::JoinHandle<()>>,
}

impl AlgorithmProcessor {
    pub fn new(event_proxy: CustomEventProxy) -> (ProcessedDataHandle, Self) {
        let (sender, receiver): (Sender<Data>, ProcessedDataHandle) = unbounded();
        let (worker_controller, controller_listener): (
            Sender<ThreadControlMessage>,
            Receiver<ThreadControlMessage>,
        ) = unbounded();

        let handle = thread::spawn(move || {
            let mut count: u64 = 0;
            let size = 20 * 20;
            let mut rng = rand::rng();
            let storage_data: Vec<f32> = (0..size).map(|_| rng.random_range(-1.0..1.0)).collect();

            loop {
                if let Ok(_) = controller_listener.try_recv() {
                    println!("Received stop signal, exiting thread.");
                    break;
                }

                // let data: Vec<f32> = vec![count as f32 * std::f32::consts::PI / 180.0; size];
                let data = storage_data
                    .iter()
                    .map(|e| e * (count as f32) * std::f32::consts::PI / 180.0)
                    .collect();
                sender
                    // .send(count as f32 * std::f32::consts::PI / 180.0)
                    .send(data)
                    .expect("Receiver is already closed");
                let _ = event_proxy.send_event(CustomEvent::RequestRedraw);
                count += 1;
                thread::sleep(Duration::from_millis(40));
            }
        });
        (
            receiver,
            Self {
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

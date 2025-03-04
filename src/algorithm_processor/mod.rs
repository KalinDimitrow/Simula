use crate::application::CustomEvent;
use crate::application::CustomEventProxy;
use crate::application::SharedContext;
use crossbeam::channel::*;
use rand::Rng;
use std::thread;
use std::time::Duration;
pub type Data = Vec<f32>;
pub type ProcessedDataHandle = Receiver<Data>;
pub enum ThreadControlMessage {
    Stop,
}

struct WorkerContext {
    controller_listener: Receiver<ThreadControlMessage>,
    sender: Sender<Data>,
    event_proxy: CustomEventProxy,
}

pub struct AlgorithmProcessor {
    ctx: Option<WorkerContext>,
    worker: Option<thread::JoinHandle<WorkerContext>>,
    worker_controller: Sender<ThreadControlMessage>,
}

impl AlgorithmProcessor {
    pub fn new(event_proxy: CustomEventProxy) -> (ProcessedDataHandle, Self) {
        let (sender, receiver): (Sender<Data>, ProcessedDataHandle) = unbounded();
        let (worker_controller, controller_listener): (
            Sender<ThreadControlMessage>,
            Receiver<ThreadControlMessage>,
        ) = unbounded();

        (
            receiver,
            Self {
                ctx: Some(WorkerContext {
                    controller_listener,
                    sender,
                    event_proxy,
                }),
                worker_controller,
                worker: None,
            },
        )
    }

    pub fn start(&mut self, shared_ctx: SharedContext) {
        if self.ctx.is_none() {
            self.shutdown()
        };

        let ctx = self
            .ctx
            .take()
            .expect("There is serious bug the threading code in algorithm processor");
        self.worker = Some(thread::spawn(move || {
            let mut count: u64 = 0;
            let size = shared_ctx.latice_dimentions.0 * shared_ctx.latice_dimentions.1;
            let mut rng = rand::rng();
            let storage_data: Vec<f32> = (0..size).map(|_| rng.random_range(-1.0..1.0)).collect();

            loop {
                if let Ok(_) = ctx.controller_listener.try_recv() {
                    println!("Received stop signal, exiting thread.");
                    break;
                }

                let data = storage_data
                    .iter()
                    .map(|e| e * (count as f32) * std::f32::consts::PI / 180.0)
                    .collect();
                ctx.sender.send(data).expect("Receiver is already closed");
                let _ = ctx.event_proxy.send_event(CustomEvent::RequestRedraw);
                count += 1;
                thread::sleep(Duration::from_millis(40));
            }
            ctx
        }));
    }

    pub fn shutdown(&mut self) {
        self.worker_controller
            .send(ThreadControlMessage::Stop)
            .expect("Channel is already closed");

        if let Some(handle) = self.worker.take() {
            self.ctx = Some(handle.join().unwrap());
        }
    }
}

impl Drop for AlgorithmProcessor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

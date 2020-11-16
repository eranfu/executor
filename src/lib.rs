use std::future::Future;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::task::Poll;

use futures::future::BoxFuture;
use futures::FutureExt;
use futures::task;
use futures::task::{ArcWake, Context};

pub struct Executor {
    ready_queue: Receiver<Arc<Task>>
}

pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>
}

struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output=()> + 'static + Send) {
        let future = future.boxed();
        let task = Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        };
        self.task_sender.send(Arc::new(task)).expect("too many tasks queued")
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.task_sender.send(arc_self.clone()).expect("too many tasks queued")
    }
}

impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = task::waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if let Poll::Pending = future.as_mut().poll(context) {
                    *future_slot = Some(future);
                }
            }
        }
    }
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = mpsc::sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

use bevy::prelude::Resource;
use std::{
    future::Future,
    io,
    sync::{
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
};
use tokio::runtime::{Builder, Runtime};

const CORE_ASYNC_WORKER_THREADS: usize = 2;

#[derive(Resource)]
pub(super) struct CoreAsyncRuntime {
    runtime: Runtime,
}

impl CoreAsyncRuntime {
    pub(super) fn new() -> Result<Self, io::Error> {
        Builder::new_multi_thread()
            .thread_name("shogun-core-async")
            .worker_threads(CORE_ASYNC_WORKER_THREADS)
            .enable_all()
            .build()
            .map(|runtime| Self { runtime })
    }

    pub(super) fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    pub(super) fn spawn_event_task<T, F>(
        &self,
        events: &CoreAsyncEvents<T>,
        future: F,
    ) -> tokio::task::JoinHandle<()>
    where
        T: Send + 'static,
        F: Future<Output = T> + Send + 'static,
    {
        let sender = events.sender.clone();
        self.spawn(async move {
            let event = future.await;
            let _ = sender.send(event);
        })
    }
}

pub(super) struct CoreAsyncEvents<T> {
    sender: Sender<T>,
    receiver: Mutex<Receiver<T>>,
}

impl<T> Default for CoreAsyncEvents<T> {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

impl<T> CoreAsyncEvents<T> {
    pub(super) fn drain(&self) -> Vec<T> {
        self.receiver
            .lock()
            .map(|receiver| receiver.try_iter().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    #[test]
    fn core_async_runtime_runs_spawned_tasks() {
        let runtime = CoreAsyncRuntime::new().expect("runtime starts");
        let events = CoreAsyncEvents::default();

        std::mem::drop(runtime.spawn_event_task(&events, async move {
            tokio::time::sleep(Duration::from_millis(1)).await;
            42
        }));

        assert_eq!(wait_for_events(&events, 1), vec![42]);
    }

    #[test]
    fn core_async_runtime_runs_multiple_tasks_concurrently() {
        let runtime = CoreAsyncRuntime::new().expect("runtime starts");
        let events = CoreAsyncEvents::default();

        for value in 0..2 {
            std::mem::drop(runtime.spawn_event_task(&events, async move {
                tokio::time::sleep(Duration::from_millis(25)).await;
                value
            }));
        }
        let mut values = wait_for_events(&events, 2);
        values.sort_unstable();

        assert_eq!(values, vec![0, 1]);
    }

    fn wait_for_events<T>(events: &CoreAsyncEvents<T>, count: usize) -> Vec<T> {
        let mut collected = Vec::new();
        for _ in 0..100 {
            collected.extend(events.drain());
            if collected.len() >= count {
                return collected;
            }
            thread::sleep(Duration::from_millis(10));
        }
        collected
    }
}

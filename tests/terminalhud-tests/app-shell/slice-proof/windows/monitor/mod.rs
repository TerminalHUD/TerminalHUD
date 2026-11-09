//! Read-only observer; no callbacks, hooks, activation, or live trace formatting.
mod probe;
mod report;
use super::Result;
use crate::observation::{Assessment, Sample, Span, Stage, Window, classify};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
const CAPACITY: usize = 10_000;
struct Buffer {
    stage: Stage,
    readings: Vec<probe::Reading>,
    truncated: bool,
}
impl Buffer {
    fn sample(&mut self, origin: Instant) -> u64 {
        let reading = probe::read(origin, self.stage);
        let elapsed = reading.sample.elapsed_us;
        if self.readings.len() < CAPACITY {
            self.readings.push(reading);
        } else {
            self.truncated = true;
        }
        elapsed
    }
}
pub(super) struct Monitor {
    buffer: Arc<Mutex<Buffer>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    origin: Instant,
}
impl Monitor {
    pub fn start() -> Result<Self> {
        let origin = Instant::now();
        let buffer = Arc::new(Mutex::new(Buffer {
            stage: Stage::Setup,
            readings: Vec::with_capacity(CAPACITY),
            truncated: false,
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let (shared, stopping) = (buffer.clone(), stop.clone());
        let worker = thread::Builder::new()
            .name("startup-observer".into())
            .spawn(move || {
                while !stopping.load(Ordering::Acquire) {
                    let Ok(mut state) = shared.lock() else {
                        return;
                    };
                    if origin.elapsed() >= Duration::from_secs(90) {
                        state.truncated = true;
                        return;
                    }
                    state.sample(origin);
                    drop(state);
                    thread::sleep(Duration::from_millis(10));
                }
            })?;
        Ok(Self {
            buffer,
            stop,
            thread: Some(worker),
            origin,
        })
    }
    pub fn elapsed_us(&self) -> u64 {
        self.origin
            .elapsed()
            .as_micros()
            .try_into()
            .unwrap_or(u64::MAX)
    }
    pub fn mark(&self, stage: Stage) -> Result<u64> {
        let mut buffer = self
            .buffer
            .lock()
            .map_err(|_| "observation buffer poisoned")?;
        buffer.stage = stage;
        Ok(buffer.sample(self.origin))
    }
    pub fn snapshot(&self) -> Result<(Vec<Sample>, bool)> {
        let buffer = self
            .buffer
            .lock()
            .map_err(|_| "observation buffer poisoned")?;
        Ok((
            buffer.readings.iter().map(|r| r.sample).collect(),
            buffer.truncated || self.thread.as_ref().is_some_and(JoinHandle::is_finished),
        ))
    }
    pub fn assess(
        &self,
        stage: Stage,
        sink: Window,
        target: Option<Window>,
        start_us: u64,
        through_quit: bool,
    ) -> Result<Assessment> {
        let end_us = self.mark(stage)?;
        let (samples, truncated) = self.snapshot()?;
        Ok(classify(
            &samples,
            sink,
            target,
            Span {
                start_us,
                end_us,
                through_quit,
            },
            truncated,
        ))
    }
    pub fn stop(&mut self) -> Result<()> {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.thread.take() {
            let deadline = Instant::now() + Duration::from_secs(1);
            while !worker.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            if !worker.is_finished() {
                return Err("observer unwind pending; no thread killed".into());
            }
            worker.join().map_err(|_| "observer thread panicked")?;
        }
        Ok(())
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            eprintln!("OBSERVER_UNWIND {error}");
        }
    }
}

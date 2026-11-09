use super::{Monitor, Result};
use std::{fs::OpenOptions, io::Write, path::Path};
impl Monitor {
    /// Finite capped snapshots, formatted only after the measured startup outcome.
    pub fn write(&self, path: &Path) -> Result<()> {
        let buffer = self
            .buffer
            .lock()
            .map_err(|_| "observation buffer poisoned")?;
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        writeln!(
            file,
            "READ_ONLY_TRACE truncated={} samples={} tick_semantics=raw_session_specific_nonmonotonic",
            buffer.truncated,
            buffer.readings.len()
        )?;
        for reading in &buffer.readings {
            let sample = &reading.sample;
            writeln!(
                file,
                "sample elapsed_us={} stage={:?} input_tick={:?} foreground={:?} first_hwnd={} second_hwnd={} foreground_pid={} foreground_tid={} owner_query_error={:?}",
                sample.elapsed_us,
                sample.stage,
                sample.input_tick,
                sample.foreground,
                reading.foreground_first,
                reading.foreground_second,
                reading.foreground_pid,
                reading.foreground_tid,
                reading.owner_error
            )?;
        }
        file.sync_all()?;
        Ok(())
    }
}

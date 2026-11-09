use super::super::{active, observe, target::Launched};
use super::{Pending, Result, Trace, Window, json};
use crate::observation::{
    Stage,
    full::{Raw, Signals},
};
impl Trace {
    fn scan(&mut self, planned: Option<(u64, u64)>) -> Result<Signals> {
        let (samples, truncated) = self.monitor.snapshot()?;
        let observed = self.control.scan(
            &samples[self.cursor..],
            self.sink,
            self.current,
            planned,
            truncated,
        );
        self.cursor = samples.len();
        Ok(observed)
    }
    pub fn begin(&mut self, name: &'static str, generation: u8) -> Result<()> {
        let begin = self.monitor.mark(Stage::Startup)?;
        // Startup setup samples are excluded once, never rebaseline later phases.
        if self.cursor == 0 {
            let (s, _) = self.monitor.snapshot()?;
            self.cursor = s.iter().take_while(|s| s.elapsed_us < begin).count();
        }
        let seen = self.scan(None)?;
        if !self.control.gate(name) {
            self.phases.push(json!({
                "name":name,
                "generation":generation,
                "raw_result":"Interrupted",
                "begin_us":begin,
                "end_us":begin,
                "observation":seen.json(),
            }));
            return Err("full observation interrupted before next scripted mutation".into());
        }
        self.generation = generation;
        self.active = Some(Pending {
            name,
            generation,
            begin,
            click: None,
            seen,
        });
        Ok(())
    }
    pub fn remember(&mut self, target: &Launched) -> Result<()> {
        self.current = Some(Window {
            hwnd: target.hwnd,
            pid: target.pid,
            tid: target.tid,
        });
        if self
            .retained
            .as_ref()
            .is_none_or(|old| old.nonce != target.nonce)
        {
            self.epoch = Some(target.epoch(self.generation, &self.profile)?);
            self.retained = Some(target.retain()?);
        }
        Ok(())
    }
    pub fn end(
        &mut self,
        result: Result<()>,
        target: Option<&Launched>,
        active: bool,
    ) -> Result<()> {
        // Native assertion result is already known. Metadata failures may stop
        // later mutations, but cannot erase that result (including final pass).
        if let Some(target) = target
            && let Err(error) = self.remember(target)
        {
            self.metadata_error = Some(error.to_string());
            self.control.signals.insufficient = true;
        }
        let end = match self.monitor.mark(if active {
            Stage::InspectActive
        } else {
            Stage::InspectRecovered
        }) {
            Ok(end) => end,
            Err(error) => {
                self.metadata_error = Some(error.to_string());
                self.control.signals.insufficient = true;
                self.monitor.elapsed_us()
            }
        };
        let pending = self.active.take().ok_or("missing full phase boundary")?;
        let planned = if result.is_ok() { pending.click } else { None };
        let mut seen = pending.seen;
        match self.scan(planned) {
            Ok(observed) => seen.merge(observed),
            Err(error) => {
                self.metadata_error = Some(error.to_string());
                self.control.signals.insufficient = true;
                seen.insufficient = true;
            }
        }
        seen.insufficient |= self.metadata_error.is_some();
        if pending.name == "close-first" && self.first_closed != Some(true) {
            seen.insufficient = true;
            self.control.signals.insufficient = true;
        }
        if pending.name == "quit-restore" && result.is_ok() {
            self.restored = true;
        }
        let raw = if result.is_ok() {
            "Passed"
        } else if self.control.stopped && self.control.raw == Raw::Interrupted {
            "Interrupted"
        } else {
            "Failed"
        };
        self.phases.push(json!({
            "name":pending.name,
            "generation":pending.generation,
            "raw_result":raw,
            "begin_us":pending.begin,
            "end_us":end,
            "observation":seen.json(),
        }));
        let error = result.as_ref().err().map(ToString::to_string);
        let keep_going = self.control.native_result(pending.name, error);
        result?;
        if !keep_going {
            return Err("full observation interrupted; native outcome retained".into());
        }
        Ok(())
    }
    pub fn close_gate(&mut self, target: &Launched, ready: Result<()>) -> Result<()> {
        let ready = ready.and_then(|_| active::ensure(target));
        self.monitor.mark(Stage::Startup)?;
        let seen = self.scan(None)?;
        if let Some(pending) = &mut self.active {
            pending.seen.merge(seen);
        }
        if let Err(error) = ready {
            self.control.signals.insufficient = true;
            if let Some(pending) = &mut self.active {
                pending.seen.insufficient = true;
            }
            self.control
                .interrupt("close-first", &format!("pre-close safety guard: {error}"));
        }
        if !self.control.gate("close-first") {
            return self.end(
                Err("close refused on uncertainty before WM_CLOSE".into()),
                None,
                false,
            );
        }
        Ok(())
    }
    pub fn after_close(&mut self, target: &Launched, result: Result<()>) -> Result<()> {
        if result.is_ok() {
            self.first_absent = true;
            self.first_closed = Some(observe::wait(|| target.closed()).is_ok());
            self.current = None;
            if self.first_closed == Some(true) {
                self.epoch = None;
                self.retained = None;
            }
        }
        self.end(result, None, false)
    }
    pub fn click_begin(&mut self) -> u64 {
        let t = self.monitor.elapsed_us();
        self.click = json!({
            "attempted":true,
            "verified":false,
            "begin_us":t,
            "end_us":null,
            "concurrent_input_unexcluded":true,
        });
        t
    }
    pub fn click_end(&mut self, begin: u64, verified: bool) {
        let end = self.monitor.elapsed_us();
        self.click["end_us"] = end.into();
        self.click["verified"] = verified.into();
        if let Some(p) = &mut self.active {
            p.click = Some((begin, end));
        }
    }
}

use super::super::{Command, client::Owner};
use super::{Result, Trace, json};
use crate::observation::full::{FixtureBasis, Raw, fixture_basis};
use std::{fs::OpenOptions, io::Write, path::Path};
impl Trace {
    fn fixture_basis(&self) -> FixtureBasis {
        fixture_basis(
            self.retained.is_some(),
            self.first_closed,
            self.relaunch_attempted,
        )
    }
    pub fn restore(&mut self, owner: &Owner) {
        if !owner.quit_state().0 {
            let result = owner.call(Command::Quit).and_then(|_| owner.wait_exit());
            if let Err(error) = result {
                self.metadata_error = Some(format!("normal Quit unconfirmed: {error}"));
                return;
            }
        }
        if owner.quit_state().2 {
            match &self.retained {
                Some(target) => match target.restored() {
                    Ok(()) => self.restored = true,
                    Err(e) => self.metadata_error = Some(format!("restoration unconfirmed: {e}")),
                },
                None if self.fixture_basis() == FixtureBasis::NoFixture => self.restored = true,
                None => {
                    self.metadata_error = Some("no retained epoch for restoration assertion".into())
                }
            }
        }
    }
    pub fn write_report(
        &self,
        path: &Path,
        manifest: &Path,
        owner: Option<&Owner>,
        owner_epoch: Option<serde_json::Value>,
        launch_attempted: bool,
        witnesses_ok: bool,
    ) -> Result<i32> {
        let (attempted, confirmed, exited) = owner
            .map(Owner::quit_state)
            .unwrap_or((false, false, false));
        let restoration = confirmed && exited && self.restored;
        let disposition = if !launch_attempted
            || restoration && self.fixture_basis() == FixtureBasis::NoFixture
        {
            "NoFixture"
        } else if restoration && self.retained.is_some() {
            if self.control.raw == Raw::Passed {
                "RestoredRetained"
            } else {
                "PreservedUncertain"
            }
        } else {
            "CleanupUnconfirmed"
        };
        let exit =
            if self.metadata_error.is_some() || !witnesses_ok || launch_attempted && !restoration {
                2
            } else {
                match self.control.raw {
                    Raw::Passed => {
                        if self.control.signals.unclear() {
                            3
                        } else {
                            0
                        }
                    }
                    Raw::Failed => 2,
                    Raw::Interrupted => 3,
                }
            };
        let value = json!({
            "schema":"terminalhud.full-observe.v1",
            "scenario":"full-lifecycle-observed",
            "manifest":manifest,
            "profile":self.profile.to_string(),
            "launch_attempted":launch_attempted,
            "owner":owner_epoch,
            "raw_result":format!("{:?}",self.control.raw),
            "reason":self.control.reason,
            "last_completed_phase":self.control.last_completed,
            "failed_or_stopped_phase":self.control.stopped_phase,
            "observation":self.control.signals.json(),
            "phases":self.phases,
            "planned_click":self.click,
            "quit":{
                "attempted":attempted,
                "confirmed_unmanaged":confirmed,
                "owner_exited":exited,
                "restoration_verified":self.restored,
            },
            "first_generation":{
                "availability_absent":self.first_absent,
                "epoch_closed":self.first_closed,
            },
            "disposition":disposition,
            "retained_epoch":self.epoch,
            "metadata_error":self.metadata_error,
            "witnesses_unchanged":witnesses_ok,
            "overall_exit":exit,
        });
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        serde_json::to_writer_pretty(&mut file, &value)?;
        writeln!(file)?;
        file.sync_all()?;
        println!(
            "APP_FULL_REPORT schema=terminalhud.full-observe.v1 raw={:?} disposition={disposition} overall_exit={exit} path={}",
            self.control.raw,
            path.display()
        );
        Ok(exit)
    }
}

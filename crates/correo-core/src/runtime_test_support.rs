use std::time::{Duration, Instant};

use super::*;

impl AppRuntime {
    pub(crate) fn recv_settings_event_timeout(
        &self,
        timeout: Duration,
    ) -> Option<SettingsPersistenceEvent> {
        let event = self.settings_worker.as_ref()?.recv_event_timeout(timeout)?;
        self.apply_settings_event(event.clone());
        Some(event)
    }

    pub(crate) fn wait_for_migration(
        &mut self,
        timeout: Duration,
        condition: impl Fn(&Self) -> bool,
    ) -> bool {
        let deadline = Instant::now() + timeout;
        while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
            let Some(event) = self
                .migration_worker
                .as_ref()
                .and_then(|worker| worker.recv_event_timeout(remaining))
            else {
                return false;
            };
            self.model.apply_event(event);
            if condition(self) {
                return true;
            }
        }
        false
    }
}

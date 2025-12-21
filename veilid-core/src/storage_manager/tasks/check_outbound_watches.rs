use super::*;

impl StorageManager {
    // Check if client-side watches on opened records either have dead nodes or if the watch has expired
    //#[instrument(level = "trace", target = "stor", skip_all, err)]
    pub(super) fn check_outbound_watches_task_routine(
        &self,
        _stop_token: StopToken,
        _last_ts: Timestamp,
        _cur_ts: Timestamp,
    ) -> EyreResult<()> {
        let mut inner = self.inner.lock();
        let cur_ts = Timestamp::now_non_decreasing();

        // Update per-node watch states
        // Desired state updates are performed by get_next_outbound_watch_operation
        inner.outbound_watch_manager.update_per_node_states(cur_ts);

        // Iterate all outbound watches and determine what work needs doing if any
        for outbound_watch in inner.outbound_watch_manager.outbound_watches.values_mut() {
            // Get next work on watch and queue it if we have something to do
            if let Some(op_fut) = self.get_next_outbound_watch_operation(
                outbound_watch.record_key(),
                cur_ts,
                outbound_watch,
            ) {
                self.background_operation_processor.add_future(op_fut);
            };
        }

        // Iterate all queued change inspections and do them
        for (k, v) in inner.outbound_watch_manager.needs_change_inspection.drain() {
            // Get next work on watch and queue it if we have something to do
            let op_fut = self.get_change_inspection_operation(k, v);
            self.background_operation_processor.add_future(op_fut);
        }

        Ok(())
    }
}

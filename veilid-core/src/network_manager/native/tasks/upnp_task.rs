use super::*;

impl Network {
    #[cfg_attr(feature = "instrument", instrument(parent = None, level = "trace", target = "net", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub(super) async fn upnp_task_routine(
        &self,
        _stop_token: StopToken,
        _l: Timestamp,
        _t: Timestamp,
    ) -> EyreResult<()> {
        if !self.igd_manager.tick().await? {
            veilid_log!(self info "upnp failed, restarting local network");
            let mut inner = self.inner.lock();
            inner.network_needs_restart = true;
        }

        Ok(())
    }
}

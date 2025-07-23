use java_spaghetti::Global;
use log::{debug, warn};

use crate::bindings::android::bluetooth::BluetoothSocket;

/// Utility struct to close the channel on drop.
pub struct L2capCloser {
    pub channel: Global<BluetoothSocket>,
}

impl L2capCloser {
    pub fn close(&self) {
        self.channel.vm().with_env(|env| {
            let channel = self.channel.as_local(env);
            match channel.close() {
                Ok(()) => debug!("l2cap channel closed"),
                Err(e) => warn!("failed to close channel: {e:?}"),
            };
        });
    }
}

impl Drop for L2capCloser {
    fn drop(&mut self) {
        self.close()
    }
}

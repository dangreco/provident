// SPDX-License-Identifier: GPL-3.0-only

use crate::models::common::Utility;

use super::Account;
use crate::Result;

impl Account {
    /// Utilities metered for this account: the distinct [`Utility`] of each of its
    /// [`meters`](Self::meters), in the order first seen.
    #[maybe_async::maybe_async]
    pub async fn utilities(&self) -> Result<Vec<Utility>> {
        let meters = self.meters().await?;
        let mut utilities = Vec::new();
        for meter in meters {
            if !utilities.contains(&meter.utility) {
                utilities.push(meter.utility);
            }
        }
        Ok(utilities)
    }
}

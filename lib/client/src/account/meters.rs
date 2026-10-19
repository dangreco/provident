// SPDX-License-Identifier: GPL-3.0-only

use std::collections::VecDeque;

use provident_http::Request;
use provident_models::api::internal::meter_tree::{Meter, TreeNode, get_children, root_nodes};

use super::Account;
use crate::Result;

impl Account {
    /// Every metering point visible to this account, in tree order.
    ///
    /// Reads the whole meter tree: the top two levels in one request, then one request per
    /// group the server left unexpanded.
    #[maybe_async::maybe_async]
    pub async fn meters(&self) -> Result<Vec<Meter>> {
        let request = Request::get(root_nodes::PATH)
            .query(&root_nodes::RootNodesRequest::default())?
            .build();
        let mut pending: VecDeque<TreeNode> = self
            .http
            .send::<root_nodes::RootNodesResponse>(request)
            .await?
            .into();

        let mut meters = Vec::new();
        while let Some(node) = pending.pop_front() {
            if node.meter_id().is_some() {
                meters.push(Meter::try_from(&node)?);
            } else if let Some(group_id) = node.group_id().filter(|_| node.has_unloaded_children())
            {
                let request = Request::get(get_children::PATH)
                    .query(&get_children::GetChildrenRequest { group_id })?
                    .build();
                let children = self
                    .http
                    .send::<get_children::GetChildrenResponse>(request)
                    .await?;
                pending.extend(children);
            }
        }
        Ok(meters)
    }
}

// SPDX-License-Identifier: GPL-3.0-only

//! Python arguments accepted where the Rust API takes node ids.

use provident::models::api::internal::usage::MeterList;
use provident::models::common::NodeId;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyString;

use crate::models::{Meter, TreeNode};

/// A meter-tree node: a [`Meter`], a [`TreeNode`], or a node-id string (`MP:1`, `GROUP:2`).
pub struct Node(pub NodeId);

impl<'a, 'py> FromPyObject<'a, 'py> for Node {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(meter) = obj.cast::<Meter>() {
            return Ok(Self(meter.get().0.node_id()));
        }
        if let Ok(node) = obj.cast::<TreeNode>() {
            return Ok(Self(node.get().0.id));
        }
        if let Ok(text) = obj.cast::<PyString>() {
            let text = text.to_str()?;
            return text
                .parse()
                .map(Self)
                .map_err(|e| PyValueError::new_err(e.to_string()));
        }
        Err(PyTypeError::new_err(format!(
            "expected a Meter, TreeNode, or node id string ('MP:<n>' / 'GROUP:<n>'), got {}",
            obj.get_type().name()?
        )))
    }
}

/// One [`Node`] or an iterable of them, for `meterlist=`.
pub struct Nodes(pub MeterList);

impl<'a, 'py> FromPyObject<'a, 'py> for Nodes {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        // Strings are iterable too; treat them as a single node id.
        if obj.is_instance_of::<PyString>()
            || obj.is_instance_of::<Meter>()
            || obj.is_instance_of::<TreeNode>()
        {
            return Ok(Self(MeterList::from(obj.extract::<Node>()?.0)));
        }
        let nodes = obj
            .try_iter()
            .map_err(|_| {
                PyTypeError::new_err(
                    "expected a Meter, TreeNode, node id string, or an iterable of them",
                )
            })?
            .map(|item| Ok(item?.extract::<Node>()?.0))
            .collect::<PyResult<_>>()?;
        Ok(Self(MeterList(nodes)))
    }
}

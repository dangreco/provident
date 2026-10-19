// SPDX-License-Identifier: GPL-3.0-only

use serde::Serialize;

/// No parameters; sent as an empty `{}` body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct GetUtilitiesRequest {}

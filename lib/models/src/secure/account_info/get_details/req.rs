// SPDX-License-Identifier: GPL-3.0-only

use serde::Serialize;

/// No parameters; GET page methods send no body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct GetDetailsRequest;

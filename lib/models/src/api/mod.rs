// SPDX-License-Identifier: GPL-3.0-only

//! `/api/`: plain JSON REST endpoints. Unlike page methods, responses have no `{"d": ...}`
//! envelope and parameters go in the query string. Require a signed-in session (401 otherwise).

pub mod internal;

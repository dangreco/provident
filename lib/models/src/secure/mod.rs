// SPDX-License-Identifier: GPL-3.0-only

//! `/secure/`: pages that require a signed-in session (`.ASPXAUTH` cookie); without one
//! page methods answer 401.

pub mod dashboard;

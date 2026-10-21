// SPDX-License-Identifier: GPL-3.0-only

//! `pemi.com`, Provident's corporate site: published rates and fees for an account location.
//! A different host from the metering portal, with its own session (`PHPSESSID`) and login
//! that takes the portal's credentials with its fields swapped (username → `LocationID`,
//! password → `CustomerNumber`).

pub mod do_rates_login;
pub mod rates_and_fees;

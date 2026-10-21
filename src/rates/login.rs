// SPDX-License-Identifier: GPL-3.0-only

//! Published rates and fees for the account's location, from Provident's corporate site.
//!
//! Wire endpoints: `crate::models::pemi` (`do-rates-login`, `rates-and-fees`).

use crate::http::{Request, Site};
use crate::models::pemi::do_rates_login;
use crate::models::pemi::rates_and_fees;
use crate::models::pemi::rates_and_fees::parsed::Rates;

use super::Rates as RatesArea;
use crate::{Error, Result};

impl RatesArea {
    /// Signs in to the rates site (`do-rates-login`) with the metering portal's credentials
    /// (portal username → `LocationID`, portal password → `CustomerNumber`). The `PHPSESSID`
    /// cookie set on success authorizes [`RatesArea::rates`] on this client.
    ///
    /// # Errors
    /// [`Error::LoginRejected`] when the location id is refused. The site accepts any
    /// non-empty customer number, so a wrong password still "succeeds" here and simply yields
    /// that location's public rates.
    #[maybe_async::maybe_async]
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let request = Request::post(do_rates_login::PATH)
            .form(&[("CustomerNumber", password), ("LocationID", username)])?
            .site(Site::Rates)
            .build();
        let response = self
            .http
            .send::<do_rates_login::RatesLoginResponse>(request)
            .await?;
        if response.accepted() {
            Ok(())
        } else {
            Err(Error::LoginRejected {
                message: (!response.message.is_empty()).then_some(response.message),
            })
        }
    }

    /// The location's published rates and fees (`rates-and-fees`); requires
    /// [`RatesArea::login`] first. Layout changes on the page degrade to fewer (`None`) fields,
    /// never an error.
    ///
    /// # Errors
    /// [`Error::RatesSessionRequired`] when the site serves its sign-in form instead (no or
    /// expired session).
    #[maybe_async::maybe_async]
    pub async fn rates(&self) -> Result<Rates> {
        let request = Request::get(rates_and_fees::PATH).site(Site::Rates).build();
        Rates::parse_page(&self.http.send_text(request).await?).ok_or(Error::RatesSessionRequired)
    }
}

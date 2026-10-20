# Changelog

## [0.2.0](https://github.com/dangreco/provident/compare/v0.1.0...v0.2.0) (2026-10-20)


### ⚠ BREAKING CHANGES

* crate renamed from provident_client to provident; provident_http and provident_models are now modules provident::http and provident::models.

### Features

* add http, models, client crates with CI ([d3b9ebd](https://github.com/dangreco/provident/commit/d3b9ebd43471e6e24275b9c7e167c99c965c9d66))
* **client:** add account.utilities() for GetUtilities ([#3](https://github.com/dangreco/provident/issues/3)) ([ce3256f](https://github.com/dangreco/provident/commit/ce3256f37bedb7a6fb43409b7c1959aedb86dfd5))
* **client:** usage series/reports and account details/API key ([#5](https://github.com/dangreco/provident/issues/5)) ([b710243](https://github.com/dangreco/provident/commit/b71024386d670790f4ff35a5ee71395ff085d5f0))
* crates.io publish ([#6](https://github.com/dangreco/provident/issues/6)) ([5c3e8a7](https://github.com/dangreco/provident/commit/5c3e8a740dd8c1726f9b8cceb5acb34bab4eea8b))


### Bug Fixes

* **ci:** don't export NIX_ENFORCE_PURITY from dev shell ([12989ea](https://github.com/dangreco/provident/commit/12989ead2f9a9a895271cb5465d38ac281844243))
* **ci:** rename CARGO env var that broke cargo fmt ([9c56bf7](https://github.com/dangreco/provident/commit/9c56bf77e780d1bedf58d3ddf30dd1ccebb6f2f4))


### Documentation

* add README ([d1aa55c](https://github.com/dangreco/provident/commit/d1aa55c01fe55435cf8ef8b3fa146d5f5ef6b311))

# Workspace tasks. `make help` lists targets.
#
# MODE selects the client flavour for build/check/test targets: `async` (default) or `blocking`.
# The two are mutually exclusive, so `blocking` swaps the default feature on the crates that
# have it (http, client, tests); models has no such feature and is unaffected.

CARGO ?= cargo
MODE ?= async

ifeq ($(MODE),async)
  MODE_FLAGS :=
else ifeq ($(MODE),blocking)
  MODE_FLAGS := -p provident_http -p provident_client -p provident_tests \
                --no-default-features --features blocking
else
  $(error MODE must be `async` or `blocking`, got `$(MODE)`)
endif

# `async` covers the whole workspace; `blocking` only the crates that have the feature.
PACKAGES := $(if $(MODE_FLAGS),$(MODE_FLAGS),--workspace)

.DEFAULT_GOAL := help
.PHONY: help build check lint fmt fmt-check test test-unit test-e2e

help: ## List targets
	@awk 'BEGIN {FS = ":.*## "} /^[a-z0-9-]+:.*## / {printf "  %-10s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@printf '\n  MODE=async|blocking (default async)\n'

build: ## Compile all crates
	$(CARGO) build $(PACKAGES) --all-targets

check: ## Fast type-check (no codegen)
	$(CARGO) check $(PACKAGES) --all-targets

lint: ## Clippy with warnings denied
	$(CARGO) clippy $(PACKAGES) --all-targets -- -D warnings

fmt: ## Format Rust, TOML, and sort Cargo.toml dependencies
	$(CARGO) fmt --all
	cargo-sort --workspace
	taplo fmt

fmt-check: ## Fail if anything is unformatted
	$(CARGO) fmt --all --check
	cargo-sort --workspace --check
	taplo fmt --check

test: test-unit ## Unit tests (alias)

test-unit: ## Offline unit + doc tests; e2e tests are skipped
	$(CARGO) test $(PACKAGES)

test-e2e: ## End-to-end tests against the real endpoint (needs PROVIDENT_USERNAME/PASSWORD in env or .env)
	$(CARGO) test $(if $(MODE_FLAGS),$(MODE_FLAGS),-p provident_tests) -- --ignored

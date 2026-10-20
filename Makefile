# Workspace tasks. `make help` lists targets.
#
# MODE selects the client flavour for build/check/test targets: `async` (default) or `blocking`.
# The two are mutually exclusive, so `blocking` swaps the default feature on every package.
# The Python bindings (`python/`) are built on the async client, so only `async` includes them.

CARGO ?= cargo
MODE ?= async

ifeq ($(MODE),async)
  PACKAGES := --workspace
else ifeq ($(MODE),blocking)
  PACKAGES := --workspace --exclude provident_python --no-default-features --features blocking
else
  $(error MODE must be `async` or `blocking`, got `$(MODE)`)
endif

# Python bindings: a venv in python/.venv that sees the interpreter's own packages (pytest,
# mypy from the dev shell), with the extension installed into it by `maturin develop`.
PYTHON ?= python3
PY_DIR := $(CURDIR)/python
PY_VENV := $(PY_DIR)/.venv
PY := $(PY_VENV)/bin/python

# COVERAGE=1 runs test targets instrumented (cargo-llvm-cov), accumulating profiles in
# target/llvm-cov-target so unit + e2e runs merge into one report (`make coverage-report`).
# `show-env` uses CARGO_TARGET_DIR as-is while `report`/`clean` append `llvm-cov-target`
# themselves, hence the two directories.
COVERAGE ?=
COV_TARGET := $(CURDIR)/target/llvm-cov-target
COV_REPORT := CARGO_TARGET_DIR=$(CURDIR)/target cargo llvm-cov
ifeq ($(COVERAGE),1)
  # Exports RUSTC_WRAPPER/LLVM_PROFILE_FILE for the cargo command that follows, in one shell.
  COV_ENV = export CARGO_TARGET_DIR=$(COV_TARGET) && eval "$$(cargo llvm-cov show-env --sh)" &&
else
  COV_ENV =
endif

.DEFAULT_GOAL := help
.PHONY: help build check lint fmt fmt-check test test-unit test-e2e coverage-clean coverage-report
.PHONY: py-dev py-lint py-fmt py-test py-test-e2e py-wheel

help: ## List targets
	@awk 'BEGIN {FS = ":.*## "} /^[a-z0-9-]+:.*## / {printf "  %-16s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@printf '\n  MODE=async|blocking (default async)   COVERAGE=1 instruments test targets\n'

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
	$(COV_ENV) $(CARGO) test $(PACKAGES)

test-e2e: ## End-to-end tests against the real endpoint (needs PROVIDENT_USERNAME/PASSWORD in env or .env)
	$(COV_ENV) $(CARGO) test $(PACKAGES) -- --ignored

coverage-clean: ## Drop accumulated coverage profiles
	$(COV_REPORT) clean --workspace

coverage-report: ## Merge COVERAGE=1 test runs into target/coverage.lcov and print a summary
	$(COV_REPORT) report --lcov --output-path target/coverage.lcov
	$(COV_REPORT) report --summary-only

$(PY):
	$(PYTHON) -m venv --system-site-packages $(PY_VENV)

py-dev: $(PY) ## Build the Python extension (debug) into python/.venv
	cd $(PY_DIR) && VIRTUAL_ENV=$(PY_VENV) maturin develop

py-lint: py-dev ## Ruff, mypy, and a stub check against the built extension
	cd $(PY_DIR) && ruff check . && ruff format --check .
	cd $(PY_DIR) && $(PY) -m mypy
	cd $(PY_DIR) && $(PY) -m mypy.stubtest provident._provident

py-fmt: ## Format Python sources
	cd $(PY_DIR) && ruff check --fix . && ruff format .

py-test: py-dev ## Offline Python tests (local fake portal)
	cd $(PY_DIR) && $(PY) -m pytest

py-test-e2e: py-dev ## Python tests against the real endpoint (same env as test-e2e)
	cd $(PY_DIR) && $(PY) -m pytest -m e2e

py-wheel: ## Release wheel + sdist for this machine into target/wheels
	cd $(PY_DIR) && maturin build --release --sdist

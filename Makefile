NIGHTLY_RUST_VERSION = "nightly"
TAPLO_CLI_VERSION = "0.9.3"

.PHONY: help
help: ## Ask for help!
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; \
		{printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the project in debug mode
	cargo build

.PHONY: build-release
build-release: ## Build the project in release mode
	cargo build --release

.PHONY: check
check: ## Check code for compilation errors
	cargo check --all-targets

.PHONY: check-format
check-format: ## Check code formatting
	cargo +$(NIGHTLY_RUST_VERSION) fmt -- --check
	taplo format --check

.PHONY: format
format: ## Format code using rustfmt and taplo
	cargo +$(NIGHTLY_RUST_VERSION) fmt
	taplo format

.PHONY: lint
lint: ## Run linter (clippy)
	cargo clippy --all-targets -- -D warnings

.PHONY: lint-beta
lint-beta: ## Run linter (clippy) using beta Rust
	cargo +beta clippy --all-targets -- -D warnings

.PHONY: lint-shell
lint-shell: ## Lint shell scripts using shellcheck
	shellcheck .github/scripts/*.sh

.PHONY: test
test: ## Run tests
	cargo test

.PHONY: test-release
test-release: ## Run tests in release mode
	cargo test --release

.PHONY: test-doc
test-doc: ## Test documentation examples
	cargo test --doc

.PHONY: bench
bench: ## Run benchmarks
	cargo bench

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

.PHONY: setup
setup: setup-taplo ## Setup development environment
	rustup toolchain install $(NIGHTLY_RUST_VERSION)
	rustup toolchain install beta
	cargo install cargo-deny

.PHONY: setup-taplo
setup-taplo: ## Install taplo TOML formatter
	@if taplo --version 2>/dev/null | \
		grep -q ${TAPLO_CLI_VERSION}; then \
		echo "taplo ${TAPLO_CLI_VERSION} already installed"; \
	else \
		cargo +nightly install taplo-cli \
			--version ${TAPLO_CLI_VERSION} --force; \
	fi

.PHONY: deny
deny: ## Run cargo deny checks
	cargo deny check

# Build automation for API Gateway
.PHONY: build test run clean

## Go-specific (if using Go)
GO_CMD=go
GO_BUILD=$(GO_CMD) build
GO_TEST=$(GO_CMD) test -v ./...

## Rust-specific (if using Rust)
CARGO_CMD=cargo
CARGO_BUILD=$(CARGO_CMD) build --release
CARGO_TEST=$(CARGO_CMD) test

# Common targets
build:
	@echo "Building API Gateway..."
	@$(GO_BUILD) -o bin/gateway ./cmd/gateway  # Go example
	# @$(CARGO_BUILD)  # Uncomment for Rust

test:
	@echo "Running tests..."
	@$(GO_TEST) -coverprofile=coverage.out  # Go
	# @$(CARGO_TEST)  # Rust

run:
	@echo "Starting gateway in dev mode..."
	@$(GO_CMD) run ./cmd/gateway --dev  # Go
	# @$(CARGO_CMD) run --features "dev"  # Rust

clean:
	@echo "Cleaning build artifacts..."
	@rm -rf bin/ coverage.out target/

docker-build:
	docker build -t api-gateway:latest .

lint:
	golangci-lint run  # Go
	# cargo clippy --all-targets  # Rust
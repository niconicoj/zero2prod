#!/bin/sh

# Function to check if a command is available
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if sqlx-cli is installed, if not, install it
if ! command_exists sqlx; then
    echo "sqlx-cli not found. Installing..."
    cargo install sqlx-cli
fi

# Check if sqlx.json is valid
echo "Checking the sqlx.json..."
if ! cargo sqlx prepare --workspace --check; then
    exit 1
fi

# Run the test suite
echo "Running tests..."
cargo test

# Check if tests passed
if [ $? -ne 0 ]; then
    echo "Error: Test suite failed."
    exit 1
fi

echo "Pre-commit checks passed."
exit 0

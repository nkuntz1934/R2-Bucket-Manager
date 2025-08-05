#!/bin/bash

echo "Testing R2 Storage Manager"
echo "=========================="

# Check if binaries exist
if [ ! -f "./target/debug/rust-r2-cli" ]; then
    echo "Building debug binaries..."
    cargo build
fi

echo ""
echo "1. Testing CLI help:"
./target/debug/rust-r2-cli --help

echo ""
echo "2. Testing with missing credentials (should show error):"
./target/debug/rust-r2-cli list 2>&1 | head -5

echo ""
echo "3. Checking GUI binary:"
if [ -f "./target/debug/rust-r2-gui" ]; then
    echo "GUI binary exists ✓"
else
    echo "GUI binary not found ✗"
fi

echo ""
echo "To test with real credentials, set these environment variables:"
echo "  export R2_ACCESS_KEY_ID='your_key'"
echo "  export R2_SECRET_ACCESS_KEY='your_secret'"
echo "  export R2_ACCOUNT_ID='your_account_id'"
echo "  export R2_BUCKET_NAME='your_bucket'"
echo ""
echo "Then run:"
echo "  ./target/debug/rust-r2-cli list"
echo "  ./target/debug/rust-r2-gui"
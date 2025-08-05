#!/bin/bash

echo "Testing folder upload with encryption..."

# Upload each file in the test folder with encryption
for file in $(find test_folder -type f); do
    # Get relative path
    rel_path=${file#test_folder/}
    # Create object key with folder prefix
    object_key="test_upload_folder/$rel_path"
    
    echo "Uploading $file as $object_key..."
    ./target/release/rust-r2-cli --config config_test.json upload "$file" "$object_key" --encrypt
done

echo ""
echo "Listing uploaded files..."
./target/release/rust-r2-cli --config config_test.json list --prefix "test_upload_folder/" 2>&1 | grep -v INFO

echo ""
echo "Testing download and decryption of a file..."
./target/release/rust-r2-cli --config config_test.json download "test_upload_folder/documents/test.txt" --output downloaded_test.txt --decrypt 2>&1 | grep -v INFO

echo "Content of downloaded file:"
cat downloaded_test.txt

# Cleanup
rm -f downloaded_test.txt
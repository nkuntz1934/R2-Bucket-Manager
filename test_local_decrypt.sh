#\!/bin/bash

echo "Testing local decryption of Wegmans_List.xlsx.gpg..."

# We already have the file locally
if [ \! -f "Wegmans_List.xlsx.gpg" ]; then
    echo "Error: Wegmans_List.xlsx.gpg not found"
    exit 1
fi

echo "File size: $(wc -c < Wegmans_List.xlsx.gpg) bytes"
echo "First 100 bytes (hex):"
xxd -l 100 Wegmans_List.xlsx.gpg

# Try to decrypt with GPG directly
echo -e "\nTrying GPG command-line decryption..."
gpg --output Wegmans_test.xlsx --decrypt Wegmans_List.xlsx.gpg 2>&1

if [ $? -eq 0 ]; then
    echo "✓ GPG decryption succeeded"
    ls -la Wegmans_test.xlsx
    file Wegmans_test.xlsx
    echo "First 16 bytes of decrypted file:"
    xxd -l 16 Wegmans_test.xlsx
else
    echo "✗ GPG decryption failed"
fi

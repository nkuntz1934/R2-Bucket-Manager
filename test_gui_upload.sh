#!/bin/bash

echo "Testing GUI upload tracking..."

# Create a test file
echo "Test content $(date)" > test_upload.txt

# Start GUI and capture debug output
echo "Starting GUI..."
./target/release/rust-r2-gui 2>&1 | tee gui_debug.log &
GUI_PID=$!

echo "GUI started with PID: $GUI_PID"
echo "Please perform the following in the GUI:"
echo "1. Go to Config tab and load config_test.json"
echo "2. Test the connection"
echo "3. Go to Upload tab"
echo "4. Upload test_upload.txt"
echo "5. Check if it appears in Recent Uploads"
echo ""
echo "Press Enter when done testing..."
read

# Kill the GUI
kill $GUI_PID 2>/dev/null

# Check debug output
echo ""
echo "Debug output related to uploads:"
grep -E "DEBUG.*upload|Recent" gui_debug.log || echo "No debug output found"

# Cleanup
rm -f test_upload.txt gui_debug.log
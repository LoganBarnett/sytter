#!/usr/bin/env bash
set -euo pipefail

# Manual test for device property extraction
# Usage: ./tests/manual_device_properties_test.sh
#
# This will start sytter and wait for you to plug in a USB device.
# When a device is connected, it will print the device properties.

echo "==================================================================="
echo "Device Property Extraction Test"
echo "==================================================================="
echo ""
echo "This test will monitor for USB device connections."
echo "When you plug in a USB device, it will display the device properties."
echo ""
echo "To run the test:"
echo "  1. Keep this terminal visible"
echo "  2. Plug in a USB device (keyboard, mouse, flash drive, etc.)"
echo "  3. Watch for device properties to be printed"
echo "  4. Press Ctrl+C when done"
echo ""
echo "==================================================================="
echo ""

# Set up test output file
export SYTTER_TEST_OUTPUT="/tmp/sytter_device_test_$$.txt"
rm -f "$SYTTER_TEST_OUTPUT"

# Run sytter with the device properties test config
./target/debug/sytter \
  --sytters-path tests/fixtures/test_device_properties.toml \
  --log-level info

# Cleanup
rm -f "$SYTTER_TEST_OUTPUT"

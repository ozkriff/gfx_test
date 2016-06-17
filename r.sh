#!/bin/sh

set -e

# NOTE:
# adb shell ip -f inet addr show wlan0
# adb tcpip 5555
# adb connect 192.168.1.36:5555

cargo apk install
adb logcat -c
echo [START]
adb shell am start -n rust.gfx_test/rust.gfx_test.MainActivity
adb logcat | grep 'Rust\|DEBUG\|gfx_test'

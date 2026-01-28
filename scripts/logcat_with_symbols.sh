#!/bin/bash

NDK_STACK=$(ls -1 $ANDROID_HOME/ndk/*/ndk-stack | head -n1)
if [ -z "$NDK_STACK" ]; then
    echo "No ndk-stack found"
    exit 1
fi

TARGET_DIR=$(cargo metadata --format-version=1 | jq -r .target_directory)

adb logcat $@ | $NDK_STACK -sym $TARGET_DIR/aarch64-linux-android/debug/
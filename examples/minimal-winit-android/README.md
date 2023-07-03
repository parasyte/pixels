# Hello Android

![Hello Android](../../img/minimal-winit-android.png)

Minimal example to run on android using `winit` with `android-native-activity` feature

## Running
```
export ANDROID_HOME="path/to/sdk"
export ANDROID_NDK_HOME="path/to/ndk"

rustup target add aarch64-linux-android
cargo install cargo-apk
```
Connect your Android device via USB cable to your computer in debug mode and run the following command
```
cargo apk run
```

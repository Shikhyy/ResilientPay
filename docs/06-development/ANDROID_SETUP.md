# Android Development Setup

This document specifies the exact toolchain required to compile and verify the Android hardware-backed integration for ResilientPay.

## Required Toolchain Versions
- **JDK**: Java 17 (Required by Gradle 8.x and Jetpack Compose compiler)
- **Android Gradle Plugin (AGP)**: 8.2.+
- **Android SDK**: API 34 (Compile SDK), Min SDK 26 (Required for `java.security.Signature` Ed25519 support)
- **Android NDK**: `26.1.10909125` (LTS) — Required for linking the Rust JNI library.
- **Rust Toolchain**: `1.70+`

## Rust Android Targets
The native SDK must be compiled for the following ABIs to cover physical devices and modern emulators:
- `aarch64-linux-android` (arm64-v8a - physical devices)
- `x86_64-linux-android` (x86_64 - emulators)
*(Note: `armeabi-v7a` and `i686` are deprecated for this project's security constraints and are NOT required).*

Install via rustup:
```bash
rustup target add aarch64-linux-android x86_64-linux-android
```

## Cargo NDK Setup
We use `cargo-ndk` to orchestrate the build:
```bash
cargo install cargo-ndk
```
Compilation command (executed via Gradle):
```bash
cargo ndk -t arm64-v8a -t x86_64 -o ../../resilientpay-android/app/src/main/jniLibs build --release
```

## Verification Limits
Because CI/local environments may lack physical Trusted Execution Environments (TEEs), the Android test suite must distinguish between:
1. **Host tests** (Rust mock Keystore)
2. **Emulator tests** (Software-backed Android Keystore)
3. **Physical Device tests** (Hardware-backed StrongBox Keystore)

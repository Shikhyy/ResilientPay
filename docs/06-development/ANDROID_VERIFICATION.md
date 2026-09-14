# Android Verification

## Toolchain Required
- **JDK**: 17
- **Gradle**: 8.5
- **AGP**: 8.2.+
- **Android SDK**: API 34
- **NDK**: 26.1.10909125
- **Rust**: 1.93.1
- **UniFFI**: 0.28.3

## Test Implementation
The following Keystore capabilities have been integrated in `KeystoreIntegrationTest.kt`:
- **Native Load**: Verifies `libresilientpay_core.so` loaded via UniFFI wrapper.
- **Keystore Signing**: Uses standard `java.security.Signature` instance with Ed25519 payload.
- **Hardware Backing**: Checks `KeyInfo.isInsideSecureHardware`.
- **Concurrency**: Exercises threaded Keystore signing requests with `CountDownLatch`.
- **Invalidation**: Verifies non-existent key requests correctly bubble up `FfiException` subclasses.

## Limitations
Currently **BLOCKED** on local testing.
- The headless CI sandbox cannot efficiently launch an ARM64/X86_64 emulator or execute `connectedAndroidTest`.
- The Android SDK provisioning via `sdkmanager` times out due to bandwidth limits inside the sandbox.
- True hardware-backed Keystore tests must be executed manually on a physical Pixel or Samsung device, or via a remote CI action running `macos-latest` GitHub runner (which provides HAXM acceleration for the emulator).

## Decision
M7 Verification Status: **BLOCKED** (Pending remote CI execution).

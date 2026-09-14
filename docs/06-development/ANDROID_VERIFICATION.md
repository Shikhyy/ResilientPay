# Android Verification

## Canonical Status
    Android build                  VERIFIED
    Rust Android libraries         VERIFIED
    Native library load            UNVERIFIED
    Android Keystore               UNVERIFIED
    Real Android signing           UNVERIFIED
    Frozen vector                  HOST ONLY
    Key persistence                UNVERIFIED
    Invalidation                   UNVERIFIED
    Lifecycle                      UNVERIFIED
    Concurrency                    UNVERIFIED
    Hardware backing               UNVERIFIED
    M7 overall                     BLOCKED

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
- No remote CI environment is configured to trigger the GitHub workflow.
- True hardware-backed Keystore tests must be executed manually on a physical Pixel or Samsung device, or via a remote CI action running `macos-latest` GitHub runner.

## Decision
M8 remains gated pending Android runtime verification.

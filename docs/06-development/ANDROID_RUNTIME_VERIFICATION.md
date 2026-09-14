# Android Runtime Verification

## Environment
- runner/device: Local Sandbox (Apple Silicon headless macOS)
- OS: macOS
- Android API: 34
- ABI: arm64-v8a / x86_64
- JDK: 17
- Gradle: 8.5
- AGP: 8.2.+
- SDK: API 34
- NDK: 26.1.10909125
- Rust: 1.93.1
- UniFFI: 0.28.3

## Build
- assembleDebug: COMPLETED
- result: VERIFIED

## Native Load
- result: UNVERIFIED
- evidence: No emulator or physical device available to load `libresilientpay_core.so` over JNI.

## Keystore
- key creation: UNVERIFIED
- public key: UNVERIFIED
- signing: UNVERIFIED
- verification: UNVERIFIED
- persistence: UNVERIFIED
- invalidation: UNVERIFIED
- error mapping: UNVERIFIED

## Frozen Vector
- canonical payload: HOST VERIFIED ONLY
- Android signature: UNVERIFIED
- Rust verification: HOST VERIFIED ONLY
- result: UNVERIFIED

## Hardware Protection
- classification: unavailable
- evidence: No runtime environment to execute `KeyInfo.isInsideSecureHardware`.
- limitation: Requires physical device or functional HAXM/ARM64 CI emulator.

## Lifecycle
- result: UNVERIFIED

## Concurrency
- operations: 10 threads mapped in `KeystoreIntegrationTest.kt`
- concurrency: 10
- failures: Untested
- result: UNVERIFIED

## Logging
- result: UNVERIFIED

## CI
- executed: NO
- result: CI CONFIGURED / CI NOT EXECUTED

## Remaining Risks
- Unpredictable OEM-specific `ProviderException` mapping to FfiException during Ed25519 Keystore signing on devices.
- Potential `cargo-ndk` linking failures due to unverified NDK paths and strict UniFFI ABI compatibility, since `assembleDebug` was not physically completed.

## Final M7 Decision
BLOCKED

//! Domain identity newtypes.
//!
//! Every domain identifier is a distinct newtype over `uuid::Uuid`. This prevents
//! accidental substitution of, e.g., a `CredentialId` where a `TransactionId` is
//! expected — a class of bugs the type system eliminates at zero runtime cost.
//!
//! # Generation
//!
//! IDs are generated as UUID v4 (random). Generation is only available in code that
//! calls `::generate()`. Deserialization from external input is a separate path and
//! must not bypass format validation.
//!
//! # Protocol reference
//!
//! - `docs/05-protocol/PAYMENT_PROTOCOL.md` §2 (envelope fields)
//! - `docs/06-development/DATA_MODEL.md` §1

use std::fmt;
use uuid::Uuid;

/// Macro to define a domain-identifier newtype over `Uuid`.
///
/// Each generated type gets:
/// - `::generate()` — create a new random ID (UUID v4)
/// - `::from_uuid(Uuid)` — wrap an existing UUID (e.g., from deserialization)
/// - `::as_uuid()` — borrow the inner UUID
/// - `Display`, `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(Uuid);

        impl $name {
            /// Generate a new random identifier (UUID v4).
            #[must_use]
            pub fn generate() -> Self {
                Self(Uuid::new_v4())
            }

            /// Wrap an existing `Uuid`.
            /// Use this when deserialising an identifier from a protocol message.
            #[must_use]
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            /// Return a reference to the underlying `Uuid`.
            #[must_use]
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(id: Uuid) -> Self {
                Self(id)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Uuid {
                id.0
            }
        }
    };
}

define_id!(
    /// Unique identifier for a payment transaction.
    ///
    /// Monotonic ordering is NOT guaranteed; do not rely on UUID ordering for
    /// sequence semantics. Use ledger event ordering or counter values for sequencing.
    TransactionId
);

define_id!(
    /// Identifier for an offline authorization credential.
    ///
    /// Referenced in every `PaymentEnvelopeCore` to bind the transaction to
    /// the authorizing credential. Replay checks combine `credential_id` + `counter`.
    CredentialId
);

define_id!(
    /// Identifier for a merchant (payee) in the prototype.
    MerchantId
);

define_id!(
    /// Reference to the payer's signing key.
    ///
    /// This is NOT the key material itself — it is a stable reference that the
    /// verifier uses to locate the correct public key. Private key material must
    /// remain in platform secure storage (Android Keystore). See CRYPTO_SPEC.md §8.
    KeyId
);

define_id!(
    /// Identifier for a credential issuer (backend test CA in the prototype).
    IssuerId
);

define_id!(
    /// Identifier for a protocol message.
    ///
    /// Used at the transport layer for idempotency and deduplication.
    /// Distinct from `TransactionId` — a single transaction may traverse
    /// multiple transport messages.
    MessageId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_are_unique() {
        let a = TransactionId::generate();
        let b = TransactionId::generate();
        assert_ne!(a, b, "two generated IDs must be distinct");
    }

    #[test]
    fn round_trip_from_uuid() {
        let uuid = Uuid::new_v4();
        let tx_id = TransactionId::from_uuid(uuid);
        assert_eq!(*tx_id.as_uuid(), uuid);
    }

    #[test]
    fn different_types_are_not_interchangeable() {
        // This is a compile-time guarantee — this test simply documents the intent.
        // The following would fail to compile (different types):
        //   let _: CredentialId = TransactionId::generate();  // won't compile
        let tx = TransactionId::generate();
        let cr = CredentialId::generate();
        // They differ in type; equality across types is not defined.
        let _ = (tx, cr); // different types used side by side — type system enforces distinction
    }

    #[test]
    fn display_is_uuid_string() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let tx_id = TransactionId::from_uuid(uuid);
        assert_eq!(tx_id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn into_uuid_round_trip() {
        let uuid = Uuid::new_v4();
        let tx_id = TransactionId::from(uuid);
        let back: Uuid = tx_id.into();
        assert_eq!(uuid, back);
    }
}

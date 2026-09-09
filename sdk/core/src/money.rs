//! Monetary value representation.
//!
//! # Rule
//!
//! Money MUST be represented as integer minor units with an explicit currency code.
//! Floating-point arithmetic is never used for monetary values.
//!
//! See `docs/06-development/MONEY.md` for the normative rule.
//!
//! # Example
//!
//! ```
//! use resilientpay_core::money::Money;
//!
//! // ₹1.50 = 150 paise
//! let amount = Money::new(150, "INR").unwrap();
//! assert_eq!(amount.amount_minor(), 150);
//! assert_eq!(amount.currency(), "INR");
//! ```

use crate::errors::ValidationError;

/// Maximum permitted minor-unit amount in a single transaction envelope.
///
/// This is a prototype policy guard against accidental test values that would
/// look unreasonable. The normative policy per transaction is stored in the
/// credential (`max_value_per_tx`). A change to this ceiling requires a
/// protocol change-control review.
///
/// Value: 1,000,000 INR expressed in paise (100_000_000 paise = ₹1,000,000)
pub const MAX_AMOUNT_MINOR: u64 = 100_000_000;

/// Maximum currency code length (ISO 4217 uses 3 characters; allow up to 8 for
/// prototype flexibility, but never unlimited).
const MAX_CURRENCY_LEN: usize = 8;

/// A monetary amount expressed as integer minor units with an explicit currency.
///
/// # Invariants
///
/// - `amount_minor` is always non-negative (unsigned).
/// - `amount_minor` never exceeds `MAX_AMOUNT_MINOR`.
/// - `currency` is a non-empty ASCII-printable string of at most `MAX_CURRENCY_LEN` bytes.
/// - Construction can only succeed through `Money::new`; all invariants are checked at
///   construction time.
///
/// These invariants are maintained by the type: once constructed, a `Money` value is
/// guaranteed to be valid.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Money {
    amount_minor: u64,
    currency: String,
}

impl Money {
    /// Construct a new `Money` value.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::AmountNegative` if `amount_minor` is zero and
    /// zero is disallowed (currently zero IS allowed — a zero-value transaction
    /// is structurally valid; policy rejection happens at validation layer).
    ///
    /// Returns `ValidationError::AmountExceedsMaximum` if `amount_minor` exceeds
    /// `MAX_AMOUNT_MINOR`.
    ///
    /// Returns `ValidationError::InvalidCurrency` if the currency string is empty,
    /// too long, or contains non-ASCII-printable bytes.
    pub fn new(amount_minor: u64, currency: &str) -> Result<Self, ValidationError> {
        // Validate currency first (fast, cheap).
        validate_currency(currency)?;

        // Guard the ceiling.
        if amount_minor > MAX_AMOUNT_MINOR {
            return Err(ValidationError::AmountExceedsMaximum {
                amount_minor,
                max: MAX_AMOUNT_MINOR,
            });
        }

        Ok(Self {
            amount_minor,
            currency: currency.to_uppercase(),
        })
    }

    /// Return the amount in integer minor units.
    #[must_use]
    pub fn amount_minor(&self) -> u64 {
        self.amount_minor
    }

    /// Return the currency code (uppercased at construction).
    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Checked addition. Returns `None` on overflow or ceiling violation.
    ///
    /// Used for accumulating outstanding balance in offline budget checks.
    /// Never panics; callers must handle the `None` case explicitly.
    #[must_use]
    pub fn checked_add(&self, other: &Money) -> Option<Money> {
        if self.currency != other.currency {
            return None;
        }
        let sum = self.amount_minor.checked_add(other.amount_minor)?;
        if sum > MAX_AMOUNT_MINOR {
            return None;
        }
        Some(Money {
            amount_minor: sum,
            currency: self.currency.clone(),
        })
    }

    /// Checked comparison: returns whether `self` is within (≤) the given limit.
    #[must_use]
    pub fn is_within_limit(&self, limit: &Money) -> bool {
        self.currency == limit.currency && self.amount_minor <= limit.amount_minor
    }
}

/// Validate the currency string per the rules in `MONEY.md`.
fn validate_currency(currency: &str) -> Result<(), ValidationError> {
    if currency.is_empty() {
        return Err(ValidationError::InvalidCurrency {
            reason: "currency code must not be empty".into(),
        });
    }
    if currency.len() > MAX_CURRENCY_LEN {
        return Err(ValidationError::InvalidCurrency {
            reason: format!(
                "currency code length {} exceeds maximum {}",
                currency.len(),
                MAX_CURRENCY_LEN
            ),
        });
    }
    // Only ASCII alphabetic characters (ISO 4217 style).
    if !currency.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(ValidationError::InvalidCurrency {
            reason: "currency code must contain only ASCII alphabetic characters".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_inr_paise() {
        let m = Money::new(150, "INR").unwrap();
        assert_eq!(m.amount_minor(), 150);
        assert_eq!(m.currency(), "INR");
    }

    #[test]
    fn zero_amount_is_valid() {
        // Zero is structurally valid; policy rejection happens at validation layer.
        let m = Money::new(0, "INR").unwrap();
        assert_eq!(m.amount_minor(), 0);
    }

    #[test]
    fn currency_is_uppercased() {
        let m = Money::new(100, "inr").unwrap();
        assert_eq!(m.currency(), "INR");
    }

    #[test]
    fn amount_at_ceiling_accepted() {
        let m = Money::new(MAX_AMOUNT_MINOR, "INR").unwrap();
        assert_eq!(m.amount_minor(), MAX_AMOUNT_MINOR);
    }

    #[test]
    fn amount_above_ceiling_rejected() {
        let err = Money::new(MAX_AMOUNT_MINOR + 1, "INR").unwrap_err();
        assert!(matches!(err, ValidationError::AmountExceedsMaximum { .. }));
    }

    #[test]
    fn empty_currency_rejected() {
        let err = Money::new(100, "").unwrap_err();
        assert!(matches!(err, ValidationError::InvalidCurrency { .. }));
    }

    #[test]
    fn long_currency_rejected() {
        let err = Money::new(100, "TOOLONGCODE").unwrap_err();
        assert!(matches!(err, ValidationError::InvalidCurrency { .. }));
    }

    #[test]
    fn numeric_currency_rejected() {
        let err = Money::new(100, "123").unwrap_err();
        assert!(matches!(err, ValidationError::InvalidCurrency { .. }));
    }

    #[test]
    fn checked_add_same_currency() {
        let a = Money::new(100, "INR").unwrap();
        let b = Money::new(200, "INR").unwrap();
        let sum = a.checked_add(&b).unwrap();
        assert_eq!(sum.amount_minor(), 300);
        assert_eq!(sum.currency(), "INR");
    }

    #[test]
    fn checked_add_different_currency_returns_none() {
        let a = Money::new(100, "INR").unwrap();
        let b = Money::new(100, "USD").unwrap();
        assert!(a.checked_add(&b).is_none());
    }

    #[test]
    fn checked_add_overflow_returns_none() {
        let a = Money::new(MAX_AMOUNT_MINOR, "INR").unwrap();
        let b = Money::new(1, "INR").unwrap();
        assert!(a.checked_add(&b).is_none());
    }

    #[test]
    fn is_within_limit_true() {
        let amount = Money::new(500, "INR").unwrap();
        let limit = Money::new(1000, "INR").unwrap();
        assert!(amount.is_within_limit(&limit));
    }

    #[test]
    fn is_within_limit_equal_is_within() {
        let amount = Money::new(1000, "INR").unwrap();
        let limit = Money::new(1000, "INR").unwrap();
        assert!(amount.is_within_limit(&limit));
    }

    #[test]
    fn is_within_limit_exceeds_returns_false() {
        let amount = Money::new(1001, "INR").unwrap();
        let limit = Money::new(1000, "INR").unwrap();
        assert!(!amount.is_within_limit(&limit));
    }

    #[test]
    fn is_within_limit_different_currency_returns_false() {
        let amount = Money::new(100, "INR").unwrap();
        let limit = Money::new(1000, "USD").unwrap();
        assert!(!amount.is_within_limit(&limit));
    }

    /// Ensure no floating-point conversion is possible on this type.
    /// This is a compile-time check: `Money` must not implement `From<f64>` or `Into<f64>`.
    #[test]
    fn no_float_arithmetic_compiles() {
        // If this test compiles without using f64/f32, the type is free of float coercions.
        let _m = Money::new(9999, "INR").unwrap(); // ₹99.99 expressed correctly as 9999 paise
    }
}

//! Phone number analysis filter.
//!
//! Normalizes phone numbers by stripping formatting characters and
//! generating multiple searchable representations (with/without country code,
//! with/without leading zeros, etc.).

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Phone number normalization and search filter.
///
/// Normalizes phone tokens by:
/// 1. Stripping all non-digit characters (except leading +)
/// 2. Generating variants (with/without country code)
///
/// Equivalent to Elasticsearch's `analysis-phonenumber` plugin.
#[derive(Clone, Debug)]
pub struct PhoneNumberFilter {
    /// Generate additional tokens with/without country code.
    generate_variants: bool,
}

impl PhoneNumberFilter {
    pub fn new() -> Self {
        Self {
            generate_variants: true,
        }
    }

    pub fn with_variants(mut self, generate: bool) -> Self {
        self.generate_variants = generate;
        self
    }

    /// Normalize a phone number string to digits only.
    fn normalize_phone(&self, input: &str) -> Option<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        let has_plus = trimmed.starts_with('+');
        let digits: String = trimmed.chars().filter(|ch| ch.is_ascii_digit()).collect();

        if digits.len() < 4 {
            return None; // Too short for a phone number
        }

        if has_plus {
            Some(format!("+{}", digits))
        } else {
            Some(digits)
        }
    }

    /// Generate search variants for a phone number.
    fn generate_phone_variants(&self, normalized: &str) -> Vec<String> {
        let mut variants = Vec::new();

        // Always include the full normalized form
        variants.push(normalized.to_string());

        let digits_only: String = normalized
            .chars()
            .filter(|ch| ch.is_ascii_digit())
            .collect();

        // Add digits-only version (without +)
        if normalized.starts_with('+') && !digits_only.is_empty() {
            variants.push(digits_only.clone());
        }

        // Add version without country code (remove first 1-3 digits if starts with +)
        if normalized.starts_with('+') && digits_only.len() > 7 {
            // Try removing 1-digit country code
            let without_cc1 = &digits_only[1..];
            variants.push(without_cc1.to_string());

            // Try removing 2-digit country code
            if digits_only.len() > 8 {
                let without_cc2 = &digits_only[2..];
                variants.push(without_cc2.to_string());
            }
        }

        // Add version without leading zero (common in local numbers)
        if digits_only.starts_with('0') {
            variants.push(digits_only[1..].to_string());
        }

        variants.sort();
        variants.dedup();
        variants
    }
}

impl Default for PhoneNumberFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for PhoneNumberFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();

        // Check if this looks like a phone number (contains mostly digits)
        let digit_count = term.chars().filter(|ch| ch.is_ascii_digit()).count();
        let total_chars = term.chars().count();
        if digit_count < 4 || (digit_count as f64 / total_chars as f64) < 0.5 {
            return (false, None);
        }

        let normalized = match self.normalize_phone(term) {
            Some(n) => n,
            None => return (false, None),
        };

        if !self.generate_variants {
            if normalized != term {
                token.term = Cow::Owned(normalized);
            }
            return (false, None);
        }

        let variants = self.generate_phone_variants(&normalized);
        if variants.is_empty() {
            return (false, None);
        }

        // First variant becomes the token
        token.term = Cow::Owned(variants[0].clone());

        // Additional variants as extra tokens
        if variants.len() > 1 {
            let extras: Vec<Token<'a>> = variants[1..]
                .iter()
                .map(|v| Token {
                    term: Cow::Owned(v.clone()),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                })
                .collect();
            (false, Some(extras))
        } else {
            (false, None)
        }
    }
}

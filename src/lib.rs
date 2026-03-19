//! Prime number string algebra.
//!
//! Maps each ASCII lowercase letter to a unique prime, then uses
//! multiplication and divisibility to perform set/multiset operations
//! on strings without sorting or hashing.
//!
//! # Examples
//!
//! ```
//! use arithmos::PrimeSignature;
//!
//! let s1 = PrimeSignature::new(b"listen");
//! let s2 = PrimeSignature::new(b"silent");
//! assert!(s1.is_anagram_of(&s2));
//!
//! let haystack = PrimeSignature::new(b"abcdef");
//! let needle = PrimeSignature::new(b"bcd");
//! assert!(haystack.contains_letters_of(&needle));
//! ```

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]

/// Lookup table: ASCII byte -> prime (0 = not a lowercase letter).
///
/// a=2, b=3, c=5, d=7, ..., z=101.
/// Using a full 256-entry table avoids branching in the hot path.
const PRIME_TABLE: [u128; 256] = {
    let mut table = [0u128; 256];
    let primes: [u128; 26] = [
        2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89,
        97, 101,
    ];
    let mut i = 0;
    while i < 26 {
        table[b'a' as usize + i] = primes[i];
        table[b'A' as usize + i] = primes[i];
        i += 1;
    }
    table
};

/// The product of primes representing a string's letters.
///
/// Non-letter bytes are ignored. Letter case is folded (A=a).
/// Uses `u128` for the product, which handles strings up to ~15 letters
/// before overflow risk becomes significant. For longer strings, use
/// [`PrimeSignatureBig`] (requires the `big` feature) or check with
/// [`PrimeSignature::checked_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrimeSignature {
    product: u128,
    len: u32,
}

impl PrimeSignature {
    /// Computes the prime signature of `bytes`, ignoring non-letter bytes.
    ///
    /// # Panics
    ///
    /// Panics on overflow. Use [`Self::checked_new`] for untrusted input.
    #[must_use]
    pub fn new(bytes: &[u8]) -> Self {
        Self::checked_new(bytes).expect("prime product overflow")
    }

    /// Computes the prime signature, returning `None` on overflow.
    #[must_use]
    pub fn checked_new(bytes: &[u8]) -> Option<Self> {
        let mut product: u128 = 1;
        let mut len: u32 = 0;
        for &b in bytes {
            let p = PRIME_TABLE[b as usize];
            if p != 0 {
                product = product.checked_mul(p)?;
                len += 1;
            }
        }
        Some(Self { product, len })
    }

    /// Returns the raw prime product.
    #[must_use]
    pub const fn product(&self) -> u128 {
        self.product
    }

    /// Returns the number of letter bytes that contributed to the product.
    #[must_use]
    pub const fn letter_count(&self) -> u32 {
        self.len
    }

    /// Two signatures are anagrams iff their products are equal.
    #[must_use]
    pub const fn is_anagram_of(&self, other: &Self) -> bool {
        self.product == other.product
    }

    /// `self` contains all the letters of `other` (with multiplicity)
    /// iff `other.product` divides `self.product`.
    #[must_use]
    pub const fn contains_letters_of(&self, other: &Self) -> bool {
        if other.product == 0 {
            return false;
        }
        self.product.is_multiple_of(other.product)
    }

    /// Combine two signatures (concatenation of their strings).
    ///
    /// Returns `None` on overflow.
    #[must_use]
    pub const fn merge(&self, other: &Self) -> Option<Self> {
        match self.product.checked_mul(other.product) {
            Some(product) => Some(Self {
                product,
                len: self.len + other.len,
            }),
            None => None,
        }
    }

    /// Remove the letters of `other` from `self`.
    ///
    /// Returns `None` if `self` does not contain all letters of `other`.
    #[must_use]
    pub const fn remove(&self, other: &Self) -> Option<Self> {
        if other.product == 0 || !self.product.is_multiple_of(other.product) {
            return None;
        }
        Some(Self {
            product: self.product / other.product,
            len: self.len - other.len,
        })
    }
}

/// Find all positions where a window of `needle.letter_count()` letters
/// in `haystack` is an anagram of `needle`.
///
/// Returns starting byte indices into `haystack`.
/// Uses a rolling product: divides out the leaving letter and multiplies
/// in the entering letter, making each step O(1).
///
/// # Examples
///
/// ```
/// use arithmos::{PrimeSignature, find_anagram_windows};
///
/// let needle = PrimeSignature::new(b"abc");
/// let positions = find_anagram_windows(b"cbadabca", &needle);
/// assert_eq!(positions, vec![0, 4, 5]);
/// ```
#[must_use]
pub fn find_anagram_windows(haystack: &[u8], needle: &PrimeSignature) -> Vec<usize> {
    let target = needle.product();
    let win_len = needle.letter_count() as usize;

    if win_len == 0 || haystack.len() < win_len {
        return Vec::new();
    }

    // Collect only the letter bytes and their original indices.
    let letters: Vec<(usize, u128)> = haystack
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| {
            let p = PRIME_TABLE[b as usize];
            if p != 0 { Some((i, p)) } else { None }
        })
        .collect();

    if letters.len() < win_len {
        return Vec::new();
    }

    let mut results = Vec::new();

    // Build initial window product.
    let mut product: u128 = 1;
    let mut overflowed = false;
    for &(_, p) in &letters[..win_len] {
        if let Some(v) = product.checked_mul(p) {
            product = v;
        } else {
            overflowed = true;
            break;
        }
    }

    if !overflowed && product == target {
        results.push(letters[0].0);
    }

    // Slide the window.
    for i in 1..=letters.len() - win_len {
        if overflowed {
            // Recompute from scratch.
            product = 1;
            overflowed = false;
            for &(_, p) in &letters[i..i + win_len] {
                if let Some(v) = product.checked_mul(p) {
                    product = v;
                } else {
                    overflowed = true;
                    break;
                }
            }
        } else {
            let leaving = letters[i - 1].1;
            product /= leaving;
            if let Some(v) = product.checked_mul(letters[i + win_len - 1].1) {
                product = v;
            } else {
                overflowed = true;
                continue;
            }
        }
        if !overflowed && product == target {
            results.push(letters[i].0);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anagram_basic() {
        let a = PrimeSignature::new(b"listen");
        let b = PrimeSignature::new(b"silent");
        assert!(a.is_anagram_of(&b));
    }

    #[test]
    fn not_anagram() {
        let a = PrimeSignature::new(b"hello");
        let b = PrimeSignature::new(b"world");
        assert!(!a.is_anagram_of(&b));
    }

    #[test]
    fn contains_letters() {
        let hay = PrimeSignature::new(b"abcdef");
        let ndl = PrimeSignature::new(b"bcd");
        assert!(hay.contains_letters_of(&ndl));
    }

    #[test]
    fn does_not_contain() {
        let hay = PrimeSignature::new(b"abc");
        let ndl = PrimeSignature::new(b"abcd");
        assert!(!hay.contains_letters_of(&ndl));
    }

    #[test]
    fn multiplicity_matters() {
        let a = PrimeSignature::new(b"aab");
        let b = PrimeSignature::new(b"ab");
        assert!(!a.is_anagram_of(&b));
        assert!(a.contains_letters_of(&b));
    }

    #[test]
    fn merge_and_remove() {
        let a = PrimeSignature::new(b"abc");
        let b = PrimeSignature::new(b"def");
        let merged = a.merge(&b).unwrap();
        let expected = PrimeSignature::new(b"abcdef");
        assert!(merged.is_anagram_of(&expected));

        let removed = merged.remove(&b).unwrap();
        assert!(removed.is_anagram_of(&a));
    }

    #[test]
    fn case_insensitive() {
        let a = PrimeSignature::new(b"Listen");
        let b = PrimeSignature::new(b"Silent");
        assert!(a.is_anagram_of(&b));
    }

    #[test]
    fn ignores_non_letters() {
        let a = PrimeSignature::new(b"a b c!");
        let b = PrimeSignature::new(b"abc");
        assert!(a.is_anagram_of(&b));
    }

    #[test]
    fn empty_string() {
        let a = PrimeSignature::new(b"");
        assert_eq!(a.product(), 1);
        assert_eq!(a.letter_count(), 0);
    }

    #[test]
    fn find_anagram_windows_basic() {
        let needle = PrimeSignature::new(b"abc");
        let positions = find_anagram_windows(b"cbadabca", &needle);
        assert_eq!(positions, vec![0, 4, 5]);
    }

    #[test]
    fn find_anagram_windows_no_match() {
        let needle = PrimeSignature::new(b"xyz");
        let positions = find_anagram_windows(b"abcdef", &needle);
        assert!(positions.is_empty());
    }

    #[test]
    fn checked_new_overflow() {
        // 26 z's: 101^26 overflows u128
        let long = vec![b'z'; 30];
        assert!(PrimeSignature::checked_new(&long).is_none());
    }
}

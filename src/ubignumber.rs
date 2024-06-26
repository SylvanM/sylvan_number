use std::{cmp::max, fmt::Debug, io::StderrLock, ops::{Add, AddAssign, BitOr, BitOrAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Range, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign}};
use algebra_kit::algebra::{EuclideanDomain, Ring};
use rand::Rng;

use crate::int_utility;

pub type Word = u64;

pub const WORD_BYTE_COUNT: usize = std::mem::size_of::<Word>();
pub const WORD_BIT_COUNT: usize = WORD_BYTE_COUNT * 8;

/// An arbitrary precision unsigned integer value
#[derive(Clone)]
pub struct UBigNumber {

	/// The words of this number. Can also be thought of as the base 2^64 digits of this number. 
	/// Represented in Little-Endian. So, words[0] is the least significant word.
	/// 
	/// - Invariant: `words` is of minimal size. That is, there are no extraneous 0 words.
	pub words: Vec<Word>

}

impl UBigNumber {

	// MARK: Initialization

	/// Creates a new `UBigNumber` representing 0
	pub fn new() -> UBigNumber {
		UBigNumber::zero()
	}

	/// Creates a random `UBigNumber`. This is *not* cryptographically secure.
	pub fn rand(len: usize) -> UBigNumber {
		let mut words = vec![0 ; len];
		for i in 0..len {
			words[i] = rand::thread_rng().gen();
		}
		UBigNumber::from_words(words)
	}

	/// Creates a UBigNumber from a string
	/// 
	/// The string must be already sanitized, meaning if it is a hexadecimal string, there is no "0x" prefix.
	pub fn from_hex_string(hex_str: &str,) -> UBigNumber {

		// The amount of hex digits that take up one Word
		let word_str_len = WORD_BYTE_COUNT * 2; // 2 hex characters per byte

		// The following is somewhat generated by ChatGPT. I asked it to implement
		// this function and it slightly failed. So, I'm just taking what worked,
		// and changing it to be a bit better :)

		// --- BEGIN AI CODE ---

		// Step 1: Strip optional "0x" prefix
        let hex_str = if hex_str.starts_with("0x") || hex_str.starts_with("0X") {
            &hex_str[2..]
        } else {
            hex_str
        };

        let hex_str = if hex_str.len() % word_str_len != 0 {
			let zeroes = (0..(word_str_len - (hex_str.len() % word_str_len))).map(|_| "0").collect::<String>();
            format!("{}{}", zeroes, hex_str)
        } else {
            hex_str.to_string()
        };

		// --- END AI CODE ---

		// Now, for each clump of 16, we map that to a word.

		debug_assert!(hex_str.len() % word_str_len == 0);

		let new_len = hex_str.len() / word_str_len;
		let mut words = vec![0 ; new_len];

		for i in 0..new_len {
			words[new_len - i - 1] = Word::from_str_radix(&hex_str[(word_str_len * i)..(word_str_len * i + word_str_len)], 16).unwrap();
		}

		let mut ubn = UBigNumber::from_words(words);
		ubn.normalize();
		ubn
	}

	/// Creates a UBigNumber from given words
	pub fn from_words(words: Vec<Word>) -> UBigNumber {
		let mut ubn = UBigNumber { words };
		ubn.normalize();
		ubn
	}
	
	/// Promotes an integer to a UBigNumber
	pub fn from_int(int: Word) -> UBigNumber {
		UBigNumber { words: vec![int]}
	}

	/// Creates a new `UBigNumber` representing 1
	pub fn one() -> Self {
		UBigNumber { words: vec![1] }
	}

	/// Creates a new `UBigNumber` representing 0
	pub fn zero() -> UBigNumber {
		UBigNumber { words: vec![0] }
	}

	pub fn is_zero(&self) -> bool {
		self.words == [0]
	}

	// MARK: Housekeeping

	/// The number of words in this `UBigNumber`
	pub fn len(&self) -> usize {
		self.words.len()
	}

	/// Extends the word array to account for possible new friends
	fn extend(&mut self, new_len: usize) {
		if new_len < self.len() {
			return;
		} else {
			self.words.append(&mut vec![0 ; new_len - self.len()])
		}
	}

	/// Removes extraneous zeroes from the end of the words array.
	fn normalize(&mut self) {
		if self.words.len() == 1 { 
			return; 
		} else if self.words.is_empty() {
			self.words = vec![0];
			return;
		} else if self.words.iter().map(|w| *w == 0).reduce(|w1, w2| w1 & w2).unwrap() { // fancy way of seeing if all words are zero
			self.words = vec![0];
			return;
		} else {
			let mut extra_zeroes = 0;
			for i in (0..self.words.len()).rev() {
				if self.words[i] != 0 {
					break;
				} else {
					extra_zeroes += 1;
				}
			}
	
			let new_len = self.words.len().saturating_sub(extra_zeroes);
			self.words.truncate(new_len);
		}
	}

	fn safe_word(&self, index: usize) -> Word {
		if index >= self.len() {
			0
		} else {
			self[index]
		}
	}

	/// The most significant word in the array representation of this number
	pub fn msw(&self) -> Word {
		self[self.len() - 1]
	}


	// fn safe_word_put(&mut self, index: usize, new_val: Word) {
	// 	if index >= self.size() {
	// 		self.extend(index + 1);
	// 	}
		
	// 	self[index] = new_val
	// }

	// MARK: Arithmetic Helpers

	/// Adds another `UBN` to this `UBN`
	fn custom_add(&mut self, rhs: UBigNumber, handle_overflow: bool) {
		let required_size = if handle_overflow { max(self.len(), rhs.len()) + 1 } else { self.len() };

		self.extend(required_size);

		let mut carry = false;

		for i in 0..self.len() {
			(self[i], carry) = self[i].carrying_add(rhs.safe_word(i), carry);
		}

		self.normalize();
	}

	/// Computes the quotient and remainder when dividing by a single 64-bit digit
	fn div_rem_short(dividend: UBigNumber, divisor: Word) -> (UBigNumber, Word) {
		let mut partial_remainder = 0;

		let mut quotient_words = vec![0 ; dividend.len()];

		let mut starting_digit_offset = 0;

		let mut partial_dividend = if divisor > dividend[dividend.len() - 1] {
			starting_digit_offset = 1;
			UBigNumber::from_words(vec![dividend[dividend.len() - 2], dividend[dividend.len() - 1]])
		} else {
			UBigNumber::from_int(dividend[dividend.len() - 1])
		};

		for j in (0..(dividend.len() - starting_digit_offset)).rev() {

			debug_assert!(partial_dividend.len() <= 2);

			let (_, qhat, _) = int_utility::div_wide(partial_dividend.safe_word(1), partial_dividend[0], divisor);
			quotient_words[j] = qhat;

			let partial_product = UBigNumber::from_int(divisor) * qhat.into();
			partial_remainder = (partial_dividend - partial_product)[0];

			if j == 0 { break; }

			partial_dividend = UBigNumber::from_words(vec![dividend[j - 1], partial_remainder]);
		}

		(UBigNumber::from_words(quotient_words), partial_remainder)
	}

	fn div_rem_core(dividend: UBigNumber, divisor: UBigNumber) -> (UBigNumber, UBigNumber) {

		// This is a REALLY BAD implementation.
		let k = dividend.len() - divisor.len() + 1;
		let mut quotient_words = vec![0 ; k];

		let mut first_digit_offset = 0;

		let mut partial_dividend = if divisor > dividend.sub_number((k - 1)..dividend.len()) {
			first_digit_offset = 1;
			dividend.sub_number((k - 2)..dividend.len())
		} else {
			dividend.sub_number((k - 1)..dividend.len())
		};

		let mut partial_remainder: UBigNumber = 0.into();

		for j in (0..(k - first_digit_offset)).rev() {
			
			// we want to divide into the partial dividend!
			let u_first = partial_dividend.safe_word(divisor.len());
			let u_second = partial_dividend[divisor.len() - 1];

			let v_first = divisor[divisor.len() - 1];

			let (q_hi, mut qhat, _) = int_utility::div_wide(u_first, u_second, v_first);

			debug_assert_eq!(q_hi, 0);

			let mut partial_product = divisor.clone() * qhat.into();

			while partial_product > partial_dividend {
				qhat -= 1;
				partial_product -= divisor.clone();
			}

			quotient_words[j] = qhat;

			partial_remainder = partial_dividend.clone() - partial_product;

			if j == 0 { break; }

			partial_dividend = partial_remainder.clone();
			partial_dividend.words.insert(0, dividend[j - 1]);

		}


		(UBigNumber::from_words(quotient_words), partial_remainder)

	}
	
	/// Returns a sub-integer, the interger represented by a selected range of the words of this UBigNumber.
	/// For example, if the (base 2^64) digits of this number are bn = 439803, then bn[2..=4] == 398
	pub fn sub_number(&self, range: Range<usize>) -> UBigNumber {
		let slice = &self.words[range];
		let sub_vec = slice.to_vec();
		UBigNumber::from_words(sub_vec)
	}

	pub fn quotient_and_remainder(&self, divisor: &Self) -> (Self, Self) {
		if divisor.is_zero() {
			panic!("Division by zero")
		} else if self < divisor {
			(UBigNumber::zero(), self.clone())
		} else if self == divisor {
			(UBigNumber::one(), UBigNumber::zero())
		} else {
			if divisor.len() == 1 {
				let (q, r) = UBigNumber::div_rem_short(self.clone(), divisor[0]);
				(q, r.into())
			} else {
				UBigNumber::div_rem_core(self.clone(), divisor.clone())
			}
		}
	}
}

// MARK: Utility

impl Index<usize> for UBigNumber {
	type Output = Word;

	fn index(&self, index: usize) -> &Self::Output {
		&self.words[index]
	}
}

impl IndexMut<usize> for UBigNumber {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.words[index]
	}
}

impl Debug for UBigNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "0x{:X}", self.words[self.len() - 1])?;

		for i in (0..(self.len() - 1)).rev() {
			write!(f, " {:016X}", self[i])?;
		}

        Ok(())
	}
}

// MARK: Convenience Conversions

impl From<&str> for UBigNumber {
	fn from(value: &str) -> Self {
		UBigNumber::from_hex_string(value)
	}
}

impl From<Word> for UBigNumber {
	fn from(value: Word) -> Self {
		UBigNumber { words: vec![value] }
	}
}

// MARK: Bitwise Operations

impl BitOr for UBigNumber {
	type Output = UBigNumber;

	fn bitor(self, rhs: Self) -> Self::Output {
		let mut or = self;
		or |= rhs;
		or
	}
}

impl BitOrAssign for UBigNumber {
	fn bitor_assign(&mut self, rhs: Self) {
		self.extend(max(self.len(), rhs.len()));

		for i in 0..self.len() {
			self[i] |= rhs.safe_word(i)
		}

		self.normalize()
	}
}

impl Shl<Word> for UBigNumber {
	type Output = UBigNumber;

	fn shl(self, rhs: Word) -> Self::Output {
		let mut shifted = self;
		shifted <<= rhs;
		shifted
	}
}

impl ShlAssign<Word> for UBigNumber {
	fn shl_assign(&mut self, rhs: Word) {
		let word_shift = rhs as usize / WORD_BIT_COUNT;
		let bit_shift = rhs as usize % WORD_BIT_COUNT;
		
		self.extend(self.len() + word_shift + 1);

		// let's first shift the words
		for i in word_shift..self.len() {
			self[i] = self[i - word_shift];
		}
		for i in 0..word_shift {
			self[i] = 0;
		}

		debug_assert!(self[self.len() - 1] == 0);

		// now we shift the bits!
		for i in (1..self.len()).rev() {
			self[i] <<= bit_shift;
			
			// take the upper bit_shift bits of self[i - 1] and put them in the lower ones here
			// self[i] |= self[i - 1] >> (WORD_BIT_COUNT - bit_shift);
			self[i] |= self[i - 1].wrapping_shr((WORD_BIT_COUNT - bit_shift) as u32)
		}

		self[0] <<= bit_shift;

		self.normalize();
	}
}

impl Shr<Word> for UBigNumber {
	type Output = UBigNumber;
	
	fn shr(self, rhs: Word) -> Self::Output {
		let mut shifted = self;
		shifted >>= rhs;
		shifted
	}
}

impl ShrAssign<Word> for UBigNumber {
	fn shr_assign(&mut self, rhs: Word) {
		let word_shift = rhs as usize / WORD_BIT_COUNT;
		let bit_shift = rhs as usize % WORD_BIT_COUNT;

		if word_shift >= self.len() {
			self.words = vec![0];
			return; // we just died :(
		}

		// let's first shift the words
		for i in 0..(self.len() - word_shift) {
			self[i] = self[i + word_shift];
		}
		for i in (self.len() - word_shift)..self.len() {
			self[i] = 0;
		}

		for i in 0..(self.len() - 1) {
			self[i] >>= bit_shift;
			self[i] |= self[i + 1].wrapping_shl((WORD_BIT_COUNT - bit_shift).try_into().unwrap())
		}

		let size = self.len(); // doing this to avoid violating borrowing
		self[size - 1] >>= bit_shift;
	}
}

// MARK: Comparison

impl PartialOrd for UBigNumber {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for UBigNumber {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		for i in (0..(max(self.len(), other.len()))).rev() {
			let cmp = self.safe_word(i).cmp(&other.safe_word(i));
			if cmp != std::cmp::Ordering::Equal {
				return cmp;
			}
 		}

		std::cmp::Ordering::Equal
	}
}

impl PartialEq for UBigNumber {
	fn eq(&self, other: &Self) -> bool {
		self.words == other.words
	}
}

impl Eq for UBigNumber { /* Nothing here I guess? */ }


// MARK: Arithmetic

impl Add for UBigNumber {
	type Output = UBigNumber;

	fn add(self, rhs: Self) -> Self::Output {
		let mut sum = self;
		sum += rhs;
		sum
	}
}

impl AddAssign for UBigNumber {
	fn add_assign(&mut self, rhs: Self) {
		self.custom_add(rhs, true);
	}
}

impl Sub for UBigNumber {
	type Output = UBigNumber;

	fn sub(self, rhs: Self) -> Self::Output {
		let mut diff = self;
		diff -= rhs;
		diff
	}
}

impl SubAssign for UBigNumber {
	fn sub_assign(&mut self, mut rhs: Self) {
		rhs.extend(self.len());
		self.extend(rhs.len());

		// negate rhs using 2's complement
		let mut complement = UBigNumber { words: rhs.words.iter().map(|w| !w).collect() };
		complement.custom_add(UBigNumber::one(), false);
		self.custom_add(complement, false);
		self.normalize()
	}
}

impl Mul for UBigNumber {
	type Output = UBigNumber;

	fn mul(mut self, rhs: Self) -> Self::Output {
		self *= rhs;
		self
	}
}

impl MulAssign for UBigNumber {
	fn mul_assign(&mut self, rhs: Self) {
		if *self == UBigNumber::zero() || rhs == UBigNumber::zero() {
			self.words = vec![0];
		} else if *self == UBigNumber::one() {
			self.words = rhs.words;
		} else if rhs == UBigNumber::one() {
			return;
		} else {
			self.words = int_utility::word_mul(self.words.clone(), rhs.words);
			self.normalize()
		}
	}
}

impl Div for UBigNumber {
	type Output = UBigNumber;

	fn div(self, rhs: Self) -> Self::Output {
		match self.quotient_and_remainder(&rhs) { (q, _) => q }
	}
}

impl DivAssign for UBigNumber {
	fn div_assign(&mut self, rhs: Self) {
		let (quotient, _) = self.quotient_and_remainder(&rhs);
		self.words = quotient.words;
		self.normalize()
	}
}

impl Rem for UBigNumber {
	type Output = UBigNumber;

	fn rem(self, rhs: Self) -> Self::Output {
		match self.quotient_and_remainder(&rhs) { (_, r) => r }
	}
}

impl RemAssign for UBigNumber {
	fn rem_assign(&mut self, rhs: Self) {
		let (_, remainder) = self.quotient_and_remainder(&rhs);
		self.words = remainder.words;
		self.normalize()
	}
}
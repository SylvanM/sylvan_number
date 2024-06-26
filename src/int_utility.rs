use crate::ubignumber::{self, Word};

/// Computes the 128-bit result of the operation `a*b + c + d`
/// 
/// This calls compiler intrinsic commands which just call processor instructions or whatever
pub fn addmul(a: Word, b: Word, c: Word, d: Word) -> (Word, Word) {
	let (lo, hi) = a.carrying_mul(b, c);
	let (new_lo, carry) = lo.carrying_add(d, false);
	let (new_hi, _) = hi.carrying_add(0, carry);
	(new_lo, new_hi)
}

/// Computes wide multiplication
pub fn word_mul(lhs: Vec<Word>, rhs: Vec<Word>) -> Vec<Word> {
	let mut product_words = vec![0 ; lhs.len() + rhs.len()];

	for j in 0..rhs.len() {
		let mut carry = 0;

		for i in 0..lhs.len() {
			(product_words[i + j], carry) = addmul(lhs[i], rhs[j], carry, product_words[i + j])
		}

		product_words[lhs.len() + j] = carry
	}

	product_words
}

/// Computes the quotient and remainder that is the result of dividing a two-word number by one word
/// 
/// Computes [hq|lq] = [hi|lo] / divisor, returns hq, lq, and the remainder, which will just be one word.
pub fn div_wide(hi: Word, lo: Word, divisor: Word) -> (Word, Word, Word) {
	
	// This part of the code requires that Word == u64

	// Rust supports 128 bit integers! We can use this to make this a lot simpler, I think!
	let dividend = ((hi as u128) << 64) + lo as u128;
	let divisor = divisor as u128;
	let (q, r) = (dividend / divisor, (dividend % divisor) as Word);
	
	((q >> ubignumber::WORD_BIT_COUNT) as Word, (q & ((1 << ubignumber::WORD_BIT_COUNT) - 1)) as Word, r)
}
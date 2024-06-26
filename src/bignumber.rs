//! A signed, arbitrary sized interger

use std::{cmp::Ordering, fmt::Debug, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign}};

use algebra_kit::algebra::{EuclideanDomain, Ring};

use crate::ubignumber::UBigNumber;

#[derive(Clone)]
pub struct BigNumber {
	pub is_negative: bool,
	pub magnitude: UBigNumber
}

impl BigNumber {

	/// Creates a big number with a certain sign and magnitude
	pub fn from_sign_magnitude(is_negative: bool, magnitude: UBigNumber) -> BigNumber {
		BigNumber { is_negative, magnitude }
	}
	
	/// Creates a big number from an unsigned big number
	pub fn from_ubn(ubn: UBigNumber) -> BigNumber {
		BigNumber::from_sign_magnitude(false, ubn)
	}

	/// Computes the euclidean remainder when dividing by something.
	/// This essentiall the "modulo" operation as it's commonly thought of in algebra,
	/// where the remainder is always nonnegative.
	pub fn euc_rem(&self, divisor: BigNumber) -> BigNumber {
		let rem = self.clone() % divisor.clone();
		if rem.is_negative {
			rem + divisor
		} else {
			rem
		}
	}

}

impl From<UBigNumber> for BigNumber {
	fn from(value: UBigNumber) -> Self {
		BigNumber::from_ubn(value)
	}
}

impl Into<UBigNumber> for BigNumber {
	fn into(self) -> UBigNumber {
		if self.is_negative {
			panic!("Cannot coerce negative interger into unsigned integer")
		} else {
			self.magnitude
		}
	}
}

// MARK: Utility

impl Debug for BigNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_zero() {
			write!(f, "0x0")?;
		}

		let magnitude_debug_str = &format!("{:?}", self.magnitude)[2..];

		write!(f, "{}{}", if self.is_negative { "-"} else { "" }, magnitude_debug_str)?;

		Ok(())
	}
}

// MARK: Comparison

impl PartialEq for BigNumber {
	fn eq(&self, other: &Self) -> bool {
		self.is_negative == other.is_negative && self.magnitude == other.magnitude
	}
}

impl PartialOrd for BigNumber {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		if self.is_negative && !other.is_negative {
			Some(Ordering::Less)
		} else if !self.is_negative && other.is_negative {
			Some(Ordering::Greater)
		} else if self.is_negative && other.is_negative { // both negative
			match self.magnitude.partial_cmp(&other.magnitude) {
				Some(Ordering::Less) => Some(Ordering::Greater),
				Some(Ordering::Greater) => Some(Ordering::Less),
				Some(Ordering::Equal) => Some(Ordering::Equal),
				None => None,
			}
		} else { // both are positive!
			self.magnitude.partial_cmp(&other.magnitude)
		}
	}
}

// MARK: Arithmetic

impl Neg for BigNumber {
	type Output = BigNumber;

	fn neg(self) -> Self::Output {
		BigNumber::from_sign_magnitude(!self.is_negative, self.magnitude)
	}
}

impl Add for BigNumber {
	type Output = BigNumber;

	fn add(self, rhs: Self) -> Self::Output {
		if self.is_negative == rhs.is_negative {
			BigNumber::from_sign_magnitude(self.is_negative, self.magnitude + rhs.magnitude)
		} else {
			// reorder so that rhs is negative, and then we do subtraction with two positive numbers
			if self.is_negative {
				rhs.sub(-self)
			} else {
				self.sub(-rhs)
			}
		}
	}
}

impl AddAssign for BigNumber {
	fn add_assign(&mut self, rhs: Self) {
		*self = self.clone() + rhs
	}
}

impl Sub for BigNumber {
	type Output = BigNumber;

	fn sub(self, rhs: Self) -> Self::Output {
		if !self.is_negative && !rhs.is_negative {
			if self > rhs {
				BigNumber::from_sign_magnitude(self.is_negative, self.magnitude - rhs.magnitude)
			} else if self == rhs {
				BigNumber::zero()
			} else {
				BigNumber::from_sign_magnitude(true, rhs.magnitude - self.magnitude)
			}
		} else if !self.is_negative && rhs.is_negative {
			self.add(-rhs)
		} else if self.is_negative && !rhs.is_negative {
			BigNumber::from_sign_magnitude(true, self.magnitude + rhs.magnitude)
		} else {
			// self is negative, rhs is negative. We should swap them so we have
			//		-self - (-rhs) = -self + rhs = rhs - self
			// this reverts to the first case
			rhs.sub(-self)
		}
	}
}

impl SubAssign for BigNumber {
	fn sub_assign(&mut self, rhs: Self) {
		*self = self.clone() - rhs;
	}
}

impl Mul for BigNumber {
	type Output = BigNumber;

	fn mul(self, rhs: Self) -> Self::Output {
		BigNumber::from_sign_magnitude(self.is_negative ^ rhs.is_negative, self.magnitude * rhs.magnitude)
	}
}

impl MulAssign for BigNumber {
	fn mul_assign(&mut self, rhs: Self) {
		*self = self.clone() * rhs
	}
}

impl Div for BigNumber {
	type Output = BigNumber;

	fn div(self, rhs: Self) -> Self::Output {
		match self.quotient_and_remainder(&rhs) {
			(q, _) => q
		}
	}
}

impl DivAssign for BigNumber {
	fn div_assign(&mut self, rhs: Self) {
		*self = self.clone() / rhs
	}
}

impl Rem for BigNumber {
	type Output = BigNumber;

	fn rem(self, rhs: Self) -> Self::Output {
		match self.quotient_and_remainder(&rhs) {
			(_, r) => r
		}
	}
}

impl RemAssign for BigNumber {
	fn rem_assign(&mut self, rhs: Self) {
		*self = self.clone() % rhs
	}
}

// MARK: Algebra

impl Ring for BigNumber {
	fn one() -> Self {
		BigNumber::from_sign_magnitude(false, UBigNumber::one())
	}

	fn zero() -> Self {
		BigNumber::from_sign_magnitude(false, UBigNumber::zero())
	}

	fn is_zero(&self) -> bool {
		self.magnitude.is_zero()
	}

	fn power(&self, n: i64) -> Self {
		if n < 0 {
			panic!("Cannot invert integer")
		} else if n == 0 {
			BigNumber::one()
		} else {
			todo!()
		}
	}
}

impl EuclideanDomain for BigNumber {
	type SizeType = UBigNumber;

	fn euc_size(&self) -> Self::SizeType {
		self.magnitude.clone()
	}

	fn quotient_and_remainder(&self, divisor: &Self) -> (Self, Self) {
		let (u_q, u_r) = self.magnitude.quotient_and_remainder(&divisor.magnitude);
		(
			BigNumber::from_sign_magnitude(self.is_negative ^ divisor.is_negative, u_q),
			BigNumber::from_sign_magnitude(self.is_negative, u_r)
		)
	}
}
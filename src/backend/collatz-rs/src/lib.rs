#![feature(test)]
extern crate test;

extern crate num_bigint as bigmath;
extern crate num_traits;
extern crate smallvec;
extern crate gmp;

use num_traits::Num;
use num_traits::One;

use std::ffi::CStr;
use smallvec::SmallVec;
use smallvec::Array;
use bigmath::BigUint as NumBigUint;
use gmp::mpz::Mpz;

use std::ptr;
use std::mem;

mod tests;

type CStrArray = *const CStr;

// Used bigint implementation
type BigUint = Mpz;

#[allow(unused)]
unsafe extern "C" fn calc_sequence(
    input: *const CStr,
    size: *mut usize,
    error: *mut bool,
) -> *const CStrArray {
    let input = match (*input).to_str() {
        Ok(input) => input,
        Err(_) => {
            *error = true;
            return ptr::null();
        }
    };

    if let Ok(mut seq) = calc_sequence_rs(input) {
        let c_strs: Vec<*const CStr> = seq.iter_mut()
            .map(|s| {
                s.push('\0');
                s
            })
            .map(|s| CStr::from_bytes_with_nul(s.as_bytes()))
            .map(Result::unwrap)
            .map(|s| s as *const _)
            .collect();

        let ptr = c_strs.as_ptr();

        *error = false;
        *size = c_strs.len();

        mem::forget(seq);
        mem::forget(c_strs);

        return ptr;
    } else {
        *error = true;
        return ptr::null();
    }
}

type ParsingFailed = ();

// Result is either sequence or parsing error
fn calc_sequence_rs(input: &str) -> Result<Vec<String>, ParsingFailed> {
    // Sequence storage for different types of calculations
    let mut sequence_u64 = SmallVec::<[u64; 1024]>::new();
    let mut sequence_bigint = SmallVec::<[Mpz; 256]>::new();

    // Try to parse u64
    let number_u64_parsed = input.parse::<u64>();

    // This block will either return number_bigint in case of overflow or u64 parsing error,
    // or return result path from the function
    let number_bigint_parsed = if let Ok(number) = number_u64_parsed {
        match calc_sequence_for_number(number, &mut sequence_u64) {
            Done(_) => return Ok(to_string_vec(&sequence_u64, &sequence_bigint)),
            Overflow(x) => Ok(Mpz::from(x)),
            _ => unreachable!(),
        }
    } else {
        BigUint::from_dec_str(input)
    };

    if let Ok(number) = number_bigint_parsed {
        match calc_sequence_for_number(number, &mut sequence_bigint) {
            Done(_) => return Ok(to_string_vec(&sequence_u64, &sequence_bigint)),
            _ => unreachable!(),
        }
    }

    Err(())
}

#[inline]
fn calc_sequence_for_number<T: Number + Clone, A: Array<Item = T>>(
    number: T,
    sequence: &mut SmallVec<A>,
) -> StepResult<T> {
    sequence.push(number.clone());
    let mut temp = number;
    loop {
        match calc_step(temp) {
            Step(number) => {
                temp = number;
                sequence.push(temp.clone());
            }
            Done(number) => {
                sequence.push(number.clone());
                break Done(number);
            }
            Overflow(number) => {
                break Overflow(number);
            }
        }
    }
}

#[inline]
fn calc_step<T: Number>(input: T) -> StepResult<T> {
    let result = if input.is_odd() {
        input.triple_and_add_one()
    } else {
        input.divide_by_two()
    };

    if let Step(number) = result {
        if number.is_one() {
            Done(number)
        } else {
            Step(number)
        }
    } else {
        result
    }
}

#[inline]
fn to_string_vec<T: Number, U: Number>(a: &[T], b: &[U]) -> Vec<String> {
    let a_to_string = a.iter().map(Number::to_string);
    let b_to_string = b.iter().map(Number::to_string);
    a_to_string.chain(b_to_string).collect()
}

enum StepResult<T: Number> {
    Step(T),
    Done(T),
    Overflow(T),
}

use StepResult::*;

trait Number: Sized {
    fn divide_by_two(self) -> StepResult<Self>;
    fn triple_and_add_one(self) -> StepResult<Self>;
    fn is_odd(&self) -> bool;
    fn is_one(&self) -> bool;
    fn to_string(&self) -> String;
    fn from_dec_str(input: &str) -> Result<Self, ParsingFailed>;
}

impl Number for u64 {
    fn divide_by_two(self) -> StepResult<Self> {
        match self.checked_div(2) {
            Some(x) => Step(x),
            None => Overflow(self),
        }
    }

    fn triple_and_add_one(self) -> StepResult<Self> {
        let x = self.checked_mul(3).and_then(|x| x.checked_add(1));
        match x {
            Some(x) => Step(x),
            None => Overflow(self),
        }
    }

    fn is_odd(&self) -> bool {
        *self & 0x1 == 1
    }

    fn is_one(&self) -> bool {
        *self == 1
    }

    fn to_string(&self) -> String {
        format!("{}", self)
    }

    fn from_dec_str(input: &str) -> Result<Self, ParsingFailed> {
        input.parse::<u64>().map_err(|_|())
    }
}

impl Number for NumBigUint {
    fn divide_by_two(self) -> StepResult<Self> {
        Step(self / NumBigUint::from(2u64))
    }

    fn triple_and_add_one(self) -> StepResult<Self> {
        Step(self * NumBigUint::from(3u64) + NumBigUint::one())
    }

    fn is_odd(&self) -> bool {
        self & NumBigUint::one() == NumBigUint::one()
    }

    fn is_one(&self) -> bool {
        *self == NumBigUint::one()
    }

    fn to_string(&self) -> String {
        self.to_str_radix(10)
    }

    fn from_dec_str(input: &str) -> Result<Self, ParsingFailed> {
        NumBigUint::from_str_radix(input, 10).map_err(|_|())
    }
}

impl Number for Mpz {
    fn divide_by_two(self) -> StepResult<Self> {
        Step(self / Mpz::from(2u64))
    }

    fn triple_and_add_one(self) -> StepResult<Self> {
        Step(self * Mpz::from(3u64) + Mpz::one())
    }

    fn is_odd(&self) -> bool {
        self & Mpz::one() == Mpz::one()
    }

    fn is_one(&self) -> bool {
        *self == Mpz::one()
    }

    fn to_string(&self) -> String {
        self.to_str_radix(10)
    }

    fn from_dec_str(input: &str) -> Result<Self, ParsingFailed> {
        Mpz::from_str_radix(input, 10).map_err(|_|())
    }
}

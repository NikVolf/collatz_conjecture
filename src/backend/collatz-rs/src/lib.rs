#![feature(test)]
extern crate test;

extern crate bigint as u256;
extern crate num_bigint as bigmath;
extern crate num_traits;
extern crate smallvec;

use std::collections::LinkedList;

use num_traits::Num;
use num_traits::One;

use std::ffi::CStr;
use smallvec::SmallVec;
use smallvec::Array;
use u256::U256;
use bigmath::BigUint;

use std::ptr;
use std::mem;

mod tests;

type CStrArray = *const CStr;

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
    let mut sequence_u256 = SmallVec::<[U256; 512]>::new();
    let mut sequence_bigint = SmallVec::<[BigUint; 256]>::new();

    // Try to parse u64
    let number_u64_parsed = input.parse::<u64>();

    // This block will either return number_u256 in case of
    // overflow or u64 parsing error,
    // or return result from the function
    let number_u256_parsed = if let Ok(number) = number_u64_parsed {
        match calc_sequence_for_number(number, &mut sequence_u64) {
            Done(_) => return Ok(to_string_vec(&sequence_u64, &sequence_u256, &sequence_bigint)),
            Overflow(x) => Ok(U256::from(x)),
            _ => unreachable!()
        }
    } else {
        U256::from_dec_str(input)
    };

    // Switchec off the U256
    let number_bigint_parsed =  { /*
        if let Ok(number) = number_u256_parsed {
            match calc_sequence_for_number(number, &mut sequence_u256) {
                Done(_) => return Ok(to_string_vec(&sequence_u64, &sequence_u256, &sequence_bigint)),
                Overflow(x) => {
                    let bytes = &mut [0; 32];
                    x.to_little_endian(bytes);
                    Ok(BigUint::from_bytes_le(bytes))
                }
                _ => unreachable!()
            }
        } else {*/
            BigUint::from_str_radix(input, 10)
        //}
    };

    if let Ok(number) = number_bigint_parsed {
        match calc_sequence_for_number(number, &mut sequence_bigint) {
            Done(_) => return Ok(to_string_vec(&sequence_u64, &sequence_u256, &sequence_bigint)),
            _ => unreachable!()
        }
    }

    Err(())
}

#[inline]
fn calc_sequence_for_number<T: Number + Clone, A: Array<Item=T>>(number: T, sequence: &mut SmallVec<A>) -> StepResult<T> {
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
                return Done(number);
            }
            Overflow(number) => {
                // upgrade to U256
                break Overflow(number);
            }
        }
    }
} 

#[inline]
fn calc_sequence_bigint(input: &str) -> Result<Vec<String>, ParsingFailed> {
    let mut sequence_bigint = Vec::with_capacity(256);

    if let Ok(number_bigint) = BigUint::from_str_radix(input, 10) {
        let mut temp = number_bigint;
        sequence_bigint.push(temp.clone());
        loop {
            match calc_step(temp) {
                Step(number) => {
                    temp = number;
                    sequence_bigint.push(temp.clone());
                }
                Done(number) => {
                    sequence_bigint.push(number);
                    return Ok(
                        sequence_bigint.iter().map(Number::to_string).collect()
                    );
                }
                Overflow(_) => unreachable!(),
            }
        }
    }

    Err(())
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
fn to_string_vec<T: Number, U: Number, V: Number>(a: &[T], b: &[U], c: &[V]) -> Vec<String> {
    let a_to_string = a.iter().map(Number::to_string);
    let b_to_string = b.iter().map(Number::to_string);
    let c_to_string = c.iter().map(Number::to_string);
    a_to_string.chain(b_to_string).chain(c_to_string).collect()
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
}

impl Number for U256 {
    fn divide_by_two(self) -> StepResult<Self> {
        let (x, overflow) = self.overflowing_div(U256::from(2));
        if overflow { Overflow(self) } else { Step(x) }
    }

    fn triple_and_add_one(self) -> StepResult<Self> {
        let (x, mul_overflow) = self.overflowing_mul(U256::from(3));
        let (x, add_overflow) = x.overflowing_add(U256::one());
        if mul_overflow || add_overflow {
            Overflow(self)
        } else {
            Step(x)
        }
    }

    fn is_odd(&self) -> bool {
        self.bit(0)
    }

    fn is_one(&self) -> bool {
        *self == U256::one()
    }

    fn to_string(&self) -> String {
        format!("{}", self)
    }
}

impl Number for BigUint {
    fn divide_by_two(self) -> StepResult<Self> {
        Step(self / BigUint::from(2u64))
    }

    fn triple_and_add_one(self) -> StepResult<Self> {
        Step(self * BigUint::from(3u64) + BigUint::one())
    }

    fn is_odd(&self) -> bool {
        self & BigUint::one() == BigUint::one()
    }

    fn is_one(&self) -> bool {
        *self == BigUint::one()
    }

    fn to_string(&self) -> String {
        self.to_str_radix(10)
    }
}

use std::iter::Iterator;

trait Sequence<'a, T: Number + 'a> {
    type Iter: Iterator;
    fn push(&'a mut self, number: T);
    fn iter(&'a self) -> Self::Iter;
}

impl<'a, T: Number + 'a, A: Array<Item=T>> Sequence<'a, T> for SmallVec<A> {
    type Iter = std::slice::Iter<'a , T>;

    fn push(&mut self, number: T)  {
        self.push(number)
    }

    fn iter(&'a self) -> Self::Iter {
        self.into_iter()
    }
}

impl<'a, T: Number + 'a> Sequence<'a, T> for LinkedList<T> {
    type Iter = std::collections::linked_list::Iter<'a, T>;

    fn push(&mut self, number: T)  {
        self.push_back(number)
    }

    fn iter(&'a self) -> Self::Iter {
        self.into_iter()
    }
}
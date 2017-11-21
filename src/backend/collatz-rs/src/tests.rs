

// Baseline bigint implementation
#[inline]
#[cfg(test)]
fn calc_sequence_bigint(input: &str) -> Result<Vec<String>, ()> {
    use Number;
    use StepResult::*;
    use BigUint;
    use calc_step;

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
                    return Ok(sequence_bigint.iter().map(Number::to_string).collect());
                }
                Overflow(_) => unreachable!(),
            }
        }
    }

    Err(())
}

#[cfg(test)]
mod tests {
    use super::calc_sequence_bigint;
    use calc_sequence_rs;

    #[test]
    fn simple() {
        let result = calc_sequence_rs("12").unwrap();
        let result_str = result.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        assert_eq!(
            result_str,
            vec!["12", "6", "3", "10", "5", "16", "8", "4", "2", "1"]
        );

    }

    #[test]
    fn simple_bigint() {
        let result = calc_sequence_bigint("12").unwrap();
        let result_str = result.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        assert_eq!(
            result_str,
            vec!["12", "6", "3", "10", "5", "16", "8", "4", "2", "1"]
        );

    }

    #[test]
    fn first_and_last_u64() {
        let result = calc_sequence_rs("12458674").unwrap();
        assert_eq!(&result[0], "12458674");
        assert_eq!(&result[result.len() - 1], "1");
    }

    #[test]
    fn first_and_last_u256() {
        let result = calc_sequence_rs("18446744073709551616").unwrap();
        assert_eq!(&result[0], "18446744073709551616");
        assert_eq!(&result[result.len() - 1], "1");
    }

    #[test]
    fn first_and_last_bigint() {
        let n = "9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999";
        let result = calc_sequence_rs(n).unwrap();
        assert_eq!(&result[0], n);
        assert_eq!(&result[result.len() - 1], "1");
    }

}

#[cfg(test)]
mod bench {
    use test;

    use super::calc_sequence_bigint;
    use calc_sequence_rs;

    use bigmath::BigUint;

    fn gen_space(mut from: BigUint, to: BigUint, space_size: u64) -> Vec<String> {
        let mut space = Vec::new();
        let step = (to.clone() - from.clone()) / BigUint::from(space_size);
        while from < to {
            space.push(from.to_str_radix(10));
            from = from + step.clone();
        }
        space
    }

    fn gen_small_space() -> Vec<String> {
        gen_space(BigUint::from(2u64), BigUint::from(100000u64), 1000)
    }

    fn gen_large_number() -> String {
        "123456789123456789123456789".to_owned()
    }

    #[bench]
    fn small_space(b: &mut test::Bencher) {
        let space = gen_small_space();
        b.iter(|| for num in &space {
            calc_sequence_rs(&num).unwrap();
        });
    }

    #[bench]
    fn large_number(b: &mut test::Bencher) {
        let number = gen_large_number();
        b.iter(|| { calc_sequence_rs(&number).unwrap(); });
    }


    #[bench]
    fn small_space_bigint(b: &mut test::Bencher) {
        let space = gen_small_space();
        b.iter(|| for num in &space {
            calc_sequence_bigint(&num).unwrap();
        });
    }

    #[bench]
    fn large_number_bigint(b: &mut test::Bencher) {
        let number = gen_large_number();
        b.iter(|| { calc_sequence_bigint(&number).unwrap(); });
    }
}

use std::collections::HashMap;

#[derive(Debug)]
struct ElfPassword {
    min: usize,
    max: usize,
    value: [usize; 6], // this could be its own type
}

impl ElfPassword {
    fn new(min: usize, max: usize) -> Self {
        let mut value = [0; 6];
        (0_usize..6)
            .map(|i| (5 - i, min / 10_usize.pow(i as u32) % 10_usize))
            .for_each(|(i, x)| value[i] = x);

        let mut result = Self { min, value, max };
        result.apply_inscrease_rule();
        result
    }

    // this will fail for range 000000-999999
    fn incr_value(&mut self) -> &mut Self {
        self.value.iter_mut().rev().fold(true, |carry, val| {
            let next_val = if carry { *val + 1 } else { *val };

            if next_val > 9 {
                *val = 0;
                true
            } else {
                *val = next_val;
                false
            }
        });

        self
    }

    fn apply_inscrease_rule(&mut self) -> &mut Self {
        let mut value = self.value.iter_mut();
        let initial = *value.next().unwrap();

        value.fold((false, initial), |(mut found, last), val| {
            if found {
                *val = last;
                return (found, last);
            }

            if *val < last {
                *val = last;
                found = true;
            }

            (found, *val)
        });

        self
    }

    fn incr(&mut self) -> &mut Self {
        self.incr_value().apply_inscrease_rule()
    }

    fn is_in_range(&self) -> bool {
        self.get_value_int() < self.max
    }

    fn get_value_int(&self) -> usize {
        self.value
            .iter()
            .rev()
            .enumerate()
            .map(|(i, val)| val * 10_usize.pow(i as u32))
            .sum()
    }
}

// should probably create an Iter type for this
impl Iterator for ElfPassword {
    type Item = [usize; 6];

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_in_range() {
            let value = self.value;
            self.incr();
            Some(value)
        } else {
            None
        }
    }
}

fn has_double_repeat(value: &[usize; 6]) -> bool {
    value.windows(2).any(|window| window[0] == window[1])
}

fn has_strict_double_repeat(value: &[usize; 6]) -> bool {
    value
        .windows(2)
        .filter(|window| window[0] == window[1])
        .fold(HashMap::new(), |mut map, window| {
            let digit = window[0];
            let counter = map.entry(digit).or_insert(0);
            *counter += 1;

            map
        })
        .iter()
        .any(|(_, count)| *count == 1)
}

fn solve_1((a, b): (usize, usize)) -> usize {
    ElfPassword::new(a, b)
        .filter(has_double_repeat)
        .count()
}

fn solve_2((a, b): (usize, usize)) -> usize {
    ElfPassword::new(a, b)
        .filter(has_strict_double_repeat)
        .count()
}

fn get_input() -> (usize, usize) {
    let text = "130254-678275"; // real-input
    let mut bounds_iter = text.split('-').map(|bound| bound.parse().unwrap());

    (bounds_iter.next().unwrap(), bounds_iter.next().unwrap())
}

fn main() {
    let input = get_input();

    println!("first solution: {:?}", solve_1(input));
    println!("second solution: {:?}", solve_2(input));
}

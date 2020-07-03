use std::{cmp::max, error::Error, fs};

struct FuelIter {
    previous: i32,
}

impl FuelIter {
    fn new(previous: i32) -> Self {
        FuelIter { previous }
    }
}

impl std::iter::Iterator for FuelIter {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let fuel = calculate_fuel(self.previous);
        self.previous = fuel;

        if fuel > 0 {
            Some(fuel)
        } else {
            None
        }
    }
}

fn get_input() -> Result<Vec<i32>, Box<dyn Error>> {
    let result = fs::read_to_string("input.txt")?
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<i32>, std::num::ParseIntError>>()?;

    Ok(result)
}

fn solve_1(input: &[i32]) -> i32 {
    input.iter().map(|&mass| calculate_fuel(mass)).sum()
}

fn solve_2(input: &[i32]) -> i32 {
    input.iter().map(|&mass| calculate_total_fuel(mass)).sum()
}

fn main() {
    let input = get_input().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    println!("first solution: {}", solve_1(&input));
    println!("second solution: {:?}", solve_2(&input));
}

fn calculate_total_fuel(start_mass: i32) -> i32 {
    FuelIter::new(start_mass).sum()
}

fn calculate_fuel(mass: i32) -> i32 {
    max(mass / 3 - 2, 0)
}

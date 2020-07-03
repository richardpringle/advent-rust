use std::{error::Error, fs};

use regex::Regex;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3 {
    fn zero() -> Self {
        Vec3 { x: 0, y: 0, z: 0 }
    }
    fn apply_gravity(&self, other: &Self) -> Self {
        let x = match self.x {
            x if x > other.x => -1,
            x if x < other.x => 1,
            _ => 0,
        };

        let y = match self.y {
            y if y > other.y => -1,
            y if y < other.y => 1,
            _ => 0,
        };

        let z = match self.z {
            z if z > other.z => -1,
            z if z < other.z => 1,
            _ => 0,
        };

        Vec3 { x, y, z }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Moon {
    position: Vec3,
    velocity: Vec3,
}

impl Moon {
    fn new(position: Vec3) -> Self {
        Moon {
            position,
            velocity: Vec3::zero(),
        }
    }

    fn apply_gravity(&mut self, other: &Self) {
        self.velocity += self.position.apply_gravity(&other.position)
    }

    fn step(&mut self) {
        self.position += self.velocity;
    }

    fn get_potential_energy(&self) -> i32 {
        let Vec3 { x, y, z } = self.position;
        x.abs() + y.abs() + z.abs()
    }

    fn get_kinetic_energy(&self) -> i32 {
        let Vec3 { x, y, z } = self.velocity;
        x.abs() + y.abs() + z.abs()
    }

    fn get_total_energy(&self) -> i32 {
        self.get_potential_energy() * self.get_kinetic_energy()
    }
}

fn main() {
    let input = get_input().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let input = parse_input(input);

    println!("first solution: {:?}", solve_1(input.clone(), 1000));
    println!("second solution: {:?}", solve_2(input));
}

fn solve_1(mut moons: Vec<Moon>, steps: usize) -> i32 {
    (0..steps).for_each(|_| {
        (0..4).for_each(|i| {
            let mut moon = moons.remove(i);
            moons
                .iter()
                .for_each(|other_moon| moon.apply_gravity(other_moon));
            moons.insert(i, moon)
        });

        (0..4).for_each(|i| {
            let mut moon = moons.remove(i);
            moon.step();
            moons.insert(i, moon)
        });
    });

    moons.iter().fold(0, |total_energy, moon| {
        total_energy + moon.get_total_energy()
    })
}

fn solve_2(mut moons: Vec<Moon>) -> usize {
    let initial = moons.clone();
    let mut count = 0;

    let x = loop {
        (0..4).for_each(|i| {
            let mut moon = moons.remove(i);
            moons
                .iter()
                .for_each(|other_moon| moon.apply_gravity(other_moon));
            moons.insert(i, moon)
        });

        (0..4).for_each(|i| {
            let mut moon = moons.remove(i);
            moon.step();
            moons.insert(i, moon)
        });

        let is_same = moons
            .iter()
            .zip(initial.iter())
            .all(|(a, b)| a.position.x == b.position.x && a.velocity.x == b.velocity.x);

        count += 1;

        if is_same {
            break count;
        }
    };

    let mut moons = initial.clone();
    let mut count = 0;

    let y = loop {
        (0..4).for_each(|i| {
            let mut moon = moons.remove(i);
            moons
                .iter()
                .for_each(|other_moon| moon.apply_gravity(other_moon));
            moons.insert(i, moon)
        });

        (0..4).for_each(|i| {
            let mut moon = moons.remove(i);
            moon.step();
            moons.insert(i, moon)
        });

        let is_same = moons
            .iter()
            .zip(initial.iter())
            .all(|(a, b)| a.position.y == b.position.y && a.velocity.y == b.velocity.y);

        count += 1;

        if is_same {
            break count;
        }
    };

    let mut moons = initial.clone();
    let mut count = 0;

    let z = loop {
        (0..4).for_each(|i| {
            let mut moon = moons.remove(i);
            moons
                .iter()
                .for_each(|other_moon| moon.apply_gravity(other_moon));
            moons.insert(i, moon)
        });

        (0..4).for_each(|i| {
            let mut moon = moons.remove(i);
            moon.step();
            moons.insert(i, moon)
        });

        let is_same = moons
            .iter()
            .zip(initial.iter())
            .all(|(a, b)| a.position.z == b.position.z && a.velocity.z == b.velocity.z);

        count += 1;

        if is_same {
            break count;
        }
    };

    x / 2 * y / 2 * z / 2
}

fn parse_input(numbers: Vec<isize>) -> Vec<Moon> {
    numbers
        .chunks(3)
        .map(|chunk| Vec3 {
            x: chunk[0] as i32,
            y: chunk[1] as i32,
            z: chunk[2] as i32,
        })
        .map(Moon::new)
        .collect()
}

fn get_input() -> Result<Vec<isize>, Box<dyn Error>> {
    let re = Regex::new(r"^<x=(.*), y=(.*), z=(.*)>$").unwrap();
    Ok(fs::read_to_string("input.txt")?
        .lines()
        .flat_map(|line| re.captures_iter(line))
        .flat_map(|x| (1..=3).map(move |i| x.get(i).unwrap().as_str()))
        .map(|x| x.parse())
        .collect::<Result<Vec<isize>, std::num::ParseIntError>>()?)
}

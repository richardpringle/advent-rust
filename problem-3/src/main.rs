use std::{
    cmp::Ordering, collections::HashSet, error::Error, fs, num::ParseIntError, str::FromStr,
};

#[derive(Debug, Clone)]
enum Step {
    Right(usize),
    Left(usize),
    Up(usize),
    Down(usize),
}

impl FromStr for Step {
    type Err = ParseIntError;

    fn from_str(str: &str) -> Result<Step, Self::Err> {
        let (direction, steps) = str.split_at(1);
        match direction {
            "R" => Ok(Step::Right(steps.parse()?)),
            "L" => Ok(Step::Left(steps.parse()?)),
            "U" => Ok(Step::Up(steps.parse()?)),
            "D" => Ok(Step::Down(steps.parse()?)),
            x => panic!("error parsing input: {}", x),
        }
    }
}

#[derive(Debug, Clone)]
struct StepList(Vec<Step>);

impl StepList {
    fn from_string(str: &str) -> Result<Self, ParseIntError> {
        let step_list = str
            .split(',')
            .map(|str| Step::from_str(str))
            .collect::<Result<_, _>>()?;

        Ok(StepList(step_list))
    }

    fn into_iter(self) -> std::vec::IntoIter<Step> {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord)]
struct Position(isize, isize);

impl Position {
    fn origin() -> Self {
        Self(0, 0)
    }

    fn walk(&self, step: Step) -> Self {
        let Self(x, y) = self;

        match step {
            Step::Right(step) => Self(x + step as isize, *y),
            Step::Left(step) => Self(x - step as isize, *y),
            Step::Up(step) => Self(*x, y + step as isize),
            Step::Down(step) => Self(*x, y - step as isize),
        }
    }

    fn distance_from_origin(&self) -> isize {
        self.0.abs() + self.1.abs()
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let a = self.distance_from_origin();
        let b = other.distance_from_origin();

        a.partial_cmp(&b)
    }
}

#[derive(Clone, Debug)]
struct PositionList(Vec<Position>);

impl PositionList {
    fn from_step_list(step_list: StepList) -> Self {
        let step_positions = step_list
            .into_iter()
            .scan(Position::origin(), |last, step| {
                let new_position = last.walk(step);

                // could do this more efficiently with a 2D array of bools
                let step_range: Vec<Position> = match new_position {
                    Position(x, y) if x > last.0 => {
                        (last.0..x).map(move |x| Position(x, y)).collect()
                    }
                    Position(x, y) if x < last.0 => (x + 1..=last.0)
                        .rev()
                        .map(move |x| Position(x, y))
                        .collect(),
                    Position(x, y) if y > last.1 => {
                        (last.1..y).map(move |y| Position(x, y)).collect()
                    }
                    Position(x, y) if y < last.1 => (y + 1..=last.1)
                        .rev()
                        .map(move |y| Position(x, y))
                        .collect(),
                    _ => panic!("steps weren't parsed properly"),
                };

                *last = new_position;

                Some(step_range)
            })
            .fold(Vec::new(), |mut full_path, mut path| {
                full_path.append(&mut path);
                full_path
            });

        Self(step_positions)
    }

    fn iter(&self) -> std::slice::Iter<Position> {
        self.0.iter()
    }

    fn into_iter(self) -> std::vec::IntoIter<Position> {
        self.0.into_iter()
    }
}

fn get_input() -> Result<(StepList, StepList), Box<dyn Error>> {
    let text = fs::read_to_string("input.txt")?;

    let mut result = text.lines().map(|line| StepList::from_string(line.trim()));

    Ok((result.next().unwrap()?, result.next().unwrap()?))
}

fn solve_1(wire_a: StepList, wire_b: StepList) -> isize {
    PositionList::from_step_list(wire_a)
        .into_iter()
        .collect::<HashSet<Position>>()
        .intersection(
            &PositionList::from_step_list(wire_b)
                .into_iter()
                .collect::<HashSet<Position>>(),
        )
        .filter(|position| **position != Position::origin())
        .map(|position| position.distance_from_origin())
        .min()
        .unwrap()
}

fn solve_2(wire_a: StepList, wire_b: StepList) -> usize {
    let wire_a_positions = PositionList::from_step_list(wire_a);
    let wire_b_positions = PositionList::from_step_list(wire_b);

    let min = &wire_a_positions
        .iter()
        .collect::<HashSet<&Position>>()
        .intersection(&wire_b_positions.iter().collect::<HashSet<&Position>>())
        .filter(|&position| **position != Position::origin())
        .map(|&position| {
            let steps_a = wire_a_positions.iter().position(|x| x == position).unwrap();
            let steps_b = wire_b_positions.iter().position(|x| x == position).unwrap();
            steps_a + steps_b
        })
        .min()
        .unwrap();
    *min
}

fn main() {
    let input = get_input().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    println!(
        "first solution: {:?}",
        solve_1(input.0.clone(), input.1.clone())
    );
    println!("second solution: {:?}", solve_2(input.0, input.1));
}

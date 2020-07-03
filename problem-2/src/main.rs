use std::{
    error::Error,
    fs,
    iter::StepBy,
    ops::{Add, Mul, Range},
};

fn get_input() -> Result<Vec<usize>, Box<dyn Error>> {
    let result = fs::read_to_string("input.txt")?
        .trim()
        .split(',')
        .map(|line| line.parse())
        .collect::<Result<Vec<usize>, std::num::ParseIntError>>()?;

    Ok(result)
}

#[derive(Debug)]
struct IntcodeMachine {
    memory: Vec<usize>,
    cmd_ptr: StepBy<Range<usize>>,
    print: bool,
}

impl IntcodeMachine {
    fn new(memory: Vec<usize>) -> Self {
        let end = memory.len();
        IntcodeMachine {
            memory,
            cmd_ptr: (0..end).step_by(4),
            print: false,
        }
    }

    fn set_noun(mut self, x: usize) -> Self {
        self.memory[1] = x;
        self
    }

    fn set_verb(mut self, x: usize) -> Self {
        self.memory[2] = x;
        self
    }

    fn execute_step(&mut self) -> Option<()> {
        self.cmd_ptr.next().and_then(|index| {
            let cmd = self.get_command(index);
            let args = self.get_args((index + 1, index + 2));
            let result_address = self.memory[index + 3];

            cmd(args).map(|result| {
                self.memory[result_address] = result;
                if self.print {
                    println!("{:?}", self.memory);
                }
            })
        })
    }

    fn run(&mut self) -> usize {
        self.for_each(|()| {});
        self.memory[0]
    }

    fn get_command(&self, index: usize) -> fn((usize, usize)) -> Option<usize> {
        match self.memory[index] {
            1 => Self::add,
            2 => Self::mul,
            99 => Self::halt,
            x => panic!("unknown instruction: {}", x),
        }
    }

    fn get_args(&self, (a, b): (usize, usize)) -> (usize, usize) {
        let address_a = self.memory[a];
        let address_b = self.memory[b];
        (self.memory[address_a], self.memory[address_b])
    }

    fn add((a, b): (usize, usize)) -> Option<usize> {
        Some(usize::add(a, b))
    }

    fn mul((a, b): (usize, usize)) -> Option<usize> {
        Some(usize::mul(a, b))
    }

    fn halt<T>((_, _): (usize, usize)) -> Option<T> {
        None
    }
}

impl Iterator for IntcodeMachine {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.execute_step()
    }
}

fn solve_1(input: &[usize]) -> usize {
    let mut computer = IntcodeMachine::new(input.to_vec()).set_noun(12).set_verb(2);
    computer.run()
}

fn solve_2(input: &[usize]) -> usize {
    let (noun, verb) = (0_usize..=99)
        .flat_map(|i| (0_usize..=99).map(move |j| (i, j)))
        .find(|(i, j)| {
            let result = IntcodeMachine::new(input.to_vec())
                .set_noun(*i)
                .set_verb(*j)
                .run();

            result == 19_690_720
        })
        .unwrap();

    (100 * noun) + verb
}

fn main() {
    let input = get_input().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    println!("first solution: {}", solve_1(&input));
    println!("second solution: {:?}", solve_2(&input));
}

use std::{
    error::Error,
    fs,
    ops::{Add, Mul},
};

fn get_input() -> Result<Vec<isize>, Box<dyn Error>> {
    let result = fs::read_to_string("input.txt")?
        .trim()
        .split(',')
        .map(|line| line.parse())
        .collect::<Result<Vec<isize>, std::num::ParseIntError>>()?;

    Ok(result)
}

#[derive(Debug)]
enum Mode {
    Position,
    Value,
}

impl Mode {
    fn from_code(code: usize) -> Self {
        match code {
            0 => Self::Position,
            1 => Self::Value,
            x => panic!("unknown code: {}", x),
        }
    }
}

#[derive(Debug)]
enum Args<T> {
    Zero,
    One(T),
    Two(T, T),
    Three(T, T, T),
}

type CommandFn<T> = fn(&mut IntcodeMachine, Args<T>) -> Option<()>;
// This probably isn't necessary to have as a separate struct
struct Command<'a, T> {
    machine: &'a mut IntcodeMachine,
    command: CommandFn<T>,
    args: Args<T>,
}

impl<'a> Command<'a, usize> {
    fn apply(self) -> Option<()> {
        let cmd = self.command;
        cmd(self.machine, self.args)
    }
}

#[derive(Debug)]
struct IntcodeMachine {
    memory: Vec<isize>,
    cmd_ptr: usize,
    input: isize,
    output: Vec<isize>,
}

impl IntcodeMachine {
    fn new(memory: Vec<isize>) -> Self {
        IntcodeMachine {
            memory,
            cmd_ptr: 0,
            input: 0,
            output: Vec::new(),
        }
    }

    fn set_input(mut self, x: isize) -> Self {
        self.input = x;
        self
    }

    fn decode_instruction(&mut self) -> (usize, Vec<Mode>) {
        let instruction = self.memory[self.cmd_ptr] as usize;
        self.cmd_ptr += 1;

        let modes = (0..5)
            .skip(2)
            .map(|i| instruction / 10_usize.pow(i) % 10)
            .map(Mode::from_code)
            .collect();

        (instruction % 100, modes)
    }

    fn get_current_memory_slice(&self, opcode: usize) -> &[isize] {
        let arg_count = match opcode {
            1 => 3,
            2 => 3,
            3 => 1,
            4 => 1,
            5 => 2,
            6 => 2,
            7 => 3,
            8 => 3,
            99 => 0,
            x => panic!("unknown opcode: {}", x),
        };

        &self.memory[self.cmd_ptr..(self.cmd_ptr + arg_count)]
    }

    fn build_args(&self, raw_args: &[isize], modes: Vec<Mode>) -> Args<usize> {
        let cmd_ptr = self.cmd_ptr;
        let args = raw_args
            .iter()
            .zip(modes)
            .enumerate()
            .map(|(i, (&raw_arg, mode))| match mode {
                Mode::Position => raw_arg as usize,
                Mode::Value => cmd_ptr + i as usize,
            })
            .collect::<Vec<_>>();

        match args.len() {
            0 => Args::Zero,
            1 => Args::One(args[0]),
            2 => Args::Two(args[0], args[1]),
            3 => Args::Three(args[0], args[1], args[2]),
            _ => panic!("memory slice too big!"),
        }
    }

    fn get_command(&mut self, opcode: usize, args: Args<usize>) -> Command<usize> {
        let machine = self;
        let command = match opcode {
            1 => IntcodeMachine::add,
            2 => IntcodeMachine::mul,
            3 => IntcodeMachine::store,
            4 => IntcodeMachine::push_output,
            5 => IntcodeMachine::jump_if_true,
            6 => IntcodeMachine::jump_if_false,
            7 => IntcodeMachine::less_than,
            8 => IntcodeMachine::equals,
            99 => IntcodeMachine::halt,
            x => panic!("unknown instruction: {}", x),
        };

        Command {
            machine,
            command,
            args,
        }
    }

    fn execute_step(&mut self) -> Option<()> {
        let (opcode, modes) = self.decode_instruction();
        let mem_slice = self.get_current_memory_slice(opcode);
        let args = self.build_args(mem_slice, modes);

        self.get_command(opcode, args).apply()
    }

    fn run(&mut self) -> &Vec<isize> {
        self.fold((), |_, _| ());
        &self.output
    }

    fn add(&mut self, args: Args<usize>) -> Option<()> {
        if let Args::Three(a_pos, b_pos, output_pos) = args {
            self.memory[output_pos] = self.memory[a_pos].add(self.memory[b_pos]);
            self.cmd_ptr += 3;
            Some(())
        } else {
            None
        }
    }

    fn mul(&mut self, args: Args<usize>) -> Option<()> {
        if let Args::Three(a_pos, b_pos, output_pos) = args {
            self.memory[output_pos] = self.memory[a_pos].mul(self.memory[b_pos]);
            self.cmd_ptr += 3;
            Some(())
        } else {
            None
        }
    }

    fn store(&mut self, args: Args<usize>) -> Option<()> {
        if let Args::One(storage_position) = args {
            self.memory[storage_position] = self.input;
            self.cmd_ptr += 1;
            Some(())
        } else {
            None
        }
    }

    fn push_output(&mut self, args: Args<usize>) -> Option<()> {
        if let Args::One(data_position) = args {
            self.output.push(self.memory[data_position]);
            self.cmd_ptr += 1;
            Some(())
        } else {
            None
        }
    }

    fn jump_if_true(&mut self, args: Args<usize>) -> Option<()> {
        if let Args::Two(should_jump_pos, instruction_position) = args {
            self.cmd_ptr = if self.memory[should_jump_pos] != 0 {
                self.memory[instruction_position] as usize
            } else {
                self.cmd_ptr + 2
            };

            Some(())
        } else {
            None
        }
    }

    fn jump_if_false(&mut self, args: Args<usize>) -> Option<()> {
        if let Args::Two(jump_on_zero_pos, instruction_position) = args {
            self.cmd_ptr = if self.memory[jump_on_zero_pos] == 0 {
                self.memory[instruction_position] as usize
            } else {
                self.cmd_ptr + 2
            };

            Some(())
        } else {
            None
        }
    }

    fn less_than(&mut self, args: Args<usize>) -> Option<()> {
        if let Args::Three(a_pos, b_pos, output_pos) = args {
            self.memory[output_pos] = (self.memory[a_pos] < self.memory[b_pos]) as isize;
            self.cmd_ptr += 3;
            Some(())
        } else {
            None
        }
    }

    fn equals(&mut self, args: Args<usize>) -> Option<()> {
        if let Args::Three(a_pos, b_pos, output_pos) = args {
            self.memory[output_pos] = (self.memory[a_pos] == self.memory[b_pos]) as isize;
            self.cmd_ptr += 3;
            Some(())
        } else {
            None
        }
    }

    fn halt<T>(&mut self, _args: Args<T>) -> Option<()> {
        None
    }
}

impl Iterator for IntcodeMachine {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.execute_step()
    }
}

fn solve_1(input: &[isize]) -> Vec<isize> {
    let mut computer = IntcodeMachine::new(input.to_vec()).set_input(1);
    computer.run();
    computer.output
}

fn solve_2(input: &[isize]) -> Vec<isize> {
    let mut computer = IntcodeMachine::new(input.to_vec()).set_input(5);
    computer.run();
    computer.output
}

fn main() {
    let input = get_input().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    println!("first solution: {:?}", solve_1(&input));
    println!("second solution: {:?}", solve_2(&input));
}

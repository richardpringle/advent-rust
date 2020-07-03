use std::{
    collections::VecDeque,
    error::Error,
    fs,
    ops::{Add, Mul},
    sync::mpsc::{channel, Receiver, Sender},
    thread::spawn,
};

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
    phase_setting: Option<isize>,
    input_signal: Receiver<isize>,
    output: Sender<isize>,
}

impl IntcodeMachine {
    fn new(
        program: &[isize],
        phase_setting: isize,
        input_signal: Receiver<isize>,
        output: Sender<isize>,
    ) -> Self {
        IntcodeMachine {
            memory: program.to_vec(),
            cmd_ptr: 0,
            phase_setting: Some(phase_setting),
            input_signal,
            output,
        }
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

    fn run(&mut self) {
        self.for_each(|_| {});
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
            self.memory[storage_position] = self
                .phase_setting
                .take()
                .or_else(|| self.input_signal.recv().ok())
                .unwrap();
            self.cmd_ptr += 1;
            Some(())
        } else {
            None
        }
    }

    fn push_output(&mut self, args: Args<usize>) -> Option<()> {
        if let Args::One(data_position) = args {
            self.output
                .send(self.memory[data_position])
                .expect("oh no!");
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

// Thanks Rosetta code... I solved with my own technique first, but this is much cleaner.
fn generate_permutations<'a, T: Copy>(
    stack: &'a mut Vec<T>,
    queue: &'a mut VecDeque<T>,
    mut output: Vec<Vec<T>>,
) -> Vec<Vec<T>> {
    if queue.is_empty() {
        output.push(stack.clone());
        return output;
    }

    (0..queue.len()).fold(output, |mut output, _| {
        stack.push(queue.pop_front().unwrap());
        output = generate_permutations(stack, queue, output);
        queue.push_back(stack.pop().unwrap());

        output
    })
}

fn get_output(program: &[isize], phase_settings: Vec<isize>) -> isize {
    let (tx, rx) = channel();
    tx.send(0).ok();

    phase_settings
        .into_iter()
        .fold(rx, |output, signal| {
            let (tx, rx) = channel();
            let mut computer = IntcodeMachine::new(program, signal, output, tx);
            computer.run();
            rx
        })
        .recv()
        .unwrap()
}

fn get_output_with_feedback_loop(program: &[isize], phase_settings: Vec<isize>) -> isize {
    let (tx, rx) = channel();

    let (output, computers) =
        phase_settings
            .into_iter()
            .fold((rx, vec![]), |(output, mut computers), signal| {
                let (tx, rx) = channel();
                computers.push(IntcodeMachine::new(program, signal, output, tx));
                (rx, computers)
            });

    tx.send(0).expect("sent-outer");

    computers.into_iter().for_each(|mut computer| {
        spawn(move || computer.run());
    });

    output.into_iter().fold(0, |_, value| {
        tx.send(value).unwrap_or(());
        value
    })
}

fn solve_1(program: &[isize]) -> isize {
    generate_permutations(&mut Vec::new(), &mut (0..=4).collect(), Vec::new())
        .into_iter()
        .map(|settings| get_output(program, settings))
        .max()
        .unwrap()
}

fn solve_2(program: &[isize]) -> isize {
    generate_permutations(&mut Vec::new(), &mut (5..=9).collect(), Vec::new())
        .into_iter()
        .map(|settings| get_output_with_feedback_loop(program, settings))
        .max()
        .unwrap()
}

fn main() {
    let input = get_input().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    println!("first solution: {:?}", solve_1(&input));
    println!("second solution: {:?}", solve_2(&input));
}

fn get_input() -> Result<Vec<isize>, Box<dyn Error>> {
    Ok(fs::read_to_string("input.txt")?
        .trim()
        .split(',')
        .map(|line| line.parse())
        .collect::<Result<Vec<isize>, std::num::ParseIntError>>()?)
}

use std::env;
use std::fs;
use std::fmt;
use std::io::{self,Write};
use std::time::Instant;

#[derive(Debug, Copy, Clone)]
enum LitType {
    Zero,
    One,
    User,
}

impl LitType {
    fn get_literal(&self) -> bool {
        match self {
            LitType::Zero => false,
            LitType::One  => true,
            LitType::User => {
                let mut stdout = io::stdout();
                print!(": ");
                stdout.flush().unwrap();
                for uline in io::stdin().lines() {
                    match uline.unwrap().as_str() {
                        "0" => return false,
                        "1" => return true,
                        _ => {}
                    }
                    print!("Input must be either 0 or 1!\n: ");
                    stdout.flush().unwrap();
                }
                panic!("Please dont input null characters. Its very mean.")
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Code {//  Symbol:
    Imovf, //            +
    Imovb, //            -
    Inand, //            N
    Icopy, //            C
    Iswap, //            S
    Ilite(LitType), //   I (0,1,U)
    Iemit, //            E
    Idele, //            D
    Ibran, //            B
    Ijump, //            ]
    Iopen, //            [
}

impl Code {
    fn changes_arr(&self) -> bool {
        match self {
            Code::Imovf =>  true,
            Code::Imovb =>  true,
            Code::Inand =>  true,
            Code::Icopy =>  true,
            Code::Iswap =>  true,
            Code::Ilite(_) =>  true,
            Code::Iemit =>  false,
            Code::Idele =>  true,
            Code::Ibran =>  false,
            Code::Ijump =>  false,
            Code::Iopen =>  false,
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Code::Imovf =>  "MOVE FORWARD",
            Code::Imovb =>  "MOVE BACKWARD",
            Code::Inand =>  "NAND",
            Code::Icopy =>  "COPY",
            Code::Iswap =>  "SWAP",
            Code::Ilite(_) =>  "LITERAL",
            Code::Iemit =>  "EMIT",
            Code::Idele =>  "DELETE",
            Code::Ibran =>  "BRANCH",
            Code::Ijump =>  "CONTINUE LOOP",
            Code::Iopen =>  "START LOOP",
        })
    }
}

fn parse(s: &String) -> Vec<Code> { 
    let mut prog:Vec<Code> = Vec::new();
    let mut iterator = s.chars();

    while let Some(cur_char) = iterator.next() {

        let next_point = match cur_char {
            '+' => Code::Imovf,
            '-' => Code::Imovb,
            'N' => Code::Inand,
            'C' => Code::Icopy,
            'S' => Code::Iswap,
            'I' => {
                let lit_type = match iterator.next() {
                    Some('0') => LitType::Zero,
                    Some('1') => LitType::One,
                    Some('U') => LitType::User,
                    _ => panic!("Syntax error!: 'I' should always be followed by what to insert (0, 1, or U)."),
                };
                Code::Ilite(lit_type)
            },
            'E' => Code::Iemit,
            'D' => Code::Idele,
            'B' => Code::Ibran,
            ']' => Code::Ijump,
            '[' => Code::Iopen,
            _ => continue,
        };
        prog.push(next_point)
    };
    prog
}

fn display_state(array: &Vec<bool>,arr_ptr:usize) {
    println!("State: {}"
            ,array.iter()
                  .map(|v| {if *v {'1'} else {'0'}})
                  .collect::<String>()
    );
    if array.len() < 70 {
        println!("      {}{}"," ".repeat(arr_ptr), " ^");
    }
}

fn evaluate(prog:Vec<Code>, show:bool) -> (Vec<bool>, usize) {
    let mut bit_arr: Vec<bool> = Vec::new(); // true = 1, false = 0 btw
    let mut arr_ptr = 0usize;

    let mut prog_ptr = 0usize;

    while prog_ptr < prog.len() {
        let instr = &prog[prog_ptr];
        if show {
            println!("Now performing: {}! (Instruction #{})\n", instr, prog_ptr);
        };

        match instr {
            // 0 arg instructions
            Code::Imovf => {
                arr_ptr = arr_ptr + 1;
            },
            Code::Imovb => {
                arr_ptr = arr_ptr.checked_sub(1).expect("Error!: The bit array pointer is out of bounds!");
            },
            Code::Ilite(val) => {
                bit_arr.insert(arr_ptr,val.get_literal());
            },
            Code::Ijump => {
                let mut depth: u8 = 1;
                while depth != 0 {
                    prog_ptr = prog_ptr.checked_sub(1).expect("Error!: Unmatched closing bracket!");
                    match &prog[prog_ptr] {
                        Code::Ijump => {depth += 1},
                        Code::Iopen => {depth -= 1},
                        _ => (),
                    }
                }
            },
            Code::Iopen => (),

            // 1 arg instructions
            Code::Icopy => {
                let a = bit_arr[arr_ptr];
                bit_arr.insert(arr_ptr, a);
            },
            Code::Ibran => {if bit_arr[arr_ptr] {prog_ptr += 1}},
            Code::Idele => {
                bit_arr.remove(arr_ptr);
            },
            Code::Iemit => {
                print!("{}", if bit_arr[arr_ptr] {'1'} else {'0'})
            },

            // 2 arg instructions
            Code::Inand => {
                let a = bit_arr.remove(arr_ptr);
                let b = bit_arr.get_mut(arr_ptr).unwrap();
                *b = !(a & *b);
            },
            Code::Iswap => {
                bit_arr.swap(arr_ptr,arr_ptr + 1);
            },
        }
        if instr.changes_arr() & show {
            display_state(&bit_arr, arr_ptr)
        };
        prog_ptr += 1;
    }
    (bit_arr, arr_ptr)
}

fn main() {
    let commargs: Vec<String> = env::args().collect();
    dbg!(&commargs);
    let file = &commargs[1];

    println!("Looking for file {file}!");

    let contents = fs::read_to_string(file)
        .expect("Unable to read file :(");

    let program:Vec<Code> = parse(&contents);
    
    print!("This program is {} instuctions long!\nDisplay calculations? (0 or 1)",program.len());
    let show_calculations = LitType::User.get_literal();

    let before = Instant::now();
    let (fin_arr, fin_ptr) = evaluate(program, show_calculations);

    println!("\n\nFinal");
    display_state(&fin_arr, fin_ptr);
    //println!("Completed in {} microseconds", before.elapsed().as_micros())
}

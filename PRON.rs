use std::env;
use std::fs;
use std::fmt;
use std::io::{self,Write};
use std::time::Instant;

#[derive(Debug, Copy, Clone)]
enum LitType {
    Zero,
    One,
    Random,
    User,
}

impl LitType {
    fn get_literal(&self,mbnow:Option<Instant>) -> bool {
        match self {
            LitType::Zero => false,
            LitType::One  => true,
            LitType::Random => {
                match mbnow {
                    Some (inst) => {
                        match inst.elapsed().as_micros() % 10 {
                            0 => false,
                            1 => true,
                            2 => false,
                            3 => true,
                            4 => true,
                            5 => false,
                            6 => false,
                            7 => true,
                            8 => false,
                            9 => true,
                            _ => panic!("Somehow the time elapsed is not an integer")
                        }
                    },
                    None => panic!(),
                }
            },
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
    Ilite(LitType), //   I (0,1,R,U)
    Idele, //            D
    Ibran, //            B
    Ijump, //            ]
    Iloop, //            [
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Code::Imovf =>  "MOVF",
            Code::Imovb =>  "MOVB",
            Code::Inand =>  "NAND",
            Code::Icopy =>  "COPY",
            Code::Iswap =>  "SWAP",
            Code::Ilite(_) =>  "LITE",
            Code::Idele =>  "DELE",
            Code::Ibran =>  "BRAN",
            Code::Ijump =>  "JUMP",
            Code::Iloop =>  "LOOP",
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
                    Some('R') => LitType::Random,
                    Some('U') => LitType::User,
                    _ => panic!("Syntax error!: 'I' should always be followed by what to insert (0, 1, R, or U)."),
                };
                Code::Ilite(lit_type)
            },
            'D' => Code::Idele,
            'B' => Code::Ibran,
            ']' => Code::Ijump,
            '[' => Code::Iloop,
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
    let now = Instant::now();

    let mut prog_ptr = 0usize;

    while prog_ptr < prog.len() {
        let instr = &prog[prog_ptr];
        if show {
            display_state(&bit_arr, arr_ptr);
            println!("Now performing: {}! (Instruction #{})\n", instr, prog_ptr);
        };

        match instr {
            // 0 arg instructions
            Code::Imovf => {arr_ptr = arr_ptr + 1},
            Code::Imovb => {arr_ptr = arr_ptr.checked_sub(1).expect("Error!: The bit array pointer is out of bounds!")},
            Code::Ilite(val) => {
                bit_arr.insert(arr_ptr,val.get_literal(Some(now)));
            },
            Code::Ijump => {
                let mut depth: u8 = 1;
                while depth != 0 {
                    prog_ptr = prog_ptr.checked_sub(1).expect("Error!: Unmatched closing bracket!");
                    match &prog[prog_ptr] {
                        Code::Ijump => {depth += 1},
                        Code::Iloop => {depth -= 1},
                        _ => (),
                    }
                }
            },
            Code::Iloop => (),

            // 1 arg instructions
            Code::Icopy => {
                let a = bit_arr[arr_ptr];
                bit_arr.insert(arr_ptr, a);
            },
            Code::Ibran => {if bit_arr[arr_ptr] {prog_ptr += 1}},
            Code::Idele => {bit_arr.remove(arr_ptr);},

            // 2 arg instructions
            Code::Inand => { // could maybe be done in just two calls to the list
                let a = bit_arr.remove(arr_ptr);
                let b = bit_arr.remove(arr_ptr);
                bit_arr.insert(arr_ptr, !(a & b));
            },
            Code::Iswap => { // NEEDS TO CATCH OOB ERROR
                bit_arr.swap(arr_ptr,arr_ptr + 1)
            },
        }
        prog_ptr += 1;
    }
    (bit_arr, arr_ptr)
}

fn main() {
    let commargs: Vec<String> = env::args().collect();
    let file = &commargs[1];

    println!("Looking for file {file}!");

    let contents = fs::read_to_string(file)
        .expect("Unable to read file :(");

    let program:Vec<Code> = parse(&contents);
    
    print!("This program is {} instuctions long!\nDisplay calculations? (0 or 1)",program.len());
    io::stdout().flush().unwrap();
    let (fin_arr, fin_ptr) = evaluate(program,LitType::User.get_literal(None));
    println!("\n\nFinal");
    display_state(&fin_arr, fin_ptr)
}

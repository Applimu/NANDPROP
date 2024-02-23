use std::env;
use std::fs;
use std::fmt;
use std::io::{self,Write};
use std::time::Instant;

enum LitType {
    Zero,
    One,
    Random,
    User,
}

impl LitType {
    fn get(&self,mbnow:Option<Instant>) -> bool {
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
                let mut tmp_str = String::new(); 
                print!(": ");
                io::stdout().flush().unwrap();
                loop {
                    io::stdin()
                        .read_line(&mut tmp_str)
                        .expect("I couldnt read that :(");
                    match tmp_str.trim() {
                        "0" => break false,
                        "1" => break true,
                        _ =>{println!("Input must be either 0 or 1!")}
                    }
                }
            },
        }
    }
}

// h = direction sensitive
enum Code {//   Symbol:
    Imovf, //h    +
    Imovb, //h    -
    Inand, //h    N
    Icopy, //h    C
    Iswap, //h    S
    Ilite(LitType), //h   I (0,1,R,U)
    Idele, //h    D
    Ibran, //     B
    Iflip, //     F
    Ijump, //     ]
    Iloop, //     [
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
            Code::Iflip =>  "FLIP",
            Code::Ijump =>  "JUMP",
            Code::Iloop =>  "LOOP",
        })
    }
}


macro_rules! new_or_break {
    ($x:ident,$y:ident) => {
        match $y.next() {
            Some(c) => {$x = c},
            None => break,
        }
    }
}

fn parse(s: &String) -> Vec<Code> { 
    let mut prog:Vec<Code> = Vec::new();
    let mut iterator = s.chars();
    let mut cur_char: char;

    loop {
        new_or_break!(cur_char,iterator);

        match cur_char {
            '+' => {prog.push(Code::Imovf);continue},
            '-' => {prog.push(Code::Imovb);continue},
            'N' => {prog.push(Code::Inand);continue},
            'C' => {prog.push(Code::Icopy);continue},
            'S' => {prog.push(Code::Iswap);continue},
            'I' => (),
            'D' => {prog.push(Code::Idele);continue}
            'B' => {prog.push(Code::Ibran);continue},
            'F' => {prog.push(Code::Iflip);continue},
            ']' => {prog.push(Code::Ijump);continue},
            '[' => {prog.push(Code::Iloop);continue},
            _ => continue,
        }
        
        new_or_break!(cur_char,iterator);

        prog.push(Code::Ilite(match cur_char {
            '0' => LitType::Zero,
            '1' => LitType::One,
            'R' => LitType::Random,
            'U' => LitType::User,
            _ => panic!("Syntax error!: 'I' should always be followed by what to insert (0 or 1)."),
        }));
    }
    return prog;
}

fn display_state(array: &Vec<bool>,arr_ptr:usize,dir:bool, prog_ptr:usize,code: &Code) {
    let thingy = array.iter()
                      .map(|v:&bool|->char {if *v {'1'} else {'0'}})
                      .collect::<String>();
    println!("State: {}",thingy);
    if array.len() < 70 {
        println!("      {}{}"," ".repeat(arr_ptr),if dir {" ^>"} else {"<^"});
    }
    println!("Now performing: {}! (Instruction #{})\n", code, prog_ptr);
}

macro_rules! short_dec {
    ($x:ident) => {
        $x.checked_sub(1).expect("Error!: The bit array pointer is out of bounds!")
    }
}


fn evaluate(prog:Vec<Code>, show:bool) -> String {
    let mut bit_arr: Vec<bool> = vec![false]; // true = 1, false = 0 btw
    //Id like to be able to start with nothing
    // but this works rn
    let mut a: bool;
    let mut b: bool;
    let mut arr_ptr = 0usize;
    let mut dir: bool = true; // true = right, false = left.
    let now = Instant::now();

    let prog_len = prog.len();
    let mut prog_ptr = 0usize;

    while prog_ptr < prog_len {
        let new_code = &prog[prog_ptr];
        if show {display_state(&bit_arr, arr_ptr, dir, prog_ptr, new_code)};
        if arr_ptr > bit_arr.len() {
            panic!("Error!: The array pointer is not pointing to data!");
        }

        match new_code { // THERE IS CURRENTLY VERY POOR ERROR HANDLING HAPPENING HERE.
            Code::Imovf => {arr_ptr = if dir {arr_ptr + 1} else {short_dec!(arr_ptr)}}, // Make it wrap around?
            Code::Imovb => {arr_ptr = if dir {short_dec!(arr_ptr)} else {arr_ptr + 1}},
            Code::Inand => { // could maybe be done in just two calls to the list
                if !dir {arr_ptr = short_dec!(arr_ptr)};
                a = bit_arr.remove(arr_ptr);
                b = bit_arr.remove(arr_ptr);
                bit_arr.insert(arr_ptr, !(a & b));
            },
            Code::Icopy => {
                a = bit_arr[arr_ptr];
                bit_arr.insert(arr_ptr, a);
                if !dir {arr_ptr += 1};
            },
            Code::Iswap => { // NEEDS TO CATCH OOB ERROR
                bit_arr.swap(arr_ptr,if dir {arr_ptr + 1} else {short_dec!(arr_ptr)})
            },
            Code::Ilite(val) => {
                if !dir {arr_ptr += 1};
                bit_arr.insert(arr_ptr,val.get(Some(now)));
            },
            Code::Idele => {
                bit_arr.remove(arr_ptr);
                if !dir {arr_ptr = short_dec!(arr_ptr)};
            },
            Code::Ibran => {if bit_arr[arr_ptr] {prog_ptr += 1}},
            Code::Iflip => {dir = !dir},
            Code::Ijump => {
                let mut depth: u8 = 1;
                while depth != 0 {
                    prog_ptr = prog_ptr.checked_sub(1).expect("Error!: Attempted to jump to before the program started!");
                    match &prog[prog_ptr] {
                        Code::Ijump => {depth += 1},
                        Code::Iloop => {depth -= 1},
                        _ => (),
                    }
                }
            },
            Code::Iloop => (),
        }
        prog_ptr += 1;
    }
    return bit_arr.iter()
                  .map(|v:&bool|->char {if *v {'1'} else {'0'}})
                  .collect::<String>();
}

fn main() {
    let commargs: Vec<String> = env::args().collect();
    let file = &commargs[1];

    println!("Looking for file {file}!");

    let contents = fs::read_to_string(file)
        .expect("Unable to read file :(");

    let program:Vec<Code> = parse(&contents);
    
    print!("This program is {} instuctions long!\nDisplay calculations?",program.len());
    io::stdout().flush().unwrap();
    println!("FINAL ANSWER: {}",evaluate(program,LitType::User.get(None)));
}

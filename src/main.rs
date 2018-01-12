#![feature(io)]
#![feature(nll)]
#![feature(fnbox)]

mod parser;
mod metacircular;
mod cps;
#[cfg(test)] mod tests;

use std::rc::Rc;
use std::io::{Write, Read};

pub struct Ctx<'a> {
    output: &'a mut Write,
    input: &'a mut Iterator<Item=char>,
    cur_char: Option<char>,
}

impl<'a> Ctx<'a> {
    fn new(output: &'a mut Write, input: &'a mut Iterator<Item=char>) -> Self {
        Ctx {
            output,
            input,
            cur_char: None,
        }
    }
}

// Err(t) means that the computation was prematurely terminated by `et.
pub type EvalResult = Result<Rc<Term>, Rc<Term>>;

pub enum Term {
    K,
    K1(Rc<Term>),
    S,
    S1(Rc<Term>),
    S2(Rc<Term>, Rc<Term>),
    I,
    V,
    D,
    Promise(Rc<Term>),
    Print(char),
    Read,
    CompareRead(char),
    Reprint,
    E,
    C,
    Cont(Rc<Fn(Rc<Term>, &mut Ctx) -> cps::ContResult>),
    Apply(Rc<Term>, Rc<Term>),
}
use Term::*;

impl std::fmt::Debug for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl ToString for Term {
    fn to_string(&self) -> String {
        match *self {
            K => String::from("k"),
            K1(ref t) => format!("k1({})", t.to_string()),
            S => String::from("s"),
            S1(ref t) => format!("s1({})", t.to_string()),
            S2(ref x, ref y) => format!("s1({}, {})", x.to_string(), y.to_string()),
            I => String::from("i"),
            V => String::from("v"),
            D => String::from("d"),
            Promise(ref t) => format!("promise({})", t.to_string()),
            Print(c) => if c == '\n' { String::from("r") } else { format!(".{}", c) }
            Read => String::from("@"),
            CompareRead(c) => format!("?{}", c),
            Reprint => String::from("|"),
            E => String::from("e"),
            C => String::from("c"),
            Cont(_) => String::from("<cont>"),
            Apply(ref f, ref x) => format!("`{}{}", f.to_string(), x.to_string()),
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: ");
        println!(
            "    {} <program.unl>",
            std::env::current_exe().unwrap().file_name().unwrap().to_string_lossy());
        std::process::exit(1);
    }
    let mut input = std::fs::File::open(&args[1]).unwrap();
    let mut program = String::new();
    input.read_to_string(&mut program).unwrap();

    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    let mut it = stdin.lock().chars().map(|c| c.unwrap());
    let mut ctx = Ctx::new(&mut stdout, &mut it);

    let program = parser::parse_str(&program);
    match program {
        Ok(program) => {
            let _ = cps::full_eval(program, &mut ctx);
        }
        Err(e) => {
            println!("Parse error: {}", e);
            std::process::exit(2);
        }
    }
}

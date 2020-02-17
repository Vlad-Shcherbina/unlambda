// TODO: Get rid of this once https://github.com/clap-rs/clap/issues/1349 is fixed.
// https://rust-lang-nursery.github.io/edition-guide/rust-2018/macros/macro-changes.html
#[macro_use] extern crate clap;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod drop;
mod parser;
mod metacircular;
mod cps;
mod small_step;
#[cfg(test)] mod tests;

use std::rc::Rc;
use std::io::{Write, Read};

pub struct Ctx<'a> {
    output: &'a mut dyn Write,
    input: &'a mut dyn Iterator<Item=char>,
    cur_char: Option<char>,
}

impl<'a> Ctx<'a> {
    fn new(output: &'a mut dyn Write, input: &'a mut dyn Iterator<Item=char>) -> Self {
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
    Apply(Rc<Term>, Rc<Term>),

    // only used by CPS interpreter
    Cont(Rc<dyn Fn(Rc<Term>, &mut Ctx) -> cps::ContResult>),

    // only used by small-step interpreter
    ReifiedCont(small_step::Cont)
}
use crate::Term::*;

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
            Cont(_) | ReifiedCont(_) => String::from("<cont>"),
            Apply(ref f, ref x) => format!("`{}{}", f.to_string(), x.to_string()),
        }
    }
}

arg_enum! {
    #[derive(Debug)]
    enum Interpreter {
        MetaCircular,
        CPS,
        SmallStep
    }
}

fn main() {
    let matches = clap::App::new("unlambda")
        .version(crate_version!())
        .arg_from_usage("--time 'Print execution time to stderr'")
        .arg(clap::Arg::from_usage("--interpreter <interpreter>")
            .possible_values(&Interpreter::variants())
            .case_insensitive(true)
            .required(false)
            .default_value("SmallStep"))
        .arg_from_usage("<file-name>")
        .get_matches();

    let file_name = matches.value_of("file-name").unwrap();
    let mut input = std::fs::File::open(file_name).unwrap();
    let mut program = String::new();
    input.read_to_string(&mut program).unwrap();

    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    let mut it = stdin.lock().bytes().map(|b| {
        let b = b.unwrap();
        assert!(b < 128);
        b as char
    });
    let mut ctx = Ctx::new(&mut stdout, &mut it);

    let program = parser::parse_str(&program);
    match program {
        Ok(program) => {
            let start = std::time::Instant::now();
            {
                let interpreter = value_t!(matches.value_of("interpreter"), Interpreter).unwrap();
                let _ = match interpreter {
                    Interpreter::MetaCircular => {
                        if metacircular::contains_c(&program) {
                            eprintln!("Metacircular interpreter does not support call/cc");
                            std::process::exit(1);
                        }
                        metacircular::eval(program, &mut ctx)
                    }
                    Interpreter::CPS => cps::full_eval(program, &mut ctx),
                    Interpreter::SmallStep => small_step::full_eval(program, &mut ctx),
                };
            }
            if matches.is_present("time") {
                eprintln!("It took {}s", start.elapsed().as_secs_f64());
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(2);
        }
    }
}

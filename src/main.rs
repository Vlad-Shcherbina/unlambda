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

#[derive(Debug)]
enum Interpreter {
    MetaCircular,
    Cps,
    SmallStep
}

#[derive(Debug)]
struct Opts {
    file_name: String,
    interpreter: Interpreter,
    time: bool,
}

impl Opts {
    fn parse(mut pargs: pico_args::Arguments) -> Result<Opts, pico_args::Error> {
        let interpreter: Interpreter = pargs.opt_value_from_fn("--interpreter", |s| {
            match s.to_ascii_lowercase().as_str() {
                "metacircular" => Ok(Interpreter::MetaCircular),
                "cps" => Ok(Interpreter::Cps),
                "smallstep" => Ok(Interpreter::SmallStep),
                _ => Err("unrecognized interpreter"),
            }
        })?.unwrap_or(Interpreter::SmallStep);
        let time: bool = pargs.contains("--time");
        let file_name: String = pargs.free_from_str()?;

        let remaining = pargs.finish();
        if !remaining.is_empty() {
            eprintln!("warning: unused arguments {:?}", remaining);
        }
        Ok(Opts {
            file_name,
            interpreter,
            time,
        })
    }
}

const USAGE: &str =
"USAGE:
    unlambda.exe [--time] [--interpreter=...] <file-name>

    --time
        Print execution time to stderr

    --interpreter <interpreter>
        Possible values: MetaCircular, CPS, SmallStep (default)
";

fn main() {
    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) {
        eprintln!("{}", USAGE);
        std::process::exit(0);
    }
    let opts = match Opts::parse(pargs) {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("{}", USAGE);
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let program = std::fs::read_to_string(&opts.file_name).unwrap();

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
                let _ = match opts.interpreter {
                    Interpreter::MetaCircular => {
                        if metacircular::contains_c(&program) {
                            eprintln!("Metacircular interpreter does not support call/cc");
                            std::process::exit(1);
                        }
                        metacircular::eval(program, &mut ctx)
                    }
                    Interpreter::Cps => cps::full_eval(program, &mut ctx),
                    Interpreter::SmallStep => small_step::full_eval(program, &mut ctx),
                };
            }
            if opts.time {
                eprintln!("It took {}s", start.elapsed().as_secs_f64());
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(2);
        }
    }
}

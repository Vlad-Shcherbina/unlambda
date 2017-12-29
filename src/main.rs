#![feature(io)]

use std::rc::Rc;
use std::io::{Write, Read};

struct Ctx<'a> {
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
type EvalResult = Result<Rc<Term>, Rc<Term>>;

#[derive(PartialEq, Eq, Debug)]
enum Term {
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
    Apply(Rc<Term>, Rc<Term>),
}
use Term::*;

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
            Apply(ref f, ref x) => format!("`{}{}", f.to_string(), x.to_string()),
        }
    }
}

fn parse(it: &mut Iterator<Item=char>) -> Term {
    loop {
        return match it.next().unwrap() {
            '`' => Apply(Rc::new(parse(it)), Rc::new(parse(it))),
            'k' => K,
            's' => S,
            'i' => I,
            'v' => V,
            'd' => D,
            'e' => E,
            '.' => Print(it.next().unwrap()),
            'r' => Print('\n'),
            '@' => Read,
            '?' => CompareRead(it.next().unwrap()),
            '|' => Reprint,
            '#' => {
                while it.next().unwrap() != '\n' {}
                continue;
            }
            c if c.is_whitespace() => continue,
            c => unimplemented!("{}", c)
        }
    }
}

fn parse_str(s: &str) -> Rc<Term> {
    let mut it = s.chars();
    let result = parse(&mut it);
    for c in it {
        assert!(c.is_whitespace());
    }
    Rc::new(result)
}

// Never returns Apply(...) term, so eval() is idempotent
// (second call returns the same value and has no IO side effects).
fn eval(term: Rc<Term>, ctx: &mut Ctx) -> EvalResult {
    if let Apply(ref f, ref x) = *term {
        let f = eval(Rc::clone(f), ctx)?;
        if let D = *f {
            return Ok(Rc::new(Promise(Rc::clone(x))));
        }
        return apply(
            f,
            eval(Rc::clone(x), ctx)?, ctx);
    }
    Ok(term)
}

// Never returns Apply(...) term.
fn apply(f: Rc<Term>, x: Rc<Term>, ctx: &mut Ctx) -> EvalResult {
    if let Apply(_, _) = *f {
        panic!();
    }
    if let Apply(_, _) = *x {
        panic!();
    }
    Ok(match *f {
        K => Rc::new(K1(x)),
        K1(ref y) => Rc::clone(y),
        S => Rc::new(S1(x)),
        S1(ref y) => Rc::new(S2(Rc::clone(y), x)),

        // Seems a bit redundant, since x, y, and z are already evaluated.
        // But we can't just write "apply(apply(y, x), apply(z, x))"
        // because apply does not handle d as a special form.
        // See example ```s`kdri in the documentation.
        // eval() is idempotent, so repeated evaluation of x, y, z is fine.
        S2(ref y, ref z) =>
            eval(Rc::new(Apply(
                Rc::new(Apply(Rc::clone(y), Rc::clone(&x))),
                Rc::new(Apply(Rc::clone(z), Rc::clone(&x))))), ctx)?,

        Print(c) => {
            ctx.output.write_fmt(format_args!("{}", c)).unwrap();
            x
        }
        I => x,
        V => Rc::clone(&f),  // TODO: ideally simply move f, but it's borrowed
        E => return Err(x),
        Read => {
            let c = ctx.input.next();
            ctx.cur_char = c;
            let t = match c {
                Some(_) => Rc::new(I),
                None => Rc::new(V),
            };
            eval(Rc::new(Apply(x, t)), ctx)?
        }
        CompareRead(c) => {
            let t = match ctx.cur_char {
                Some(cc) if cc == c => Rc::new(I),
                _ => Rc::new(V),
            };
            eval(Rc::new(Apply(x, t)), ctx)?
        }
        Reprint => {
            let t = match ctx.cur_char {
                Some(c) => Rc::new(Print(c)),
                None => Rc::new(V),
            };
            eval(Rc::new(Apply(x, t)), ctx)?
        }
        D => panic!("should be handled in eval"),

        // Similarly, apply(eval(f), x) instead of eval(`fx)
        // is probably incorrect. What if f = Promise(D)?
        Promise(ref f) => eval(Rc::new(Apply(Rc::clone(f), x)), ctx)?,

        Apply(_, _) => panic!("should be handled by eval()")
    })
}

fn main() {
    let program = parse_str("``````````````.H.e.l.l.o.,. .w.o.r.l.d.!rv");
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    let mut it = stdin.lock().chars().map(|c| c.unwrap());
    let mut ctx = Ctx::new(&mut stdout, &mut it);
    let t = eval(program, &mut ctx);
    assert_eq!(t.unwrap().to_string(), "v");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_to_string() {
        assert_eq!(parse_str("  `r` `kv`. s  ").to_string(), "`r``kv`. s");
        assert_eq!(parse_str("`k  # comment
                               v").to_string(), "`kv");
    }

    fn run_and_expect(program: &str, result: Option<&str>, output: Option<&str>) {
        run_with_input_and_expect(program, "", result, output, None);
    }

    fn run_with_input_and_expect(
            program: &str, input: &str,
            result: Option<&str>, output: Option<&str>, remaining_input: Option<&str>) {
        let mut buf = Vec::<u8>::new();
        let mut input_it = input.chars();
        let actual_result = {
            let mut ctx = Ctx::new(&mut buf, &mut input_it);
            eval(parse_str(program), &mut ctx)
                .unwrap_or_else(|e| e)
                .to_string()
        };
        if let Some(result) = result {
            assert_eq!(&actual_result.to_string(), result);
        }
        if let Some(output) = output {
            assert_eq!(std::str::from_utf8(&buf).unwrap(), output);
        }
        if let Some(remaining_input) = remaining_input {
            let actual_rimaining_input: String = input_it.collect();
            assert_eq!(actual_rimaining_input, remaining_input);
        }
    }

    #[test]
    fn test_eval() {
        run_and_expect("`.a``ks.b", Some("s"), Some("a"));

        run_and_expect("``ksv", Some("s"), None);
        run_and_expect("```skss", Some("s"), None);

        run_and_expect("`ir", Some("r"), Some(""));
        run_and_expect("`ri", Some("i"), Some("\n"));

        run_and_expect("`vs", Some("v"), None);

        run_and_expect(
            "``````````````.H.e.l.l.o.,. .w.o.r.l.d.!rv",
            None, Some("Hello, world!\n"));

        // From the documentation on d
        run_and_expect("`d`ri", None, Some(""));
        run_and_expect("``d`rii", None, Some("\n"));
        run_and_expect("``dd`ri", None, Some("\n"));
        run_and_expect("``id`ri", None, Some(""));
        run_and_expect("```s`kdri", None, Some(""));

        run_and_expect("``ii`.av", Some("v"), Some("a"));
        run_and_expect("``ei`.av", Some("i"), Some(""));
    }

    #[test]
    fn test_input() {
        run_with_input_and_expect("@", "zzz", None, None, Some("zzz"));

        run_with_input_and_expect("`@i", "", Some("v"), None, Some(""));
        run_with_input_and_expect("`@i", "a", Some("i"), None, Some(""));
        run_with_input_and_expect("``@i`?ai", "a", Some("i"), None, Some(""));
        run_with_input_and_expect("``@i`?bi", "a", Some("v"), None, Some(""));
        run_with_input_and_expect("`?ai", "a", Some("v"), None, Some("a"));

        run_with_input_and_expect("```@i`|ik", "ab", Some("k"), Some("a"), Some("b"));
    }

    #[test]
    fn ramanujan() {
        // From the documentation
        let mut expected = "*".repeat(1729);
        expected.push('\n');
        run_and_expect("
        ```s`kr``s``si`k.*`ki
         ```s``s`k``si`k`s``s`ksk``s``s`ksk``s``s`kski
           ``s`k``s``s`ksk``s``s`kski`s``s`ksk
          ```s``s`kski``s``s`ksk``s``s`kski
        ", None, Some(&expected));
    }
}

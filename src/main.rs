use std::rc::Rc;
use std::io::Write;

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
fn eval(term: Rc<Term>, io: &mut Write) -> EvalResult {
    if let Apply(ref f, ref x) = *term {
        let f = eval(Rc::clone(f), io)?;
        if let D = *f {
            return Ok(Rc::new(Promise(Rc::clone(x))));
        }
        return apply(
            f,
            eval(Rc::clone(x), io)?, io);
    }
    Ok(term)
}

// Never returns Apply(...) term.
fn apply(f: Rc<Term>, x: Rc<Term>, io: &mut Write) -> EvalResult {
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
                Rc::new(Apply(Rc::clone(z), Rc::clone(&x))))), io)?,

        Print(c) => {
            io.write_fmt(format_args!("{}", c)).unwrap();
            x
        }
        I => x,
        V => Rc::clone(&f),  // TODO: ideally simply move f, but it's borrowed
        E => return Err(x),
        D => panic!("should be handled in eval"),

        // Similarly, apply(eval(f), x) instead of eval(`fx)
        // is probably incorrect. What if f = Promise(D)?
        Promise(ref f) => eval(Rc::new(Apply(Rc::clone(f), x)), io)?,

        _ => unimplemented!("{:?}", f),
    })
    }

fn main() {
    let program = parse_str("``````````````.H.e.l.l.o.,. .w.o.r.l.d.!rv");
    let t = eval(program, &mut std::io::stdout());
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
        let mut buf = Vec::<u8>::new();
        let actual_result = eval(parse_str(program), &mut buf)
            .unwrap_or_else(|e| e)
            .to_string();
        if let Some(result) = result {
            assert_eq!(&actual_result.to_string(), result);
        }
        if let Some(output) = output {
            assert_eq!(std::str::from_utf8(&buf).unwrap(), output);
        }
    }

    #[test]
    fn test_eval() {
        run_and_expect("`.a``ks.b" , Some("s"), Some("a"));

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

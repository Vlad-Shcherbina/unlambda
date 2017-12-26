use std::rc::Rc;
use std::io::Write;

#[derive(PartialEq, Eq, Debug)]
enum Term {
    K,
    K1(Rc<Term>),
    S,
    S1(Rc<Term>),
    S2(Rc<Term>, Rc<Term>),
    I,
    V,
    Print(char),
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
            Print(c) => if c == '\n' { String::from("r") } else { format!(".{}", c) }
            Apply(ref f, ref x) => format!("`{}{}", f.to_string(), x.to_string()),
        }
    }
}

fn parse(it: &mut Iterator<Item=char>) -> Term {
    // TODO: whitespaces, comments
    match it.next().unwrap() {
        '`' => Apply(Rc::new(parse(it)), Rc::new(parse(it))),
        'k' => K,
        's' => S,
        'i' => I,
        'v' => V,
        '.' => Print(it.next().unwrap()),
        'r' => Print('\n'),
        _ => unimplemented!(),
    }
}

fn parse_str(s: &str) -> Rc<Term> {
    let mut it = s.chars();
    let result = parse(&mut it);
    assert!(it.next().is_none());
    Rc::new(result)
}

fn eval(term: Rc<Term>, io: &mut Write) -> Rc<Term> {
    match *term {
        Apply(ref f, ref x) =>
            return apply(
                eval(f.clone(), io),
                eval(x.clone(), io), io),
        _ => ()
    }
    term
}

fn apply(f: Rc<Term>, x: Rc<Term>, io: &mut Write) -> Rc<Term> {
    match *f {
        K => Rc::new(K1(x)),
        K1(ref y) => y.clone(),
        S => Rc::new(S1(x)),
        S1(ref y) => Rc::new(S2(y.clone(), x)),
        S2(ref y, ref z) =>
            apply(
                apply(y.clone(), x.clone(), io),
                apply(z.clone(), x.clone(), io), io),
        Print(c) => {
            io.write_fmt(format_args!("{}", c)).unwrap();
            x
        }
        _ => unimplemented!("{:?}", f),
    }
}

fn main() {
    let program = parse_str("``````````````.H.e.l.l.o.,. .w.o.r.l.d.!rv");
    let t = eval(program, &mut std::io::stdout());
    assert_eq!(t.to_string(), "v");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_to_string() {
        assert_eq!(parse_str("`r``kv`.as").to_string(), "`r``kv`.as");
    }

    #[test]
    fn test_eval() {
        let mut buf = Vec::<u8>::new();
        assert_eq!(eval(parse_str("`.a``ks.b"), &mut buf).to_string(), "s");
        let buf = std::str::from_utf8(&buf).unwrap();
        assert_eq!(buf, "a");
    }
}

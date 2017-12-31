use std::rc::Rc;
use Term;
use Term::*;

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

pub fn parse_str(s: &str) -> Rc<Term> {
    let mut it = s.chars();
    let result = parse(&mut it);
    for c in it {
        assert!(c.is_whitespace());
    }
    Rc::new(result)
}

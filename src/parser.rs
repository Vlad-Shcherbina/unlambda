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
                skip_comment(it);
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
    while let Some(c) = it.next() {
        match c {
            '#' => skip_comment(&mut it),
            c if c.is_whitespace() => {}
            c => panic!("unexpected {}", c)
        }
    }
    Rc::new(result)
}

fn skip_comment(it: &mut Iterator<Item=char>) {
    while it.next().unwrap_or('\n') != '\n' {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_to_string() {
        assert_eq!(parse_str("  `r` `kv`. s  ").to_string(), "`r``kv`. s");
        assert_eq!(parse_str("`k  # comment
                                v").to_string(), "`kv");

        assert_eq!(parse_str("`kv  # comment
                                ").to_string(), "`kv");
        assert_eq!(parse_str("`kv  # comment").to_string(), "`kv");
    }
}

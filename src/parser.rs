use std::rc::Rc;
use Term;
use Term::*;

fn parse(it: &mut Iterator<Item=char>) -> Result<Term, String> {
    loop {
        return Ok(match it.next() {
            None => return Err("unexpected EOF".to_string()),
            Some('`') => Apply(Rc::new(parse(it)?), Rc::new(parse(it)?)),
            Some('k') => K,
            Some('s') => S,
            Some('i') => I,
            Some('v') => V,
            Some('d') => D,
            Some('e') => E,
            Some('c') => C,
            Some('.') => Print(it.next().ok_or("unexpected EOF after '.'")?),
            Some('r') => Print('\n'),
            Some('@') => Read,
            Some('?') => CompareRead(it.next().ok_or("unexpected EOF after '?'")?),
            Some('|') => Reprint,
            Some('#') => {
                skip_comment(it);
                continue;
            }
            Some(c) if c.is_whitespace() => continue,
            Some(c) => return Err(format!("unrecognized {:?}", c))
        })
    }
}

pub fn parse_str(s: &str) -> Result<Rc<Term>, String> {
    let mut it = s.chars();
    let result = parse(&mut it)?;
    while let Some(c) = it.next() {
        match c {
            '#' => skip_comment(&mut it),
            c if c.is_whitespace() => {}
            c => return Err(format!("unexpected {:?}", c))
        }
    }
    Ok(Rc::new(result))
}

fn skip_comment(it: &mut Iterator<Item=char>) {
    while it.next().unwrap_or('\n') != '\n' {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors() {
        assert_eq!(parse_str("").unwrap_err(), "unexpected EOF");
        assert_eq!(parse_str("  ").unwrap_err(), "unexpected EOF");
        assert_eq!(parse_str("`k").unwrap_err(), "unexpected EOF");
        assert_eq!(parse_str(".").unwrap_err(), "unexpected EOF after '.'");
        assert_eq!(parse_str("`s?").unwrap_err(), "unexpected EOF after '?'");

        assert_eq!(parse_str("z").unwrap_err(), "unrecognized 'z'");
        assert_eq!(parse_str("`kks").unwrap_err(), "unexpected 's'");
    }

    #[test]
    fn parse_and_to_string() {
        assert_eq!(parse_str("  `r` `kv`. s  ").unwrap().to_string(), "`r``kv`. s");
        assert_eq!(parse_str("`k  # comment
                                v").unwrap().to_string(), "`kv");

        assert_eq!(parse_str("`kv  # comment
                                ").unwrap().to_string(), "`kv");
        assert_eq!(parse_str("`kv  # comment").unwrap().to_string(), "`kv");
    }
}

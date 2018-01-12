use std::rc::Rc;
use Term;
use Term::*;

pub fn parse_str(s: &str) -> Result<Rc<Term>, String> {
    let mut path: Vec<Option<Rc<Term>>> = Vec::new();
    let mut it = s.chars();
    let result;
    'outer: loop {
        let leaf = Rc::new(match it.next() {
            None => Err("unexpected EOF")?,
            Some('`') => {
                path.push(None);
                continue;
            }
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
                skip_comment(&mut it);
                continue;
            }
            Some(c) if c.is_whitespace() => continue,
            Some(c) => Err(format!("unrecognized {:?}", c))?,
        });
        let mut subtree = leaf;
        loop {
            match path.pop() {
                None => {
                    result = subtree;
                    break 'outer;
                }
                Some(None) => {
                    path.push(Some(subtree));
                    break;
                }
                Some(Some(left)) => subtree = Rc::new(Apply(left, subtree)),
            }
        }
    }

    while let Some(c) = it.next() {
        match c {
            '#' => skip_comment(&mut it),
            c if c.is_whitespace() => {}
            c => Err(format!("unexpected {:?}", c))?
        }
    }

    Ok(result)
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

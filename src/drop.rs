// Default recursive drop overflows stack.

use crate::Term;
use crate::Term::*;
use std::rc::Rc;
use crate::small_step::ContEntry::*;

unsafe fn raw_copy<T>(src: &T) -> T {
    std::mem::transmute_copy(src)
}

fn deconstruct_term(mut t: Term, terms: &mut Vec<Rc<Term>>) {
    unsafe {
        match t {
            K1(ref mut x) | S1(ref mut x) | Promise(ref mut x) =>
                terms.push(raw_copy(x)),
            S2(ref mut x, ref mut y) | Apply(ref mut x, ref mut y) => {
                terms.push(raw_copy(x));
                terms.push(raw_copy(y));
            }
            ReifiedCont(ref mut c) => {
                while let Some(ce) = c.try_pop_unwrap() {
                    match ce {
                        Cont1(x) | Cont2(x) => terms.push(x),
                    }
                }
                drop(raw_copy(c))
            },
            Cont(ref mut c) =>
                // no support for non-recursive closure drop()
                drop(raw_copy(c)),

            K | S | I | V | D | E | C | Read | Reprint |
            Print(_) | CompareRead(_) => {}
        }
        std::mem::forget(t);
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        let mut terms = Vec::new();
        deconstruct_term(std::mem::replace(self, K), &mut terms);
        while let Some(p) = terms.pop() {
            if let Ok(t) = Rc::try_unwrap(p) {
                deconstruct_term(t, &mut terms);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_term() {
        let mut t = Rc::new(Term::K);
        for _ in 0..1_000_000 {
            t = Rc::new(Term::Apply(Rc::clone(&t), t));
        }
    }

    #[test]
    fn deep_reified_cont() {
        use crate::small_step::Cont;

        let mut c = Cont::new();
        for _ in 0..1_000_000 {
            let t = Rc::new(Term::ReifiedCont(c));
            c = Cont::new();
            c.push(Cont1(t));
        }
    }
}

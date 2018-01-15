// Default recursive drop overflows stack.

use Term;
use Term::*;
use std::rc::Rc;
use std::mem;
use small_step::Cont;
use small_step::Cont::*;

fn deconstruct_term(mut t: Term, terms: &mut Vec<Rc<Term>>, conts: &mut Vec<Rc<Cont>>) {
    unsafe {
        match t {
            K1(ref mut x) =>
                terms.push(mem::replace(x, mem::uninitialized())),
            S1(ref mut x) =>
                terms.push(mem::replace(x, mem::uninitialized())),
            S2(ref mut x, ref mut y) => {
                terms.push(mem::replace(x, mem::uninitialized()));
                terms.push(mem::replace(y, mem::uninitialized()));
            }
            Promise(ref mut x) =>
                terms.push(mem::replace(x, mem::uninitialized())),
            Apply(ref mut f, ref mut x) => {
                terms.push(mem::replace(f, mem::uninitialized()));
                terms.push(mem::replace(x, mem::uninitialized()));
            }
            ReifiedCont(ref mut c) =>
                conts.push(mem::replace(c, mem::uninitialized())),
            Cont(ref mut c) =>
                // no support for non-recursive closure drop()
                drop(mem::replace(c, mem::uninitialized())),

            K | S | I | V | D | E | C | Read | Reprint |
            Print(_) | CompareRead(_) => {}
        }
        mem::forget(t);
    }
}

fn deconstruct_cont(mut c: Cont, terms: &mut Vec<Rc<Term>>, conts: &mut Vec<Rc<Cont>>) {
    unsafe {
        match c {
            Cont1(ref mut t, ref mut c) |
            Cont2(ref mut t, ref mut c) => {
                terms.push(mem::replace(t, mem::uninitialized()));
                conts.push(mem::replace(c, mem::uninitialized()));
            }
            Eval(ref mut c) =>
                conts.push(mem::replace(c, mem::uninitialized())),
            Cont0 => {}
        }
        mem::forget(c);
    }
}

fn devour(mut terms: Vec<Rc<Term>>, mut conts: Vec<Rc<Cont>>) {
    loop {
        while let Some(p) = terms.pop() {
            if let Ok(t) = Rc::try_unwrap(p) {
                deconstruct_term(t, &mut terms, &mut conts);
            }
        }
        if conts.is_empty() {
            break;
        }
        while let Some(p) = conts.pop() {
            if let Ok(c) = Rc::try_unwrap(p) {
                deconstruct_cont(c, &mut terms, &mut conts);
            }
        }
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        let mut terms = Vec::new();
        let mut conts = Vec::new();
        deconstruct_term(mem::replace(self, K), &mut terms, &mut conts);
        devour(terms, conts);
    }
}

impl Drop for Cont {
    fn drop(&mut self) {
        let mut terms = Vec::new();
        let mut conts = Vec::new();
        deconstruct_cont(mem::replace(self, Cont0), &mut terms, &mut conts);
        devour(terms, conts);
    }
}

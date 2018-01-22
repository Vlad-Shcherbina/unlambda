use Ctx;
use EvalResult;
use Term;
use Term::*;
use std::rc::Rc;
use rc_stack::RcStack;

#[derive(Clone, Debug)]
pub enum ContEntry {
    Cont1(Rc<Term>),
    Cont2(Rc<Term>),
}
use self::ContEntry::*;

pub type Cont = RcStack<ContEntry>;

type ContResult = Result<(Cont, Rc<Term>), EvalResult>;

/*
Call graph:
    eval          calls  eval_of_apply, cont
    eval_of_apply calls  eval
    apply         calls  eval_of_apply, cont
    cont1         calls  eval, cont
    cont2         calls  apply
    cont          is     cont0, cont1, cont2

To break recursion, the following calls are mediated by the outer loop:
    eval -> cont
    apply -> cont
    cont1 -> cont

Recursion  eval -> eval_of_apply -> eval  is implemented as a loop in eval().
*/

fn resume(mut cont: Cont, value: Rc<Term>, ctx: &mut Ctx) -> ContResult {
    match cont.pop_clone() {
        None /* cont0 */ => Err(Ok(value)),
        Some(Cont1(ref x)) => {
            let ef = value;
            if let D = *ef {
                Ok((cont, Rc::new(Promise(Rc::clone(x)))))
            } else {
                cont.push(Cont2(ef));
                eval(Rc::clone(x), cont)
            }
        }
        Some(Cont2(ref ef)) => apply(Rc::clone(ef), value, cont, ctx),
    }
}

fn eval(mut term: Rc<Term>, mut cont: Cont) -> ContResult {
    // this loop always terminates (terms have finite depth),
    // but it's not constant time, so perhaps technically
    // this isn't a small-step interpreter anymore
    while let Apply(ref f, ref x) = *term {
        cont.push(Cont1(Rc::clone(x)));
        term = Rc::clone(f);
    }
    Ok((cont, term))
}

// equivalent to eval(Apply(f, x))
fn eval_of_apply(f: Rc<Term>, x: Rc<Term>, mut cont: Cont) -> ContResult {
    cont.push(Cont1(x));
    eval(f, cont)
}

fn apply(f: Rc<Term>, x: Rc<Term>, cont: Cont, ctx: &mut Ctx) -> ContResult {
    if let Apply(_, _) = *f {
        panic!();
    }
    if let Apply(_, _) = *x {
        panic!();
    }

    let result = match *f {
        K => Rc::new(K1(x)),
        K1(ref y) => Rc::clone(y),
        S => Rc::new(S1(x)),
        S1(ref y) => Rc::new(S2(Rc::clone(y), x)),

        S2(ref y, ref z) => {
            return eval_of_apply(
                Rc::new(Apply(Rc::clone(y), Rc::clone(&x))),
                Rc::new(Apply(Rc::clone(z), Rc::clone(&x))), cont);
        }

        Print(c) => {
            ctx.output.write_fmt(format_args!("{}", c)).unwrap();
            x
        }
        I => x,
        V => f,
        E => return Err(Err(x)),
        Read => {
            let c = ctx.input.next();
            ctx.cur_char = c;
            let t = match c {
                Some(_) => Rc::new(I),
                None => Rc::new(V),
            };
            return eval_of_apply(x, t, cont);
        }
        CompareRead(c) => {
            let t = match ctx.cur_char {
                Some(cc) if cc == c => Rc::new(I),
                _ => Rc::new(V),
            };
            return eval_of_apply(x, t, cont);
        }
        Reprint => {
            let t = match ctx.cur_char {
                Some(c) => Rc::new(Print(c)),
                None => Rc::new(V),
            };
            return eval_of_apply(x, t, cont);
        }
        D => panic!("should be handled in eval"),

        Promise(ref f) => {
            return eval_of_apply(Rc::clone(f), x, cont);
        }

        C => {
            let c = Rc::new(Term::ReifiedCont(RcStack::clone(&cont)));
            return eval_of_apply(x, c, cont);
        }
        ReifiedCont(ref cont) => {
            return Ok((RcStack::clone(cont), x));
        }

        Cont(_) => panic!("not supported!"),

        Apply(_, _) => panic!("should be handled by eval()")
    };
    Ok((cont, result))
}

pub fn full_eval(term: Rc<Term>, ctx: &mut Ctx) -> EvalResult {
    let cont0 = RcStack::new();
    let mut r: ContResult = eval(term, cont0);
    loop {
        match r {
            Ok((cont, term)) =>
                r = resume(cont, term, ctx),
            Err(result) => return result,
        }
    }
}

use Ctx;
use EvalResult;
use Term;
use Term::*;
use std::rc::Rc;

#[derive(Debug)]
pub enum Cont {
    Cont0,
    Cont1(Rc<Term>, Rc<Cont>),
    Cont2(Rc<Term>, Rc<Cont>),
    Eval(Rc<Cont>),
}

type ContResult = Result<(Rc<Cont>, Rc<Term>), EvalResult>;

/*
Call graph:
    eval          calls  eval_of_apply, cont
    eval_of_apply calls  eval
    apply         calls  eval_of_apply, cont
    cont1         calls  eval, cont
    cont2         calls  apply
    cont          is     cont0, cont1, cont2, eval

To break recursion, the following calls are mediated by the outer loop:
    eval -> cont
    apply -> cont
    cont1 -> cont

Recursion  eval -> eval_of_apply -> eval  is implemented as a loop in eval().
*/

fn resume(cont: Rc<Cont>, value: Rc<Term>, ctx: &mut Ctx) -> ContResult {
    match *cont {
        Cont::Cont0 => Err(Ok(value)),
        Cont::Cont1(ref x, ref c) => {
            let ef = value;
            if let D = *ef {
                Ok((Rc::clone(c), Rc::new(Promise(Rc::clone(x)))))
            } else {
                let c2 = Cont::Cont2(ef, Rc::clone(c));
                eval(Rc::clone(x), Rc::new(c2))
            }
        }
        Cont::Cont2(ref ef, ref c) => apply(Rc::clone(ef), value, Rc::clone(c), ctx),
        Cont::Eval(ref cont) => eval(value, Rc::clone(cont)),
    }
}

fn eval(mut term: Rc<Term>, mut cont: Rc<Cont>) -> ContResult {
    // this loop always terminates (terms have finite depth),
    // but it's not constant time, so perhaps technically
    // this isn't a small-step interpreter anymore
    while let Apply(ref f, ref x) = *term {
        let c1 = Cont::Cont1(Rc::clone(x), cont);
        cont = Rc::new(Cont::Eval(Rc::new(c1)));
        term = Rc::clone(f);
    }
    Ok((cont, term))
}

// equivalent to eval(Apply(f, x))
fn eval_of_apply(f: Rc<Term>, x: Rc<Term>, cont: Rc<Cont>) -> ContResult {
    let c1 = Cont::Cont1(x, cont);
    eval(f, Rc::new(c1))
}

fn apply(f: Rc<Term>, x: Rc<Term>, cont: Rc<Cont>, ctx: &mut Ctx) -> ContResult {
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
            let c = Rc::new(Term::ReifiedCont(Rc::clone(&cont)));
            return eval_of_apply(x, c, cont);
        }
        ReifiedCont(ref cont) => {
            return Ok((Rc::clone(cont), x));
        }

        Cont(_) => panic!("not supported!"),

        Apply(_, _) => panic!("should be handled by eval()")
    };
    Ok((cont, result))
}

pub fn full_eval(term: Rc<Term>, ctx: &mut Ctx) -> EvalResult {
    let mut r: ContResult = Ok((Rc::new(Cont::Eval(Rc::new(Cont::Cont0))), term));
    loop {
        match r {
            Ok((ref cont, ref term)) =>
                r = resume(Rc::clone(cont), Rc::clone(term), ctx),
            Err(result) => return result,
        }
    }
}

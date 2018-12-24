use crate::Ctx;
use crate::EvalResult;
use crate::Term;
use crate::Term::*;
use std::rc::Rc;

// Never returns Apply(...) term, so eval() is idempotent
// (second call returns the same value and has no IO side effects).
pub fn eval(term: Rc<Term>, ctx: &mut Ctx) -> EvalResult {
    if let Apply(ref f, ref x) = *term {
        let ef = eval(Rc::clone(f), ctx)?;
        if let D = *ef {
            return Ok(Rc::new(Promise(Rc::clone(x))));
        }
        return apply(
            ef,
            eval(Rc::clone(x), ctx)?, ctx);
    }
    Ok(term)
}

// Never returns Apply(...) term.
fn apply(f: Rc<Term>, x: Rc<Term>, ctx: &mut Ctx) -> EvalResult {
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
                Rc::new(Apply(Rc::clone(z), Rc::clone(&x))))), ctx)?,

        Print(c) => {
            ctx.output.write_fmt(format_args!("{}", c)).unwrap();
            x
        }
        I => x,
        V => f,
        E => return Err(x),
        Read => {
            let c = ctx.input.next();
            ctx.cur_char = c;
            let t = match c {
                Some(_) => Rc::new(I),
                None => Rc::new(V),
            };
            eval(Rc::new(Apply(x, t)), ctx)?
        }
        CompareRead(c) => {
            let t = match ctx.cur_char {
                Some(cc) if cc == c => Rc::new(I),
                _ => Rc::new(V),
            };
            eval(Rc::new(Apply(x, t)), ctx)?
        }
        Reprint => {
            let t = match ctx.cur_char {
                Some(c) => Rc::new(Print(c)),
                None => Rc::new(V),
            };
            eval(Rc::new(Apply(x, t)), ctx)?
        }
        D => panic!("should be handled in eval"),

        // Similarly, apply(eval(f), x) instead of eval(`fx)
        // is probably incorrect. What if f = Promise(D)?
        Promise(ref f) => eval(Rc::new(Apply(Rc::clone(f), x)), ctx)?,

        C => panic!("unsupported"),
        Cont(_) => panic!("unsupported"),
        ReifiedCont(_) => panic!("unsupported"),

        Apply(_, _) => panic!("should be handled by eval()")
    })
}

pub fn contains_c(t: &Term) -> bool {
    let mut q = vec![t];
    while let Some(t) = q.pop() {
        match *t {
            C => return true,
            Apply(ref f, ref x) => {
                q.push(f);
                q.push(x);
            }
            _ => {}
        }
    }
    false
}

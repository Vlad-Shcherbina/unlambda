use std::cell::RefCell;
use Ctx;
use EvalResult;
use Term;
use Term::*;
use std::rc::Rc;

// mechanically derived from metacircular::eval()
fn eval(
    term: Rc<Term>, ctx: &mut Ctx,
    cont: Rc<Fn(Rc<Term>, &mut Ctx)>, abort: Rc<Fn(Rc<Term>)>,
) {
    if let Apply(ref f, ref x) = *term {
        eval(Rc::clone(f), ctx, Rc::new({
            let x = Rc::clone(x);
            let abort = Rc::clone(&abort);
            move |ef: Rc<Term>, ctx: &mut Ctx| {
                if let D = *ef {
                    cont(Rc::new(Promise(Rc::clone(&x))), ctx);
                    return;
                } else {
                    eval(Rc::clone(&x), ctx, Rc::new({
                        let cont = Rc::clone(&cont);
                        let abort = Rc::clone(&abort);
                        move |ex: Rc<Term>, ctx: &mut Ctx| {
                            apply(Rc::clone(&ef), ex, ctx, Rc::clone(&cont), Rc::clone(&abort));
                        }
                    }), Rc::clone(&abort));
                }
            }
        }), abort);
    } else {
        cont(term, ctx);
    }
}

// mechanically derived from metacircular::apply()
fn apply(
    f: Rc<Term>, x: Rc<Term>, ctx: &mut Ctx,
    cont: Rc<Fn(Rc<Term>, &mut Ctx)>, abort: Rc<Fn(Rc<Term>)>,
) {
    if let Apply(_, _) = *f {
        panic!();
    }
    if let Apply(_, _) = *x {
        panic!();
    }

    cont(match *f {
        K => Rc::new(K1(x)),
        K1(ref y) => Rc::clone(y),
        S => Rc::new(S1(x)),
        S1(ref y) => Rc::new(S2(Rc::clone(y), x)),

        S2(ref y, ref z) => {
            eval(Rc::new(Apply(
                Rc::new(Apply(Rc::clone(y), Rc::clone(&x))),
                Rc::new(Apply(Rc::clone(z), Rc::clone(&x))))), ctx, cont, abort);
            return;
        }

        Print(c) => {
            ctx.output.write_fmt(format_args!("{}", c)).unwrap();
            x
        }
        I => x,
        V => f,
        E => {
            abort(x);
            return;
        }
        Read => {
            let c = ctx.input.next();
            ctx.cur_char = c;
            let t = match c {
                Some(_) => Rc::new(I),
                None => Rc::new(V),
            };
            eval(Rc::new(Apply(x, t)), ctx, cont, abort);
            return;
        }
        CompareRead(c) => {
            let t = match ctx.cur_char {
                Some(cc) if cc == c => Rc::new(I),
                _ => Rc::new(V),
            };
            eval(Rc::new(Apply(x, t)), ctx, cont, abort);
            return;
        }
        Reprint => {
            let t = match ctx.cur_char {
                Some(c) => Rc::new(Print(c)),
                None => Rc::new(V),
            };
            eval(Rc::new(Apply(x, t)), ctx, cont, abort);
            return;
        }
        D => panic!("should be handled in eval"),

        Promise(ref f) => {
            eval(Rc::new(Apply(Rc::clone(f), x)), ctx, cont, abort);
            return;
        }

        Apply(_, _) => panic!("should be handled by eval()")
    }, ctx);
}

pub fn full_eval(term: Rc<Term>, ctx: &mut Ctx) -> EvalResult {
    let result: Rc<RefCell<Option<EvalResult>>> = Rc::new(RefCell::new(None));
    let cont = {
        let result = Rc::clone(&result);
        move |x, _ctx: &mut Ctx| {
            let mut r = result.borrow_mut();
            assert!(r.is_none());
            *r = Some(Ok(x));
        }
    };
    let abort = {
        let result = Rc::clone(&result);
        move |x| {
            let mut r = result.borrow_mut();
            assert!(r.is_none());
            *r = Some(Err(x));
        }
    };
    eval(term, ctx, Rc::new(cont), Rc::new(abort));

    let r = result.borrow();
    match *r.as_ref().unwrap() {
        Ok(ref t) => Ok(Rc::clone(t)),
        Err(ref t) => Err(Rc::clone(t)),
    }
}

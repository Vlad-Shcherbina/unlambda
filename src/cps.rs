use Ctx;
use EvalResult;
use Term;
use Term::*;
use std::rc::Rc;
use std::boxed::FnBox;

pub enum ContResult {
    NextStep(Box<FnBox(&mut Ctx) -> ContResult>),
    Finished(EvalResult),
}

/*
Call graph:
   eval  calls  eval, cont
   cont1 calls  eval, cont
   cont2 calls  apply
   apply calls  eval, cont
   cont  is     cont0, cont1, cont2 (think dotted lines)

Recursive call eval->eval is mediated by NextStep.
*/

// mechanically derived from metacircular::eval()
fn eval(
    term: Rc<Term>, ctx: &mut Ctx,
    cont: Rc<Fn(Rc<Term>, &mut Ctx) -> ContResult>,
) -> ContResult {
    if let Apply(ref f, ref x) = *term {
        return ContResult::NextStep(Box::new({
            let x = Rc::clone(x);
            let f = Rc::clone(f);
            move |ctx: &mut Ctx| {
                eval(Rc::clone(&f), ctx, Rc::new(
                    // cont1
                    move |ef: Rc<Term>, ctx: &mut Ctx| {
                        if let D = *ef {
                            return cont(Rc::new(Promise(Rc::clone(&x))), ctx)
                        } else {
                            return eval(Rc::clone(&x), ctx, Rc::new({
                                let cont = Rc::clone(&cont);
                                // cont2
                                move |ex: Rc<Term>, ctx: &mut Ctx| {
                                    return apply(Rc::clone(&ef), ex, ctx, Rc::clone(&cont));
                                }
                            }));
                        }
                    }
                ))
            }
        }));
    } else {
        return cont(term, ctx);
    }
}

// mechanically derived from metacircular::apply()
fn apply(
    f: Rc<Term>, x: Rc<Term>, ctx: &mut Ctx,
    cont: Rc<Fn(Rc<Term>, &mut Ctx) -> ContResult>,
) -> ContResult {
    if let Apply(_, _) = *f {
        panic!();
    }
    if let Apply(_, _) = *x {
        panic!();
    }

    return cont(match *f {
        K => Rc::new(K1(x)),
        K1(ref y) => Rc::clone(y),
        S => Rc::new(S1(x)),
        S1(ref y) => Rc::new(S2(Rc::clone(y), x)),

        S2(ref y, ref z) => {
            return eval(Rc::new(Apply(
                Rc::new(Apply(Rc::clone(y), Rc::clone(&x))),
                Rc::new(Apply(Rc::clone(z), Rc::clone(&x))))), ctx, cont);
        }

        Print(c) => {
            ctx.output.write_fmt(format_args!("{}", c)).unwrap();
            x
        }
        I => x,
        V => f,
        E => {
            return ContResult::Finished(Err(x));
        }
        Read => {
            let c = ctx.input.next();
            ctx.cur_char = c;
            let t = match c {
                Some(_) => Rc::new(I),
                None => Rc::new(V),
            };
            return eval(Rc::new(Apply(x, t)), ctx, cont);
        }
        CompareRead(c) => {
            let t = match ctx.cur_char {
                Some(cc) if cc == c => Rc::new(I),
                _ => Rc::new(V),
            };
            return eval(Rc::new(Apply(x, t)), ctx, cont);
        }
        Reprint => {
            let t = match ctx.cur_char {
                Some(c) => Rc::new(Print(c)),
                None => Rc::new(V),
            };
            return eval(Rc::new(Apply(x, t)), ctx, cont);
        }
        D => panic!("should be handled in eval"),

        Promise(ref f) => {
            return eval(Rc::new(Apply(Rc::clone(f), x)), ctx, cont);
        }

        C => {
            return eval(Rc::new(Apply(x, Rc::new(Cont(Rc::clone(&cont))))), ctx, cont);
        }
        Cont(ref cont) => {
            return cont(x, ctx);
        }

        Apply(_, _) => panic!("should be handled by eval()")
    }, ctx);
}

pub fn full_eval(term: Rc<Term>, ctx: &mut Ctx) -> EvalResult {
    // cont0
    let cont = |x, _ctx: &mut Ctx| {
        ContResult::Finished(Ok(x))
    };

    let mut r = ContResult::NextStep(Box::new(move |ctx: &mut Ctx| {
        eval(term, ctx, Rc::new(cont))
    }));
    loop {
        match r {
            ContResult::NextStep(step) =>
                // https://github.com/rust-lang/rust/issues/25647
                r = step.call_box((ctx,)),
            ContResult::Finished(result) => return result,
        }
    }
}

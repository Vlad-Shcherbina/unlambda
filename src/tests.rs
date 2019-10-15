use super::*;
use crate::parser::parse_str;

fn run_and_expect(
        eval: &dyn Fn(Rc<Term>, &mut Ctx) -> EvalResult,
        program: &str,
        result: Option<&str>,
        output: Option<&str>) {
    run_with_input_and_expect(eval, program, "", result, output, None);
}

fn run_with_input_and_expect(
        eval: &dyn Fn(Rc<Term>, &mut Ctx) -> EvalResult,
        program: &str, input: &str,
        result: Option<&str>, output: Option<&str>, remaining_input: Option<&str>) {
    let mut buf = Vec::<u8>::new();
    let mut input_it = input.chars();
    let actual_result = {
        let mut ctx = Ctx::new(&mut buf, &mut input_it);
        eval(parse_str(program).unwrap(), &mut ctx)
            .unwrap_or_else(|e| e)
            .to_string()
    };
    if let Some(result) = result {
        assert_eq!(&actual_result, result);
    }
    if let Some(output) = output {
        assert_eq!(std::str::from_utf8(&buf).unwrap(), output);
    }
    if let Some(remaining_input) = remaining_input {
        let actual_rimaining_input: String = input_it.collect();
        assert_eq!(actual_rimaining_input, remaining_input);
    }
}

#[test]
fn test_eval() {
    let evals = [metacircular::eval, cps::full_eval, small_step::full_eval];
    for eval in &evals {
        run_and_expect(eval, "s", Some("s"), None);
        run_and_expect(eval, "s", Some("s"), None);

        run_and_expect(eval, "`.a``ks.b", Some("s"), Some("a"));

        run_and_expect(eval, "``ksv", Some("s"), None);
        run_and_expect(eval, "```skss", Some("s"), None);

        run_and_expect(eval, "`ir", Some("r"), Some(""));
        run_and_expect(eval, "`ri", Some("i"), Some("\n"));

        run_and_expect(eval, "`vs", Some("v"), None);

        run_and_expect(
            eval,
            "``````````````.H.e.l.l.o.,. .w.o.r.l.d.!rv",
            None, Some("Hello, world!\n"));

        // From the documentation on d
        run_and_expect(eval, "`d`ri", None, Some(""));
        run_and_expect(eval, "``d`rii", None, Some("\n"));
        run_and_expect(eval, "``dd`ri", None, Some("\n"));
        run_and_expect(eval, "``id`ri", None, Some(""));
        run_and_expect(eval, "```s`kdri", None, Some(""));

        run_and_expect(eval, "``ii`.av", Some("v"), Some("a"));
        run_and_expect(eval, "``ei`.av", Some("i"), Some(""));
    }
}

#[test]
fn test_input() {
    let evals = [metacircular::eval, cps::full_eval, small_step::full_eval];
    for eval in &evals {
        run_with_input_and_expect(eval, "@", "zzz", None, None, Some("zzz"));

        run_with_input_and_expect(eval, "`@i", "", Some("v"), None, Some(""));
        run_with_input_and_expect(eval, "`@i", "a", Some("i"), None, Some(""));
        run_with_input_and_expect(eval, "``@i`?ai", "a", Some("i"), None, Some(""));
        run_with_input_and_expect(eval, "``@i`?bi", "a", Some("v"), None, Some(""));
        run_with_input_and_expect(eval, "`?ai", "a", Some("v"), None, Some("a"));

        run_with_input_and_expect(eval, "```@i`|ik", "ab", Some("k"), Some("a"), Some("b"));
    }
}

#[test]
fn call_cc() {
    let evals = [cps::full_eval, small_step::full_eval];
    for eval in &evals {
        // from http://www.madore.org/~david/programs/unlambda/#callcc
        run_and_expect(eval, "``cir", Some("r"), Some("\n"));
        run_and_expect(eval, "`c``s`kr``si`ki", Some("i"), Some(""));
    }
}

#[test]
fn ramanujan() {
    let evals = [metacircular::eval, cps::full_eval, small_step::full_eval];
    for eval in &evals {
        // http://www.madore.org/~david/programs/unlambda/#howto_num
        let mut expected = "*".repeat(1729);
        expected.push('\n');
        run_and_expect(
            eval, "
            ```s`kr``s``si`k.*`ki
                ```s``s`k``si`k`s``s`ksk``s``s`ksk``s``s`kski
                ``s`k``s``s`ksk``s``s`kski`s``s`ksk
                ```s``s`kski``s``s`ksk``s``s`kski
            ",
            None, Some(&expected));
    }
}

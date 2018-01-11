use super::*;
use parser::parse_str;

fn run_and_expect(program: &str, result: Option<&str>, output: Option<&str>) {
    run_with_input_and_expect(program, "", result, output, None);
}

fn run_with_input_and_expect(
        program: &str, input: &str,
        result: Option<&str>, output: Option<&str>, remaining_input: Option<&str>) {
    let mut buf = Vec::<u8>::new();
    let mut input_it = input.chars();
    let actual_result = {
        let mut ctx = Ctx::new(&mut buf, &mut input_it);
        cps::full_eval(parse_str(program).unwrap(), &mut ctx)
            .unwrap_or_else(|e| e)
            .to_string()
    };
    if let Some(result) = result {
        assert_eq!(&actual_result.to_string(), result);
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
    run_and_expect("`.a``ks.b", Some("s"), Some("a"));

    run_and_expect("``ksv", Some("s"), None);
    run_and_expect("```skss", Some("s"), None);

    run_and_expect("`ir", Some("r"), Some(""));
    run_and_expect("`ri", Some("i"), Some("\n"));

    run_and_expect("`vs", Some("v"), None);

    run_and_expect(
        "``````````````.H.e.l.l.o.,. .w.o.r.l.d.!rv",
        None, Some("Hello, world!\n"));

    // From the documentation on d
    run_and_expect("`d`ri", None, Some(""));
    run_and_expect("``d`rii", None, Some("\n"));
    run_and_expect("``dd`ri", None, Some("\n"));
    run_and_expect("``id`ri", None, Some(""));
    run_and_expect("```s`kdri", None, Some(""));

    run_and_expect("``ii`.av", Some("v"), Some("a"));
    run_and_expect("``ei`.av", Some("i"), Some(""));
}

#[test]
fn test_input() {
    run_with_input_and_expect("@", "zzz", None, None, Some("zzz"));

    run_with_input_and_expect("`@i", "", Some("v"), None, Some(""));
    run_with_input_and_expect("`@i", "a", Some("i"), None, Some(""));
    run_with_input_and_expect("``@i`?ai", "a", Some("i"), None, Some(""));
    run_with_input_and_expect("``@i`?bi", "a", Some("v"), None, Some(""));
    run_with_input_and_expect("`?ai", "a", Some("v"), None, Some("a"));

    run_with_input_and_expect("```@i`|ik", "ab", Some("k"), Some("a"), Some("b"));
}

#[test]
fn call_cc() {
    // from http://www.madore.org/~david/programs/unlambda/#callcc
    run_and_expect("``cir", Some("r"), Some("\n"));
    run_and_expect("`c``s`kr``si`ki", Some("i"), Some(""));
}

#[test]
fn ramanujan() {
    // https://stackoverflow.com/a/29980945/6335232
    let child = std::thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(|| {
        // From the documentation
        let mut expected = "*".repeat(1729);
        expected.push('\n');
        run_and_expect("
        ```s`kr``s``si`k.*`ki
            ```s``s`k``si`k`s``s`ksk``s``s`ksk``s``s`kski
            ``s`k``s``s`ksk``s``s`kski`s``s`ksk
            ```s``s`kski``s``s`ksk``s``s`kski
        ", None, Some(&expected));
    }).unwrap();
    child.join().unwrap();
}

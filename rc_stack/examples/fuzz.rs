use std::sync::Mutex;
use std::ops::Deref;
use rand::Rng;
use rc_stack::RcStackSimple;
use rc_stack::RcStack;

struct Var {
    name: char,
    v: Vec<i32>,
    r: RcStackSimple<i32>,
    s: RcStack<i32>,
}

fn run(max_iterations: usize, log: &Mutex<Vec<String>>) {
    let mut rng = rand::thread_rng();

    let lg = |s: String| {
        // println!("{}", s);
        log.lock().unwrap().push(s);
    };

    let mut next_name = b'a';
    let mut new_name = || {
        next_name += 1;
        (next_name - 1) as char
    };

    let mut vars = Vec::new();
    while log.lock().unwrap().len() < max_iterations {
        let idx = rng.gen_range(0, vars.len() + 1);
        if idx == vars.len() {
            let name = new_name();
            lg(format!("let mut {} = RcStack::new();", name));
            vars.push(Var {
                name,
                v: Vec::new(),
                r: RcStackSimple::new(),
                s: RcStack::new(),
            });
            continue;
        }
        match rng.gen_range(0, 9) {
            0 => {
                let var = &mut vars[idx];
                let elem = rng.gen_range(0, 100);
                lg(format!("{}.push({});", var.name, elem));
                var.v.push(elem);
                var.r.push(elem);
                var.s.push(elem);
            }
            1 => {
                let var = &vars[idx];
                let name = new_name();
                lg(format!("let mut {} = {}.clone();", name, var.name));
                vars.push(Var {
                    name,
                    v: var.v.clone(),
                    r: var.r.clone(),
                    s: var.s.clone(),
                });
            }
            2 => {
                lg(format!("drop({});", vars[idx].name));
                drop(vars.remove(idx));
            }
            3 => {
                let var = &vars[idx];
                lg(format!("{}.peek();", var.name));
                let e1 = var.v.last().cloned();
                let e2 = var.r.peek().cloned();
                let e3 = var.s.peek().map(|e| e.clone());
                assert_eq!(e1, e2);
                assert_eq!(e1, e3);
            }
            4 => {
                let var = &mut vars[idx];
                lg(format!("{}.try_pop_unwrap();", var.name));
                let e2 = var.r.try_pop_unwrap();
                let e3 = var.s.try_pop_unwrap();
                assert_eq!(e2, e3);
                if e2.is_some() {
                    assert_eq!(e2, var.v.pop());
                }
            }
            5 => {
                let var = &mut vars[idx];
                lg(format!("{}.pop_clone();", var.name));
                let e1 = var.v.pop();
                let e2 = var.r.pop_clone();
                let e3 = var.s.pop_clone();
                assert_eq!(e1, e2);
                assert_eq!(e1, e3);
            }
            6 => {
                let var = &mut vars[idx];
                lg(format!("{}.discard_top();", var.name));
                let e1 = var.v.pop().is_some();
                let e2 = var.r.discard_top();
                let e3 = var.s.discard_top();
                assert_eq!(e1, e2);
                assert_eq!(e1, e3);
            }
            7 => {
                let var = &vars[idx];
                lg(format!("{}.is_empty();", var.name));
                let e1 = var.v.is_empty();
                let e2 = var.r.is_empty();
                let e3 = var.s.is_empty();
                assert_eq!(e1, e2);
                assert_eq!(e1, e3);
            }
            8 => {
                let var = &vars[idx];
                lg(format!("{}.check();", var.name));
                var.s.check();
            }
            _ => panic!()
        }
    }
    lg(String::from("done"));
}

fn main() {
    let log = Mutex::new(Vec::new());

    let mut num_tests = 0;

    let mut max_iterations = 26;
    loop {
        num_tests += 1;
        if num_tests % 100_000 == 0 {
            println!("{} tests run", num_tests);
        }
        log.lock().unwrap().clear();
        let r = std::panic::catch_unwind(|| {
            run(max_iterations, &log);
        });
        if r.is_ok() {
            continue;
        }

        for line in log.lock().unwrap().deref() {
            println!("{}", line);
        }
        println!();
        let len = log.lock().unwrap().len();
        assert!(len > 0);
        max_iterations = len - 1;
    }
}

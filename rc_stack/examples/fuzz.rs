#![feature(nll)]

extern crate rc_stack;
extern crate rand;

use std::sync::Mutex;
use std::ops::Deref;
use rand::Rng;
use rc_stack::RcStackSimple;

struct Var {
    name: char,
    s: RcStackSimple<i32>,
    v: Vec<i32>,
}

fn run(max_iterations: usize, log: &Mutex<Vec<String>>) {
    let mut rng = rand::thread_rng();

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
            log.lock().unwrap().push(format!("let {} = RcStack::new();", name));
            vars.push(Var {
                name,
                s: RcStackSimple::new(),
                v: Vec::new(),
            });
            continue;
        }
        match rng.gen_range(0, 8) {
            0 => {
                let var = &mut vars[idx];
                let elem = rng.gen_range(0, 100);
                log.lock().unwrap().push(format!("{}.push({});", var.name, elem));
                var.s.push(elem);
                var.v.push(elem);
            }
            1 => {
                let var = &vars[idx];
                let name = new_name();
                log.lock().unwrap().push(format!("let {} = {}.clone();", name, var.name));
                vars.push(Var {
                    name,
                    s: var.s.clone(),
                    v: var.v.clone(),
                });
            }
            2 => {
                log.lock().unwrap().push(format!("drop({});", vars[idx].name));
                vars.remove(idx);
            }
            3 => {
                let var = &vars[idx];
                log.lock().unwrap().push(format!("{}.peek();", var.name));
                let e1 = var.s.peek().cloned();
                let e2 = var.v.last().cloned();
                assert_eq!(e1, e2);
            }
            4 => {
                let var = &mut vars[idx];
                log.lock().unwrap().push(format!("{}.try_pop_unwrap();", var.name));
                let e1 = var.s.try_pop_unwrap();
                if e1.is_some() {
                    assert_eq!(e1, var.v.pop());
                }
            }
            5 => {
                let var = &mut vars[idx];
                log.lock().unwrap().push(format!("{}.pop_clone();", var.name));
                let e1 = var.s.pop_clone();
                let e2 = var.v.pop();
                assert_eq!(e1, e2);
            }
            6 => {
                let var = &mut vars[idx];
                log.lock().unwrap().push(format!("{}.discard_top();", var.name));
                var.s.discard_top();
                var.v.pop();
            }
            7 => {
                let var = &vars[idx];
                log.lock().unwrap().push(format!("{}.is_empty();", var.name));
                let e1 = var.s.is_empty();
                let e2 = var.v.is_empty();
                assert_eq!(e1, e2);
            }
            _ => panic!()
        }
    }
    log.lock().unwrap().push(String::from("done"));
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

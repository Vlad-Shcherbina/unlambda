### Unlambda interpreters

Interpreters for the [Unlambda](http://www.madore.org/~david/programs/unlambda/) programming language.

 * `metacircular.rs` simple recursive interpreter, no `call/cc` support
 * `cps.rs` continuation-passing style interpreter with closures
 * `small_step.rs` completely explicit non-recursive interpreter, quite fast

### CLI

```
cargo run -- --help
```

### How to test

```
cargo check --all --examples --tests
cargo test --all
cargo run -p rc_stack --example fuzz  # run for a while
```

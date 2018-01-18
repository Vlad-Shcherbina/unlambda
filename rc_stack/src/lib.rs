#![feature(nll)]

mod rc_stack_simple;
mod rc_stack;

pub use rc_stack_simple::RcStack as RcStackSimple;
pub use rc_stack::RcStack;
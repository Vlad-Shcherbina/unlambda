use std::fmt::Debug;
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::mem;

// TODO: remove Debug trait bound everywhere
#[derive(Debug)]
pub struct RcStack<T>(Link<T>);

type Link<T> = Option<(Rc<Block<T>>, usize)>;

#[derive(Debug)]
struct Block<T> {
    items: RefCell<Vec<(T, usize)>>,  // never empty
    tail: RcStack<T>,
}

impl<T> Drop for RcStack<T> {
    fn drop(&mut self) {
        if let Some((block, idx)) = self.0.take() {
            let mut items = block.items.borrow_mut();
            items[idx].1 -= 1;
            while !items.is_empty() && items[items.len() - 1].1 == 0 {
                items.pop();
            }
        }
        // TODO: avoid recursion
    }
}

impl<T> RcStack<T> {
    pub fn new() -> Self {
        RcStack(None)
    }

    pub fn peek(&self) -> Option<Ref<T>> {
        match self.0 {
            Some((ref block, idx)) => {
                let items = block.items.borrow();
                Some(Ref::map(items, |items| &items[idx].0))
            }
            None => None,
        }
    }

    pub fn push(&mut self, elem: T) {
        if let Some((ref mut block, ref mut idx)) = self.0 {
            let mut items = block.items.borrow_mut();
            if *idx + 1 == items.len() {
                items[*idx].1 -= 1;
                *idx += 1;
                items.push((elem, 1));
                return;
            }
        }

        let tail = mem::replace(self, RcStack::new());
        let block = Block {
            items: RefCell::new(vec![(elem, 1)]),
            tail: tail,
        };
        self.0 = Some((Rc::new(block), 0));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    fn pop_and_apply<UF, SF, EF, R>(&mut self, pop_shared: bool, unique_fn: UF, shared_fn: SF, empty_fn: EF) -> R
    where
        UF: FnOnce(T) -> R,
        SF: FnOnce(&T) -> R,
        EF: FnOnce() -> R,
    {
        match self.0 {
            Some((ref mut block, ref mut idx)) => {
                let mut items = block.items.borrow_mut();
                items[*idx].1 -= 1;
                let result = if *idx + 1 == items.len() && items[*idx].1 == 0 {
                    unique_fn(items.pop().unwrap().0)
                    // TODO: downsize when too much capacity is wasted
                } else {
                    if !pop_shared {
                        items[*idx].1 += 1;
                        return empty_fn();
                    }
                    shared_fn(&items[*idx].0)
                };
                if *idx > 0 {
                    *idx -= 1;
                    items[*idx].1 += 1;
                } else {
                    drop(items);
                    match block.tail.0 {
                        Some((ref block2, idx2)) => {
                            block2.items.borrow_mut()[idx2].1 += 1;
                            *block = Rc::clone(block2);
                            *idx = idx2;
                            assert!(!block.items.borrow().is_empty());
                        }
                        None => self.0 = None,
                    }
                }
                result
            }
            None => empty_fn()
        }
    }

    pub fn discard_top(&mut self) -> bool {
        self.pop_and_apply(true, |_| true, |_| true, || false)
    }

    /// If the top element of the stack is not shared, pops and returns it.
    /// Otherwise, returns None and leaves the stack unchanged.
    ///
    /// Somewhat similar to Rc::try_unwrap().
    pub fn try_pop_unwrap(&mut self) -> Option<T> {
        self.pop_and_apply(false, |elem| Some(elem), |&_| None, || None)
    }
}

impl<T: Clone> RcStack<T> {
    pub fn pop_clone(&mut self) -> Option<T> {
        let result = self.pop_and_apply(true, |elem| Some(elem), |r| Some(r.clone()), || None);
        if let Some(ref q) = self.0 {
            assert!(!q.0.items.borrow().is_empty());
        }
        result
    }
}

impl<T: Debug> Clone for RcStack<T> {
    fn clone(&self) -> RcStack<T> {
        if let Some((ref block, idx)) = self.0 {
            block.items.borrow_mut()[idx].1 += 1;
        }
        RcStack(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regr1() {
        let mut a = RcStack::new();
        a.push(49);
        let mut b = a.clone();
        a.push(27);
        b.discard_top();
        drop(a);
    }

    #[test]
    fn stuff() {
        let mut a = RcStack::new();
        a.push(10);

        let mut b = RcStack::clone(&a);
        b.push(20);

        assert_eq!(a.pop_clone(), Some(10));
        assert_eq!(a.pop_clone(), None);
        assert_eq!(a.pop_clone(), None);

        assert_eq!(b.pop_clone(), Some(20));
        assert_eq!(b.pop_clone(), Some(10));
        assert_eq!(b.pop_clone(), None);
        assert_eq!(b.pop_clone(), None);
    }

    #[test]
    #[ignore]  // TODO
    fn drop_is_non_recursive() {
        let mut s = RcStack::new();
        for i in 0..1_000_000 {
            let mut t = s.clone();
            t.push(42);
            s.push(i);
        }
    }
}

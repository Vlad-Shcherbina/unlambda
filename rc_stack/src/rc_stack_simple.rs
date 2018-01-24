use std::rc::Rc;
use std::mem;

#[derive(Default)]
pub struct RcStack<T>(Option<Rc<(T, RcStack<T>)>>);

impl<T> Drop for RcStack<T> {
    fn drop(&mut self) {
        while self.try_pop_unwrap().is_some() {}
    }
}

impl<T> RcStack<T> {
    pub fn new() -> Self {
        RcStack(None)
    }

    pub fn peek(&self) -> Option<&T> {
        match self.0 {
            Some(ref p) => Some(&p.0),
            None => None,
        }
    }

    pub fn push(&mut self, elem: T) {
        let old = mem::replace(self, RcStack::new());
        self.0 = Some(Rc::new((elem, old)));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn discard_top(&mut self) -> bool {
        match self.0 {
            Some(ref p) => {
                *self = RcStack::clone(&p.1);
                true
            }
            None => false
        }
    }

    /// If the top element of the stack is not shared, pops and returns it.
    /// Otherwise, returns None and leaves the stack unchanged.
    ///
    /// Somewhat similar to Rc::try_unwrap().
    pub fn try_pop_unwrap(&mut self) -> Option<T> {
        let r = self.0.take()?;
        match Rc::try_unwrap(r) {
            Ok((elem, tail)) => {
                *self = tail;
                Some(elem)
            }
            Err(r) => {
                self.0 = Some(r);
                None
            }
        }
    }
}

impl<T: Clone> RcStack<T> {
    pub fn pop_clone(&mut self) -> Option<T> {
        match self.try_pop_unwrap() {
            Some(elem) => Some(elem),
            None => {
                let result = self.peek().cloned();
                self.discard_top();
                result
            }
        }
    }
}

impl<T> Clone for RcStack<T> {
    fn clone(&self) -> RcStack<T> {
        RcStack(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stuff() {
        let mut a = RcStack::new();
        a.push(1);

        let mut b = RcStack::clone(&a);
        b.push(2);

        assert_eq!(b.pop_clone(), Some(2));
        assert_eq!(b.pop_clone(), Some(1));
        assert_eq!(b.pop_clone(), None);

        assert_eq!(a.pop_clone(), Some(1));
        assert_eq!(a.pop_clone(), None);
    }

    #[test]
    fn drop_is_non_recursive() {
        let mut s = RcStack::new();
        for i in 0..1_000_000 {
            s.push(i);
        }
    }
}

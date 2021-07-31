mod macros;
mod reactivity;

pub use reactivity::{batch, Effect, Memo, Signal};

#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
use std::rc::Rc;

#[test]
fn test_effect() {
    let out = Rc::new(RefCell::new(Vec::new()));
    let a = Signal::new(1);
    let b = Signal::new(2);
    let c = Memo::new(cloned!((a, b) => move || *a.read() + *b.read()));
    let _effect = Effect::new(cloned!((out) => move || out.borrow_mut().push(*c.read())));
    *a.write() = 3;
    batch(|| {
        *a.write() = 10;
        *b.write() = 12;
    });
    assert_eq!(*out.borrow(), vec![3, 5, 22]);
}

#[test]
fn test_calmed() {
    let out = Rc::new(RefCell::new(Vec::new()));
    let a: Signal<i32> = Signal::new(1);
    let b = Memo::new_calmed_eq(cloned!((a) => move || *a.read()));
    let _effect = Effect::new(cloned!((out) =>move || out.borrow_mut().push(*b.read())));
    *a.write() = 2;
    *a.write() = 2;
    *a.write() = 3;
    assert_eq!(*out.borrow(), vec![1, 2, 3]);
}

#[test]
fn test_memo_mem() {
    let out = Rc::new(RefCell::new(Vec::new()));
    let a = Signal::new(1);
    {
        let _e = Effect::new(cloned!((a, out) => move || out.borrow_mut().push(*a.read())));
        *a.write() = 2;
    }
    *a.write() = 3;
    // here should only output 1 2
    assert_eq!(*out.borrow(), vec![1, 2]);
}

#[test]
fn test_write_sig_in_effect() {
    let out = Rc::new(RefCell::new(Vec::new()));
    let a = Signal::new(1);
    let b = Signal::new(0);
    let _e1 = Effect::new(cloned!((a,b) => move || *b.write() = *a.read() * 2));
    let _e2 = Effect::new(cloned!((out,b) => move || out.borrow_mut().push(*b.read())));
    batch(move || {
        *a.write() = 2;
        *b.write() = 3;
    });
    assert_eq!(*out.borrow(), vec![2, 3, 4]);
}


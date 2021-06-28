mod macros;
mod reactivity;

pub use reactivity::{batch, Effect, Memo, Signal};

#[test]
fn test_effect() {
    let mut a = Signal::new(1);
    let mut b = Signal::new(2);
    let c = Memo::new(cloned!((a, b) => move || *a.read() + *b.read()));
    let _effect = Effect::new(move || println!("{}", *c.read()));
    *a.write() = 3;
    batch(|| {
        *a.write() = 10;
        *b.write() = 12;
    });
}

#[test]
fn test_calmed() {
    let mut a: Signal<i32> = Signal::new(1);
    let b = Memo::new_calmed_eq(cloned!((a) => move || *a.read()));
    let _effect = Effect::new(move || println!("{}", *b.read()));
    *a.write() = 2;
    *a.write() = 2;
    *a.write() = 3;
}

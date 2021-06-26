mod context;
mod effect;
mod memo;
mod node;
mod signal;

pub use self::effect::Effect;
pub use self::signal::Signal;
pub use self::memo::Memo;
pub use self::context::batch;

#[test]
fn test_effect() {
    let mut a = Signal::new(1);
    let mut b = Signal::new(2);
    let c;
    {
        let a = a.clone();
        let b = b.clone();
        c = Memo::new(move || *a.read() + *b.read());
    }
    let _effect = Effect::new(move || println!("{}", *c.read()));
    *a.write() = 3;
    batch(|| {
        *a.write() = 10;
        *b.write() = 12;
    });
}

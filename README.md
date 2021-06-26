# Fine Grain Reactivity for Rust

Based on:
https://indepth.dev/posts/1269/finding-fine-grained-reactive-programming

Example:
```
fn main() {
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
```
Will Output:
```
3
5
22
```

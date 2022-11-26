use utemplate::fmt;

fn main() {
    let mut dst = String::new();
    let x = 5usize;
    let ys = vec![4u8, 3, 5];
    fmt!(dst += 
r#"Hello world {x}
[for y in &ys {-]
[]  y: {y}
[}-]
"#);
    println!("{}", dst);
    dst.clear();
    let dstp = &mut dst;
    fmt!(*dstp += "{x}*[[[for y in &ys {]{y}, [}]]");
    println!("{}", dst);
    println!("{}", fmt!("{x}*[[[for y in &ys {]{y}, [}]]"));
}
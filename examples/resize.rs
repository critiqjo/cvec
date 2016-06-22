extern crate cvec;

use cvec::CVec;

fn main() {
    let mut x: CVec<usize> = CVec::with_capacity(2);
    x.push(300).unwrap();
    x.push(200).unwrap();
    println!("{:?}", x.push(100));
}

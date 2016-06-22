extern crate cvec;

use cvec::CVec;

fn main() {
    let mut x: CVec<usize> = CVec::with_capacity(4);
    x.push(300).unwrap();
    x.push(200).unwrap();
    x.shrink_to_fit();
    println!("{:?}", x.push(100));
}

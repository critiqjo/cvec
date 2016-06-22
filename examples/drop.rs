extern crate cvec;

use cvec::CVec;

#[derive(Debug)]
struct Inspect(usize);

impl Drop for Inspect {
    fn drop(&mut self) {
        println!("Dropped {}", self.0);
    }
}

fn main() {
    let mut x: CVec<Inspect> = CVec::with_capacity(4);
    x.push(Inspect(300)).unwrap();
    x.push(Inspect(200)).unwrap();
    x.push(Inspect(100)).unwrap();
}

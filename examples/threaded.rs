extern crate cvec;

use cvec::CVec;
use std::thread;

fn main() {
    let mut v = CVec::with_capacity(1000);
    let vv = v.view();
    let h1 = thread::spawn(move || {
        for i in 0..1000 {
            v.push(i).unwrap();
            thread::yield_now();
        }
    });
    let h2 = thread::spawn(move || {
        let mut old_len = 0;
        while old_len < 1000 {
            let s = vv.as_slice();
            if s.len() > old_len {
                println!("{:?}", &s[old_len..]);
                old_len = s.len();
            }
        }
    });
    h1.join().unwrap();
    h2.join().unwrap();
}

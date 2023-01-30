#![feature(test)]
#![feature(pattern)]
extern crate test;
use strmatch::strmatch as s;

fn main() {
    let str = "one two two three three three";
    match str.as_bytes() {
        s!("one ":"two "x2:"three "x2:rest) => {
            let str = String::from_utf8(rest.to_vec()).unwrap();
            assert_eq!(&str, "three")
        }
        s!() => println!(),
        [_a @ _] => println!("yay"),
        _ => println!("hmm"),
    }
}

#[cfg(test)]
mod tests {
    use strmatch::strmatch;

    #[test]
    fn empty() {
        assert!(matches!("".as_bytes(), strmatch!()))
    }

    #[test]
    fn byte_or_string_literal() {
        assert!(matches!("hello".as_bytes(), strmatch!("hello")));
        assert!(matches!("hello".as_bytes(), strmatch!(b"hello")));
    }
}

#[cfg(test)]
mod bench {
    use std::str::pattern::Pattern;
    use strmatch::strmatch;
    use test::{black_box, Bencher};

    #[bench]
    fn prefix_of(b: &mut Bencher) {
        b.iter(|| black_box("hello").is_prefix_of(black_box("helloword")))
    }

    #[bench]
    fn strmatch(b: &mut Bencher) {
        b.iter(|| {
            black_box(matches!(
                black_box("helloworld").as_bytes(),
                strmatch!("hello": _rest)
            ))
        })
    }
}

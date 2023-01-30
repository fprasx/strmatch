use strmatch::strmatch;

fn main() {
    let str = "one two two three three three";
    match str.as_bytes() {
        strmatch!(b"one ":b"two "x2:b"three "x2:rest) => {
            let str = String::from_utf8(rest.to_vec()).unwrap();
            assert_eq!(&str, "three")
        }
        _ => println!("hmm"),
    }
}

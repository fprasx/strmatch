use strmatch::strmatch;

fn main() {
    // Convert to bytes so we can use slice pattern matching
    let test = "hello hello strmatch!".as_bytes();
    match test {
        strmatch!("hello hello "[rest]) => assert_eq!(rest, b"strmatch!"),
        strmatch!("hello "x2 "strmatch" rest) => assert_eq!(*rest, b'!'),
        strmatch!("hello strmatch" rest) => assert_eq!(*rest, b'!'),
        strmatch!() => {}
        _ => {}
    }

    // Convert to bytes so we can use slice pattern matching
    let str = "one twotwo threethreethree";
    match str.as_bytes() {
        // We can start off by matching an empty string
        strmatch!() => {}

        // Ignore one character ...
        strmatch!(_) => {}

        // Or take it!
        strmatch!(mine_now) => {}

        // Match a literal ...
        strmatch!('x') => {}
        strmatch!("xyz") => {}

        // And match repeats!
        strmatch!("one" _ "two"x2  _ "three"x3) => {}

        // Bracketed patterns can be the last term of a pattern.
        // Ignore everything past "one"
        strmatch!("one" [_]) => {}

        // Or give it a name :)
        strmatch!("one" _ [hellooo]) => {
            assert_eq!(hellooo, b"twotwo threethreethree");
        }

        // We can combine patterns however we want!
        strmatch!("one" _ "two"x2 space "three"x2 [rest]) => {
            assert_eq!(space, &b' ');
            assert_eq!(rest, b"three");
        }

        _ => println!("Macros are fun :p"),
    }
}

#[cfg(test)]
mod tests {
    use strmatch::strmatch;

    #[test]
    fn syntax() {
        match "".as_bytes() {
            strmatch!() | strmatch!() => {}
            _ => {}
        }
    }

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

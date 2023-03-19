# strmatch!

Ever felt like regex is a little overkill?
`strmatch!` is a macro that makes validating and extracting parts of
strings easier.  It works by converting your query into a slice pattern,
hopefully allowing to optimizer to take a stab at your queries! Since it uses
raw slice patterns, it should be very fast.

# Usage

```rust
// Convert to bytes so we can use slice pattern matching.
let str = "one twotwo threethreethree";

// This `as_bytes` call is completely free!
// It's just a transmute under the hood.
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
    strmatch!("one" ' ' "two"x2 space "three"x2 [rest]) => {
        assert_eq!(space, &b' ');
        assert_eq!(rest, b"three");
    }

    _ => println!("Macros are fun :p"),
}
```

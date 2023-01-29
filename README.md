# Passable 🛳

Pass a pointer around. Kinda like `Rc`, but there can only be one accessor at a time, making mutable mutation possible. When a `Pass` holding the reference is dropped, it gives the reference back to its predecessor.

```rust
use passable::Pass;

let mut one = Pass::new("hello");

{
    let mut two = one.pass().unwrap();

    // now two has the reference, and not one.
    assert_eq!(two.deref(), Some(&"hello"));
    assert_eq!(one.deref(), None);

    *two.deref_mut().unwrap() = "goodbye";
}

// two is dropped here, giving the reference back to one.

assert_eq!(one.deref(), Some(&"goodbye"));
```

You can also drop a reference in the middle of the chain.

```rust
use passable::Pass;

let mut one = Pass::new(true);

let mut two = one.pass().unwrap();
*two.deref_mut().unwrap() = false;

let mut three = two.pass().unwrap();

std::mem::drop(two);

assert_eq!(three.deref(), Some(&false));

*three.deref_mut().unwrap() = true;
std::mem::drop(three);

assert_eq!(one.deref(), Some(&true));
```

## Notes 
`Pass` implements `Default`, but it does not implement any of the other std library traits that rely on having a reference to the internal value, like `Clone`, `Debug`, and `Display`. If you have a suggestion for how these should be implemented when the `Pass` does not have a reference to the internal value, please submit an issue!
`Pass` is implemented as a linked list, with each node holding an `Option<NonNull<T>>` to the internal value. Each `Pass` object on the stack has a size of 8 bytes, and each node in the list has a size of 24 bytes.

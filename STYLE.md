# Style Guide

## Type Annotations
* Prefer turbofish syntax over type ascription:
```rust
// Good
let json = serde_json::from_str::<Value>(input).unwrap();

// Avoid
let json: Value = serde_json::from_str(input).unwrap();
```

## String Formatting
* Use inlined string formatting where possible:
```rust
// Good
println!("=== Item ID: {id} ===");

// Avoid
println!("=== Item ID: {} ===", id);
```

* For complex formatting, use intermediate variables to avoid parameter-based interpolation:
```rust
// Good
let item_json_pretty = serde_json::to_string_pretty(item_json).unwrap();
println!("{item_json_pretty}");

// Avoid
println!("{}", serde_json::to_string_pretty(item_json).unwrap());
```
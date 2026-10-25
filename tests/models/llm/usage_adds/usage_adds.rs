use super::*;

/// Usage adds field by field.
#[test]
fn usage_adds() {
    let a = Usage { input_tokens: 10, cache_read_tokens: 2, cache_write_tokens: 0, output_tokens: 5, reasoning_tokens: 1 };
    let b = Usage { input_tokens: 4, cache_read_tokens: 0, cache_write_tokens: 3, output_tokens: 6, reasoning_tokens: 2 };
    let mut sum = a + b;
    assert_eq!(sum, Usage { input_tokens: 14, cache_read_tokens: 2, cache_write_tokens: 3, output_tokens: 11, reasoning_tokens: 3 });
    sum += a;
    assert_eq!(sum.input_tokens, 24);
}

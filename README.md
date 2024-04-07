# C Defines To Enum
Turn C Style `#define` statement into `enum` in Rust.

# Usage

Providing exactly one enum name. And string literal with name `content` to generate enums from C defines.

Content could be evaluated from macros like `include_str!`.

This crate is crafted to use under `no_std` environment.

```Rust
use c_defines_to_enum::parse_c_defines_to_enum;
use std::convert::TryFrom;

parse_c_defines_to_enum!(
            TestEnum,
            to_lower = true,
            content = include_str!("test.h")
        );

fn it_works() {
    println!("{:?}", TestEnum::try_from(1234));
    let value: usize = TestEnum::test1.into();
    println!("{:?} = {}", TestEnum::test1, value);
}
```
Result:
```
Ok(test1)
test1 = 1234
```

# Support attributes
* to_lower
  * bool, turn C defines' name into lowercase.
* to_upper
  * bool, turn C defines' name into uppercase.
* remove_prefix
  * str, remove prefix of C defines' name.
* remove_suffix
  * str, remove suffix of C defines' name.

# Known Issue
* Not support cascading ahead like:
```C
#define A B
#define B 1000
```
* If any duplicated values detected, enum is not stored with actual value by rust. You should use trait of `Into<usize>`
* `#[repr(usize)]` and `TryFrom<usize>`/`Into<usize>` are fixed.
* Generated enum is always `pub`
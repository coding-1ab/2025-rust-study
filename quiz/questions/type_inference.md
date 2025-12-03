# 타입 추론

아래 코드의 변수 value가 가지는 타입을 고르시오:

```rs
fn main() {
    let mut value = 10;
    while value < 100 {
        println!("{value}");
        value = value * 2;
    }
}
```

- [ ] u32
- [ ] f32
- [ ] i64
- [x] i32

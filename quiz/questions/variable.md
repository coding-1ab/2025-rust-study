# 변수 선언

아래 _로 표기한 빈칸에 들어갈 올바른 변수 선언문을 고르시오:

```rs
fn main() {
    __________
    println!("{n}");
    while n != 1 {
        if n % 2 == 0 {
            n = n / 2;
        } else {
            n = 3 * n + 1;
        }
        println!("{n}");
    }
}
```

- [ ] int n = 13;
- [ ] var n = 13;
- [ ] let n = 13;
- [x] let mut n = 13;

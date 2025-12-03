# 가변성

아래 코드를 실행한 이후 변수 a의 값을 고르시오:

```rs
fn main() {
    let mut a = 35;
    let mut b = 27;
    
    let mut alias = &mut a;
    *alias = b;
    alias = &mut b;
    *alias = a + 1;
}
```

- [ ] 35
- [ ] 36
- [x] 27
- [ ] 28

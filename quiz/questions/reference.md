# 레퍼런스

아래 코드를 실행하면 나오는 결과를 고르시오:

```rs
fn main() {
    let mut a = 10;
    let b = 20;
    
    let mut reference = &b;
    reference = &a;
    
    println!("{}", reference);
}
```

- [ ] 20
- [x] 10
- [ ] a의 주소값.
- [ ] 컴파일 오류.

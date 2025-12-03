# String vs &str

String은 동적으로 힙에 할당된 문자열입니다. String은 유저 입력을 받는 버퍼로 사용할 수 있지만
&str은 사용할 수 없는 이유를 고르시오.

```rs
fn main() {
    let mut buffer = String::new();
    let mut literal = "";
    std::io::stdin().read_line(&mut buffer).unwrap();
    std::io::stdin().read_line(&mut literal).unwrap(); // 오류
}
```

- [ ] read_line은 힙에 있는 문자열만 사용할 수 있어서.
- [x] read_line 함수는 재할당 가능한 버퍼가 필요하지만 &str은 재할당이 불가능해서.
- [ ] &str은 표준 입출력에 사용할 수 없어서.
- [ ] &str은 불변이라 다른 함수에 전달할 수 없어서.

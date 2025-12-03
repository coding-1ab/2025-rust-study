# match

아래 코드는 컴파일 되지 않습니다. 그 적절한 이유를 서술하시오:

```rs
fn main() {
    let a = 5;
    let b = 3;
    
    match a.cmp(&b) {
        Ordering::Less => {
            println!("Less"),
        }
        Ordering::Greater => println!("Greater")
    }
}
```

- [x] 비교 결과의 모든 경우의 수(Less, Equal, Greater)를 다 확인하지 않아서.
- [ ] 5와 3을 비교한 결과는 자명하기 때문에 Less를 확인하는 구문을 제거해서.
- [ ] cmp 함수는 레퍼런스가 아니라 값을 전달받아서.
- [ ] 숫자는 cmp 함수를 사용할 수 없어서.
- [ ] Greater를 확인하는 구문에 중괄호가 없어서.

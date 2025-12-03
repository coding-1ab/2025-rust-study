# char의 크기

아래 코드에서 첫번째 println!은 5를 출력하고 두번째 println!은 20을 출력하는 이유를 고르시오:

```rs
fn main() {
    let apple = "apple";
    let a = 'a';
    let p = 'p';
    let l = 'l';
    let e = 'e';

    println!("string size: {}", apple.len()); // 5
    println!(
        "char sum size: {}",
        size_of_val(&a) + size_of_val(&p) + size_of_val(&p) + size_of_val(&l) + size_of_val(&e)
    ); // 20
}
```

- [x] 문자열은 UTF-8 인코딩을 사용해 각 글자가 1~4바이트만 사용하지만, char의 크기는 언제나 4 바이트라서.
- [ ] 각 char마다 바이트 길이가 달라서.
- [ ] size_of_val(&a)가 char의 길이 대신 레퍼런스의 바이트 길이를 반환해서.
- [ ] apple 문자열은 중복된 p 문자를 하나로 압축해서.

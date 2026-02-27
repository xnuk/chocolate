# https://chocolate.xnu.kr
[백준 25800번 초콜릿 프로그래밍 언어]의 Rust 구현체이자 웹 구현체입니다.

## Build/Contributing
### 준비물
- [mise](https://mise.jdx.dev/)
- [rustup](https://rustup.rs/)
- sed
- cp

### Build
```sh
# Install wasm Rust compiler
rustup target add wasm32-unknown-unknown

# Install dependencies
mise install

# Build web
mise run build:web
```

### Contributing
```sh
# Install devDependencies - oxfmt, oxlint, typescript
mise run dev-install

mise run lint
mise run format
```

## Copyrights
- 소스 코드에 동봉된 웹 폰트는 [Inconsolata]로, [SIL Open Font License]를
  따릅니다.
- 소스 코드 중 일부는 [백준 25800번 초콜릿 프로그래밍 언어]의 언어 명세 및
  구현체를 참고 및 사용했습니다.
- 이 소스코드는 [AGPL 3.0](./LICENSE)로 배포됩니다.

[Inconsolata]: https://levien.com/type/myfonts/inconsolata.html
[SIL Open Font License]: https://openfontlicense.org/
[백준 25800번 초콜릿 프로그래밍 언어]: https://www.acmicpc.net/problem/25800

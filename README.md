## テストカバレッジの確認方法

```
cargo install cargo-llvm-cov
cargo llvm-cov
```

結果：

<img width="1626" height="188" alt="Image" src="https://github.com/user-attachments/assets/d7ac3521-a100-4838-9c3f-671e81a4b421" />

## エラーハンドリングのライブラリ

- `thiserror`：エラーの種類によって挙動を変えたい時などはこっちの方がいい

https://github.com/dtolnay/thiserror

- `anyhow`：エラーの種類が全て`anyhow::Error`に変換されるので、エラーの種類を気にせず、素早く実装したい時など

https://github.com/dtolnay/anyhow

# 桃太郎伝説 (FC) パスワードライブラリ

パスワードとゲーム状態の相互変換を行う。

## Example

```rust, no_run
//! パスワード「ふ」をロードする。

use momoden_password::*;

fn main() {
    let password = Password::parse("ふ").unwrap();
    let bytes = SerializedBytes::from_password(&password);

    if let Some(savedata) = bytes.to_savedata() {
        // 生のセーブデータ(装備インデックスが不正なまま)を出力する。
        println!("raw: {savedata:?}");
        // 実際にロードされるセーブデータ(装備が変化した後)を出力する。
        let savedata_norm = savedata.normalize();
        println!("normalized: {savedata_norm:?}");
    } else {
        // チェックサムが一致しない場合、to_savedata() は失敗する。
        println!("checksum mismatch");
    }
}
```

```rust, no_run
//! パスワード「ふ」をロードした直後の状態をパスワード化する。

use momoden_password::*;

fn main() {
    // パスワード「ふ」をロードし、実際にロードされるセーブデータを得る。
    let password = Password::parse("ふ").unwrap();
    let bytes = SerializedBytes::from_password(&password);
    let savedata = bytes.to_savedata().unwrap();
    let savedata = savedata.normalize();

    // このセーブデータに対応するパスワードを得る。
    // (パスワード「ふ」で始めてすぐ神主に話しかけたときのパスワードと一致するはず)
    let bytes = SerializedBytes::from_savedata(&savedata);
    let password = bytes.to_password();

    println!("{}", password.display_pretty());
}
```

## License

GPLv3

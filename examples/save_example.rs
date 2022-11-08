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

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

//! cargo run --example=load -- 'おにのばか'

use momoden_password::*;

fn main() -> anyhow::Result<()> {
    let password = std::env::args().nth(1).expect("Usage: load <password>");

    let password = Password::parse(&password)?;
    let bytes = SerializedBytes::from_password(&password);

    if let Some(savedata) = bytes.to_savedata() {
        println!("{savedata:?}");
    } else {
        println!(
            "checksum mismatch: embed={:?}, calculated={:?}",
            bytes.checksum_embed(),
            bytes.checksum_calculated()
        );
    };

    Ok(())
}

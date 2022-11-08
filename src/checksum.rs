use crate::bounded::BoundedU8;

/// mod 64 加算によるチェックサム。
pub type ChecksumAdd = BoundedU8<0, 0x3F>;

/// XOR によるチェックサム。
pub type ChecksumXor = BoundedU8<0, 0x3F>;

/// ゲーム状態をシリアライズしたバイト列のチェックサム。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Checksum {
    sum_add: ChecksumAdd,
    sum_xor: ChecksumXor,
}

impl Checksum {
    /// 加算チェックサムと XOR チェックサムを指定して `Checksum` を作る。
    pub const fn new(sum_add: ChecksumAdd, sum_xor: ChecksumXor) -> Self {
        Self { sum_add, sum_xor }
    }

    /// 加算チェックサムを返す。
    pub const fn sum_add(self) -> ChecksumAdd {
        self.sum_add
    }

    /// XOR チェックサムを返す。
    pub const fn sum_xor(self) -> ChecksumXor {
        self.sum_xor
    }
}

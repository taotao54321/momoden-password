use std::num::ParseIntError;

use thiserror::Error;

use crate::macros::assert_unchecked;

/// 値域が `MIN..=MAX` に制限された `u8`。
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundedU8<const MIN: u8, const MAX: u8>(u8);

impl<const MIN: u8, const MAX: u8> BoundedU8<MIN, MAX> {
    /// 最小の内部値。
    pub const MIN_VALUE: u8 = {
        // MIN_VALUE はインスタンス生成時に必ず使われるので、
        // ここに assert を書いておけばコンパイル時に MIN <= MAX のチェックができる。
        // (ただし、インスタンスが生成されない型についてはチェックできない)
        assert!(MIN <= MAX);
        MIN
    };

    /// 最大の内部値。
    pub const MAX_VALUE: u8 = MAX;

    /// 最小値。
    pub const MIN: Self = unsafe { Self::new_unchecked(Self::MIN_VALUE) };

    /// 最大値。
    pub const MAX: Self = unsafe { Self::new_unchecked(Self::MAX_VALUE) };

    /// 引数が値域内にあるかどうかを返す。
    pub const fn in_range(x: u8) -> bool {
        Self::MIN_VALUE <= x && x <= Self::MAX_VALUE
    }

    /// `u8` から `BoundedU8` を作る。引数が値域内になければ `None` を返す。
    pub const fn new(inner: u8) -> Option<Self> {
        if Self::in_range(inner) {
            Some(unsafe { Self::new_unchecked(inner) })
        } else {
            None
        }
    }

    /// `u8` から `BoundedU8` を作る。
    ///
    /// # Safety
    ///
    /// 引数は値域内になければならない。
    pub const unsafe fn new_unchecked(inner: u8) -> Self {
        assert_unchecked!(Self::in_range(inner));
        Self(inner)
    }

    /// 内部値を返す。
    pub const fn get(self) -> u8 {
        self.0
    }

    /// 全ての値を昇順で返す。
    pub fn all(
    ) -> impl Iterator<Item = Self> + DoubleEndedIterator + ExactSizeIterator + std::iter::FusedIterator
    {
        (Self::MIN_VALUE..=Self::MAX_VALUE).map(|i| unsafe { Self::new_unchecked(i) })
    }

    /// 指定された基数で文字列をパースする。
    pub fn from_str_radix(s: &str, radix: u32) -> Result<Self, BoundedIntegerParseError> {
        let value = u8::from_str_radix(s, radix)?;

        if value < Self::MIN_VALUE {
            return Err(BoundedIntegerParseError::BelowMin);
        } else if value > Self::MAX_VALUE {
            return Err(BoundedIntegerParseError::AboveMax);
        }

        Ok(unsafe { Self::new_unchecked(value) })
    }
}

/// 最小値が 0 の場合、デフォルト値は 0 となる。
///
/// 最小値が非 0 の場合、`Default` は実装しない。
impl<const MAX: u8> Default for BoundedU8<0, MAX> {
    fn default() -> Self {
        unsafe { Self::new_unchecked(0) }
    }
}

macro_rules! impl_primitive_from_bounded_u8 {
    ($($ty:ty)*) => {
        $(
            impl<const MIN: u8, const MAX: u8> From<BoundedU8<MIN, MAX>> for $ty {
                fn from(x: BoundedU8<MIN, MAX>) -> Self {
                    Self::from(x.get())
                }
            }
        )*
    };
}

impl_primitive_from_bounded_u8!(i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);

impl<const MIN: u8, const MAX: u8> std::str::FromStr for BoundedU8<MIN, MAX> {
    type Err = BoundedIntegerParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_radix(s, 10)
    }
}

macro_rules! impl_fmt_traits {
    ($($trait:ident),*) => {
        $(
            impl<const MIN: u8, const MAX: u8> std::fmt::$trait for BoundedU8<MIN, MAX> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::$trait::fmt(&self.0, f)
                }
            }
        )*
    };
}

impl_fmt_traits!(Binary, Debug, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex);

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum BoundedIntegerParseError {
    /// 最小値よりも小さい。
    #[error("number is below MIN")]
    BelowMin,

    /// 最大値よりも大きい。
    #[error("number is above MAX")]
    AboveMax,

    /// パースエラー。
    #[error(transparent)]
    Parse(#[from] ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_radix() {
        type Num = BoundedU8<3, 44>;

        assert_eq!(Num::from_str_radix("3", 10).unwrap().get(), 3);
        assert_eq!(Num::from_str_radix("2C", 16).unwrap().get(), 44);

        assert!(matches!(
            Num::from_str_radix("A", 10),
            Err(BoundedIntegerParseError::Parse(_))
        ));
        assert!(matches!(
            Num::from_str_radix("2", 10),
            Err(BoundedIntegerParseError::BelowMin)
        ));
        assert!(matches!(
            Num::from_str_radix("45", 10),
            Err(BoundedIntegerParseError::AboveMax)
        ));
    }
}

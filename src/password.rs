use std::fmt::Write as _;

use arrayvec::ArrayVec;
use thiserror::Error;

use crate::macros::assert_unchecked;
use crate::serialized::SerializedBytes;

/// `Password` の内部バッファ。
pub type PasswordInner = ArrayVec<PasswordChar, { Password::MAX_LEN }>;

/// パスワード。
#[repr(transparent)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Password(PasswordInner);

impl Password {
    /// パスワードの最小文字数。
    pub const MIN_LEN: usize = 1;

    /// パスワードの最大文字数。
    pub const MAX_LEN: usize = 38;

    /// 音楽室に入る特殊パスワード(末尾を削っていない完全なもの)。
    pub const SPECIAL_AUDIO: &[PasswordChar] = {
        use PasswordChar::*;
        &[Su, Be, Te, No, Ki, Yo, Ku, Ga, Ki, Ki, Ta, I, Na]
    };

    /// 美術室に入る特殊パスワード(末尾を削っていない完全なもの)。
    pub const SPECIAL_ENEMY: &[PasswordChar] = {
        use PasswordChar::*;
        &[Su, Be, Te, No, Te, Ki, Ga, Mi, Ta, I, Na]
    };

    /// `PasswordChar` のスライスから `Password` を作る。文字数が範囲外なら `None` を返す。
    pub fn new(chars: &[PasswordChar]) -> Option<Self> {
        matches!(chars.len(), Self::MIN_LEN..=Self::MAX_LEN)
            .then(|| unsafe { Self::new_unchecked(chars) })
    }

    /// `PasswordChar` のスライスから `Password` を作る。
    ///
    /// # Safety
    ///
    /// 文字数は `Password::MIN_LEN..=Password::MAX_LEN` の範囲内になければならない。
    pub unsafe fn new_unchecked(chars: &[PasswordChar]) -> Self {
        assert_unchecked!(matches!(chars.len(), Self::MIN_LEN..=Self::MAX_LEN));
        Self(chars.iter().copied().collect())
    }

    /// ひらがな文字列をパースして `Password` を作る。
    pub fn parse(s: &str) -> Result<Self, PasswordParseError> {
        let mut inner = PasswordInner::new();

        for (i, c) in s.chars().enumerate() {
            let pc = PasswordChar::from_char(c)
                .ok_or(PasswordParseError::InvalidChar { pos: i, ch: c })?;
            inner
                .try_push(pc)
                .map_err(|_| PasswordParseError::InvalidLength)?;
        }

        if inner.is_empty() {
            return Err(PasswordParseError::InvalidLength);
        }

        Ok(Self(inner))
    }

    /// 内部バッファを返す。
    pub fn into_inner(self) -> PasswordInner {
        self.0
    }

    /// パスワード全体を含むスライスを返す。
    pub fn as_slice(&self) -> &[PasswordChar] {
        self.0.as_slice()
    }

    /// 文字数を返す。
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// パスワードをひらがな文字列(空白区切りなし)としてフォーマットする `Display` オブジェクトを返す。
    pub fn display(&self) -> PasswordDisplay {
        PasswordDisplay { password: self }
    }

    /// パスワードをひらがな文字列(空白区切りあり)としてフォーマットする `Display` オブジェクトを返す。
    pub fn display_pretty(&self) -> PasswordDisplayPretty {
        PasswordDisplayPretty { password: self }
    }

    /// パスワードを内部値の 16 進ダンプとしてフォーマットする `Display` オブジェクトを返す。
    ///
    /// 結果の文字列は Mesen や FCEUX のメモリエディタにそのまま貼り付け可能。
    pub fn display_hex(&self) -> PasswordDisplayHex {
        PasswordDisplayHex { password: self }
    }

    /// パスワードが有効(ゲーム状態としてロードできる)かどうかを返す。
    pub fn is_valid(&self) -> bool {
        SerializedBytes::from_password(self).checksum_is_ok()
    }

    /// パスワードの 2 文字目のみを見たとき、それが有効なパスワードになりえないかどうかを返す。
    pub fn is_invalid_second_char(pc_second: PasswordChar) -> bool {
        // 一般に add と xor の偶奇は一致するので、
        // sum_add と sum_xor の偶奇が異なるなら有効なパスワードにはなりえない。
        //
        // パスワードの先頭 2 文字を prefix とおくと、sum_add, sum_xor の計算式は以下の通り:
        //
        //   sum_add = (prefix[0] ^ 0x1F).wrapping_sub(0x05) & 0x3F
        //   sum_xor = (prefix[1] ^ prefix[0]).wrapping_sub(0x19) & 0x3F
        //
        // bit0 のみに注目すると:
        //
        //   | prefix[0] | prefix[1] | sum_add | sum_xor | invalid |
        //   | --------- | --------- | ------- | ------- | ------- |
        //   |     0     |     0     |    0    |    1    |    1    |
        //   |     0     |     1     |    0    |    0    |    0    |
        //   |     1     |     0     |    1    |    0    |    1    |
        //   |     1     |     1     |    1    |    1    |    0    |
        //
        // よって、prefix[1] が偶数ならば有効なパスワードにはなりえない。

        pc_second.to_inner() % 2 == 0
    }

    /// 特殊パスワード(音楽室/美術室)かどうかを返す。
    pub fn is_special(&self) -> bool {
        self.is_special_audio() || self.is_special_enemy()
    }

    /// 音楽室に入る特殊パスワードかどうかを返す。
    pub fn is_special_audio(&self) -> bool {
        Self::SPECIAL_AUDIO.starts_with(self)
    }

    /// 美術室に入る特殊パスワードかどうかを返す。
    pub fn is_special_enemy(&self) -> bool {
        Self::SPECIAL_ENEMY.starts_with(self)
    }
}

impl std::ops::Deref for Password {
    type Target = [PasswordChar];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl AsRef<[PasswordChar]> for Password {
    fn as_ref(&self) -> &[PasswordChar] {
        self
    }
}

impl std::borrow::Borrow<[PasswordChar]> for Password {
    fn borrow(&self) -> &[PasswordChar] {
        self
    }
}

impl IntoIterator for Password {
    type Item = PasswordChar;
    type IntoIter = arrayvec::IntoIter<PasswordChar, { Self::MAX_LEN }>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<'a> IntoIterator for &'a Password {
    type Item = &'a PasswordChar;
    type IntoIter = std::slice::Iter<'a, PasswordChar>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct PasswordDisplay<'a> {
    password: &'a Password,
}

impl std::fmt::Display for PasswordDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for pc in self.password {
            f.write_char(pc.to_char())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct PasswordDisplayPretty<'a> {
    password: &'a Password,
}

impl std::fmt::Display for PasswordDisplayPretty<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const LENS: &[usize] = &[5, 7, 5, 7, 7, 7];

        let mut pcs = self.password.iter().peekable();
        let pcs = pcs.by_ref();

        for &len in LENS {
            let chunk = pcs.take(len);
            for &pc in chunk {
                f.write_char(pc.to_char())?;
            }
            if pcs.peek().is_some() {
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct PasswordDisplayHex<'a> {
    password: &'a Password,
}

impl std::fmt::Display for PasswordDisplayHex<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, &pc) in self.password.iter().enumerate() {
            if i != 0 {
                f.write_char(' ')?;
            }
            write!(f, "{:02X}", pc.to_inner())?;
        }

        Ok(())
    }
}

/// パスワード内の文字。
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PasswordChar {
    A = 0x00,
    I = 0x01,
    U = 0x02,
    E = 0x03,
    O = 0x04,
    Ka = 0x05,
    Ki = 0x06,
    Ku = 0x07,
    Ke = 0x08,
    Ko = 0x09,
    Sa = 0x0A,
    Si = 0x0B,
    Su = 0x0C,
    Se = 0x0D,
    So = 0x0E,
    Ta = 0x0F,
    Ti = 0x10,
    Tu = 0x11,
    Te = 0x12,
    To = 0x13,
    Na = 0x14,
    Ni = 0x15,
    Nu = 0x16,
    Ne = 0x17,
    No = 0x18,
    Ha = 0x19,
    Hi = 0x1A,
    Hu = 0x1B,
    He = 0x1C,
    Ho = 0x1D,
    Ma = 0x1E,
    Mi = 0x1F,
    Mu = 0x20,
    Me = 0x21,
    Mo = 0x22,
    Ya = 0x23,
    Yu = 0x24,
    Yo = 0x25,
    Ra = 0x26,
    Ri = 0x27,
    Ru = 0x28,
    Re = 0x29,
    Ro = 0x2A,
    Wa = 0x2B,
    Ga = 0x2C,
    Gi = 0x2D,
    Gu = 0x2E,
    Ge = 0x2F,
    Go = 0x30,
    Za = 0x31,
    Zi = 0x32,
    Zu = 0x33,
    Ze = 0x34,
    Zo = 0x35,
    Ba = 0x36,
    Bi = 0x37,
    Bu = 0x38,
    Be = 0x39,
    Bo = 0x3A,
    Pa = 0x3B,
    Pi = 0x3C,
    Pu = 0x3D,
    Pe = 0x3E,
    Po = 0x3F,
}

impl PasswordChar {
    /// 内部値から `PasswordChar` を作る。無効値に対しては `None` を返す。
    pub const fn from_inner(inner: u8) -> Option<Self> {
        if matches!(inner, 0..=0x3F) {
            Some(unsafe { Self::from_inner_unchecked(inner) })
        } else {
            None
        }
    }

    /// 内部値から `PasswordChar` を作る。
    ///
    /// # Safety
    ///
    /// `inner` は有効値、即ち `0..=0x3F` でなければならない。
    pub const unsafe fn from_inner_unchecked(inner: u8) -> Self {
        assert_unchecked!(matches!(inner, 0..=0x3F));
        std::mem::transmute(inner)
    }

    /// ひらがな文字を `PasswordChar` に変換する。無効な文字に対しては `None` を返す。
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'あ' => Some(Self::A),
            'い' => Some(Self::I),
            'う' => Some(Self::U),
            'え' => Some(Self::E),
            'お' => Some(Self::O),
            'か' => Some(Self::Ka),
            'き' => Some(Self::Ki),
            'く' => Some(Self::Ku),
            'け' => Some(Self::Ke),
            'こ' => Some(Self::Ko),
            'さ' => Some(Self::Sa),
            'し' => Some(Self::Si),
            'す' => Some(Self::Su),
            'せ' => Some(Self::Se),
            'そ' => Some(Self::So),
            'た' => Some(Self::Ta),
            'ち' => Some(Self::Ti),
            'つ' => Some(Self::Tu),
            'て' => Some(Self::Te),
            'と' => Some(Self::To),
            'な' => Some(Self::Na),
            'に' => Some(Self::Ni),
            'ぬ' => Some(Self::Nu),
            'ね' => Some(Self::Ne),
            'の' => Some(Self::No),
            'は' => Some(Self::Ha),
            'ひ' => Some(Self::Hi),
            'ふ' => Some(Self::Hu),
            'へ' => Some(Self::He),
            'ほ' => Some(Self::Ho),
            'ま' => Some(Self::Ma),
            'み' => Some(Self::Mi),
            'む' => Some(Self::Mu),
            'め' => Some(Self::Me),
            'も' => Some(Self::Mo),
            'や' => Some(Self::Ya),
            'ゆ' => Some(Self::Yu),
            'よ' => Some(Self::Yo),
            'ら' => Some(Self::Ra),
            'り' => Some(Self::Ri),
            'る' => Some(Self::Ru),
            'れ' => Some(Self::Re),
            'ろ' => Some(Self::Ro),
            'わ' => Some(Self::Wa),
            'が' => Some(Self::Ga),
            'ぎ' => Some(Self::Gi),
            'ぐ' => Some(Self::Gu),
            'げ' => Some(Self::Ge),
            'ご' => Some(Self::Go),
            'ざ' => Some(Self::Za),
            'じ' => Some(Self::Zi),
            'ず' => Some(Self::Zu),
            'ぜ' => Some(Self::Ze),
            'ぞ' => Some(Self::Zo),
            'ば' => Some(Self::Ba),
            'び' => Some(Self::Bi),
            'ぶ' => Some(Self::Bu),
            'べ' => Some(Self::Be),
            'ぼ' => Some(Self::Bo),
            'ぱ' => Some(Self::Pa),
            'ぴ' => Some(Self::Pi),
            'ぷ' => Some(Self::Pu),
            'ぺ' => Some(Self::Pe),
            'ぽ' => Some(Self::Po),
            _ => None,
        }
    }

    /// 内部値を返す。
    pub const fn to_inner(self) -> u8 {
        self as u8
    }

    /// 対応するひらがな文字を返す。
    pub const fn to_char(self) -> char {
        match self {
            Self::A => 'あ',
            Self::I => 'い',
            Self::U => 'う',
            Self::E => 'え',
            Self::O => 'お',
            Self::Ka => 'か',
            Self::Ki => 'き',
            Self::Ku => 'く',
            Self::Ke => 'け',
            Self::Ko => 'こ',
            Self::Sa => 'さ',
            Self::Si => 'し',
            Self::Su => 'す',
            Self::Se => 'せ',
            Self::So => 'そ',
            Self::Ta => 'た',
            Self::Ti => 'ち',
            Self::Tu => 'つ',
            Self::Te => 'て',
            Self::To => 'と',
            Self::Na => 'な',
            Self::Ni => 'に',
            Self::Nu => 'ぬ',
            Self::Ne => 'ね',
            Self::No => 'の',
            Self::Ha => 'は',
            Self::Hi => 'ひ',
            Self::Hu => 'ふ',
            Self::He => 'へ',
            Self::Ho => 'ほ',
            Self::Ma => 'ま',
            Self::Mi => 'み',
            Self::Mu => 'む',
            Self::Me => 'め',
            Self::Mo => 'も',
            Self::Ya => 'や',
            Self::Yu => 'ゆ',
            Self::Yo => 'よ',
            Self::Ra => 'ら',
            Self::Ri => 'り',
            Self::Ru => 'る',
            Self::Re => 'れ',
            Self::Ro => 'ろ',
            Self::Wa => 'わ',
            Self::Ga => 'が',
            Self::Gi => 'ぎ',
            Self::Gu => 'ぐ',
            Self::Ge => 'げ',
            Self::Go => 'ご',
            Self::Za => 'ざ',
            Self::Zi => 'じ',
            Self::Zu => 'ず',
            Self::Ze => 'ぜ',
            Self::Zo => 'ぞ',
            Self::Ba => 'ば',
            Self::Bi => 'び',
            Self::Bu => 'ぶ',
            Self::Be => 'べ',
            Self::Bo => 'ぼ',
            Self::Pa => 'ぱ',
            Self::Pi => 'ぴ',
            Self::Pu => 'ぷ',
            Self::Pe => 'ぺ',
            Self::Po => 'ぽ',
        }
    }

    /// 全ての文字を昇順で返す。
    pub const fn all() -> [Self; 0x40] {
        use PasswordChar::*;

        #[rustfmt::skip]
        const ALL: [PasswordChar; 0x40] = [
            A,  I,  U,  E,  O,
            Ka, Ki, Ku, Ke, Ko,
            Sa, Si, Su, Se, So,
            Ta, Ti, Tu, Te, To,
            Na, Ni, Nu, Ne, No,
            Ha, Hi, Hu, He, Ho,
            Ma, Mi, Mu, Me, Mo,
            Ya, Yu, Yo,
            Ra, Ri, Ru, Re, Ro,
            Wa,
            Ga, Gi, Gu, Ge, Go,
            Za, Zi, Zu, Ze, Zo,
            Ba, Bi, Bu, Be, Bo,
            Pa, Pi, Pu, Pe, Po,
        ];

        ALL
    }
}

/// パスワードのパース時に発生しうるエラー。
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum PasswordParseError {
    /// パスワードの文字数が正しくない。
    #[error(
        "password must contain {}..={} chars",
        Password::MIN_LEN,
        Password::MAX_LEN
    )]
    InvalidLength,

    /// パスワードに無効な文字が含まれている。
    #[error("password contains an invalid character '{ch}' at position {pos}")]
    InvalidChar { pos: usize, ch: char },
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    #[test]
    fn test_password_new() {
        assert_equal(
            Password::new(&[PasswordChar::A]).unwrap(),
            [PasswordChar::A],
        );
        assert_equal(
            Password::new(&[PasswordChar::Po; Password::MAX_LEN]).unwrap(),
            [PasswordChar::Po; Password::MAX_LEN],
        );

        assert_eq!(Password::new(&[]), None);
        assert_eq!(
            Password::new(&[PasswordChar::A; Password::MAX_LEN + 1]),
            None
        );
    }

    #[test]
    fn test_password_parse() {
        assert_equal(Password::parse("ふ").unwrap(), [PasswordChar::Hu]);
        assert_equal(
            Password::parse("おにのばか").unwrap(),
            [
                PasswordChar::O,
                PasswordChar::Ni,
                PasswordChar::No,
                PasswordChar::Ba,
                PasswordChar::Ka,
            ],
        );

        assert_eq!(Password::parse(""), Err(PasswordParseError::InvalidLength));
        assert_eq!(
            Password::parse(&"あ".repeat(Password::MAX_LEN + 1)),
            Err(PasswordParseError::InvalidLength)
        );
        assert_eq!(
            Password::parse("あいう?えお"),
            Err(PasswordParseError::InvalidChar { pos: 3, ch: '?' })
        );
    }

    #[test]
    fn test_password_display() {
        assert_eq!(
            Password::new(&[PasswordChar::A, PasswordChar::Ba, PasswordChar::Po])
                .unwrap()
                .display()
                .to_string(),
            "あばぽ"
        );
    }

    #[test]
    fn test_password_display_pretty() {
        assert_eq!(
            Password::parse("あ").unwrap().display_pretty().to_string(),
            "あ"
        );
        assert_eq!(
            Password::parse("あああああ")
                .unwrap()
                .display_pretty()
                .to_string(),
            "あああああ"
        );
        assert_eq!(
            Password::parse("あああああいいいいいいい")
                .unwrap()
                .display_pretty()
                .to_string(),
            "あああああ いいいいいいい"
        );
        assert_eq!(
            Password::parse("あああああいいいいいいいううううう")
                .unwrap()
                .display_pretty()
                .to_string(),
            "あああああ いいいいいいい ううううう"
        );
        assert_eq!(
            Password::parse("あああああいいいいいいいうううううえええええええ")
                .unwrap()
                .display_pretty()
                .to_string(),
            "あああああ いいいいいいい ううううう えええええええ"
        );
        assert_eq!(
            Password::parse("あああああいいいいいいいうううううえええええええおおおおおおお")
                .unwrap()
                .display_pretty()
                .to_string(),
            "あああああ いいいいいいい ううううう えええええええ おおおおおおお"
        );
        assert_eq!(
            Password::parse(
                "あああああいいいいいいいうううううえええええええおおおおおおおかかかかかかか"
            )
            .unwrap()
            .display_pretty()
            .to_string(),
            "あああああ いいいいいいい ううううう えええええええ おおおおおおお かかかかかかか"
        );
    }

    #[test]
    fn test_password_display_hex() {
        assert_eq!(
            Password::parse("あばぽ").unwrap().display_hex().to_string(),
            "00 36 3F"
        );
    }

    #[test]
    fn test_password_is_invalid_second_char() {
        fn naive(prefix: [PasswordChar; 2]) -> bool {
            let password = Password::new(&prefix).unwrap();
            let bytes = SerializedBytes::from_password(&password);
            let checksum = bytes.checksum_embed();
            (checksum.sum_add().get() % 2) != (checksum.sum_xor().get() % 2)
        }

        for (pc0, pc1) in itertools::iproduct!(PasswordChar::all(), PasswordChar::all()) {
            assert_eq!(Password::is_invalid_second_char(pc1), naive([pc0, pc1]));
        }
    }

    #[test]
    fn test_password_is_special() {
        assert!(Password::parse("す").unwrap().is_special());
        assert!(Password::parse("すべてのき").unwrap().is_special());
        assert!(Password::parse("すべてのきよくがききたいな")
            .unwrap()
            .is_special());
        assert!(Password::parse("すべてのてき").unwrap().is_special());
        assert!(Password::parse("すべてのてきがみたいな")
            .unwrap()
            .is_special());

        assert!(!Password::parse("あ").unwrap().is_special());
        assert!(!Password::parse("すべてのきよくがききたいなあ")
            .unwrap()
            .is_special());
        assert!(!Password::parse("すべてのてきがみたいなあ")
            .unwrap()
            .is_special());
    }
}

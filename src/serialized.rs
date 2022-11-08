use arrayvec::ArrayVec;
use bitvec::prelude::*;

use crate::bounded::BoundedU8;
use crate::checksum::{Checksum, ChecksumAdd, ChecksumXor};
use crate::macros::assert_unchecked;
use crate::password::{Password, PasswordChar};
use crate::savedata::*;

/// `SerializedBytes` の要素。
pub type SerializedByte = BoundedU8<0, 0x3F>;

/// `SerializedBytes` の内部バッファ。
pub type SerializedBytesInner = ArrayVec<SerializedByte, { Password::MAX_LEN }>;

/// ゲーム状態をシリアライズしたバイト列。
///
/// 各要素は実質 6bit 値。
/// ゲーム状態を表すビット列は各要素の上位ビットから順に格納される。つまり以下のようになる:
///
/// ```text
/// ..012345 ..6789AB ..CDEFGH ...(以下略)
/// ```
///
/// パスワードをデコードして得られたバイト列のバイト数は元のパスワードの文字数に等しい。
/// デシリアライズの際にビット数が不足する場合、足りないビットは全て 1 として扱われる。
#[repr(transparent)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SerializedBytes(SerializedBytesInner);

impl SerializedBytes {
    /// パスワードのエンコード時に用いる加算値テーブル。
    const ENCODE_ADD_TABLE: [u8; 4] = [0x05, 0x19, 0x32, 0x21];

    /// `SerializedByte` のスライスから `SerializedBytes` を作る。バイト数が範囲外なら `None` を返す。
    pub fn new(buf: &[SerializedByte]) -> Option<Self> {
        matches!(buf.len(), Password::MIN_LEN..=Password::MAX_LEN)
            .then(|| unsafe { Self::new_unchecked(buf) })
    }

    /// `SerializedByte` のスライスから `SerializedBytes` を作る。
    ///
    /// # Safety
    ///
    /// バイト数は `Password::MIN_LEN..=Password::MAX_LEN` の範囲内になければならない。
    pub unsafe fn new_unchecked(buf: &[SerializedByte]) -> Self {
        assert_unchecked!(matches!(buf.len(), Password::MIN_LEN..=Password::MAX_LEN));
        Self(buf.iter().copied().collect())
    }

    /// パスワードをデコードして `SerializedBytes` を得る。特殊パスワードでも構わずデコードする。
    ///
    /// 戻り値はチェックサムが一致していない可能性がある。
    pub fn from_password(password: &Password) -> Self {
        // 演算は u8 で行う。最終結果は 6bit 値になる。
        let mut inner: ArrayVec<u8, { Password::MAX_LEN }> = password
            .iter()
            .copied()
            .map(PasswordChar::to_inner)
            .collect();

        // デコード: XOR
        for i in (1..inner.len()).rev() {
            inner[i] ^= inner[i - 1];
        }
        inner[0] ^= 0x1F;

        // デコード: mod 64 減算
        for (i, b) in inner.iter_mut().enumerate() {
            *b = b.wrapping_sub(Self::ENCODE_ADD_TABLE[i % 4]);
            *b &= 0x3F;
        }

        let inner: SerializedBytesInner = inner
            .into_iter()
            .map(|b| unsafe { SerializedByte::new_unchecked(b) })
            .collect();

        Self(inner)
    }

    /// `SerializedBytes` をパスワードにエンコードする。
    pub fn to_password(&self) -> Password {
        // 演算は u8 で行う。最終結果は 6bit 値になる。
        let mut inner: ArrayVec<u8, { Password::MAX_LEN }> =
            self.iter().copied().map(SerializedByte::get).collect();

        // エンコード: mod 64 加算
        for (i, b) in inner.iter_mut().enumerate() {
            *b = b.wrapping_add(Self::ENCODE_ADD_TABLE[i % 4]);
            *b &= 0x3F;
        }

        // エンコード: XOR
        inner[0] ^= 0x1F;
        for i in 1..inner.len() {
            inner[i] ^= inner[i - 1];
        }

        let inner: ArrayVec<PasswordChar, { Password::MAX_LEN }> = inner
            .into_iter()
            .map(|b| unsafe { PasswordChar::from_inner_unchecked(b) })
            .collect();

        unsafe { Password::new_unchecked(&inner) }
    }

    /// ゲーム状態をシリアライズして `SerializedBytes` を得る。
    ///
    /// 戻り値はチェックサムが一致していることが保証される。
    pub fn from_savedata(savedata: &Savedata) -> Self {
        let mut bits = SerializedBits::new();

        let xp_lo = savedata.xp as u8;
        let xp_hi = (savedata.xp >> 8) as u8;
        let purse_lo = savedata.purse as u8;
        let purse_hi = (savedata.purse >> 8) as u8;

        bits.push_bits(8, savedata.age_timer_hi);
        bits.push_bits(8, purse_hi);
        bits.push_bits(8, savedata.age);
        bits.push_bits(8, purse_lo);
        bits.push_bits(8, xp_lo);
        bits.push_bits(6, savedata.deposit.get());
        bits.push_bits(8, xp_hi);
        serialize_spells(&mut bits, savedata.spells);
        serialize_treasures(&mut bits, savedata.treasures);
        bits.push_bits(4, savedata.respawn.get());
        serialize_bookmarks1(&mut bits, savedata.bookmarks);
        serialize_minions(&mut bits, savedata.minions);
        serialize_bookmarks0(&mut bits, savedata.bookmarks);
        serialize_events(&mut bits, savedata.events);
        serialize_equipment(&mut bits, savedata.equipment);
        serialize_inventory(&mut bits, &savedata.inventory);

        bits.to_bytes()
    }

    /// `SerializedBytes` をゲーム状態にデシリアライズする。チェックサムが一致していなければ `None` を返す。
    pub fn to_savedata(&self) -> Option<Savedata> {
        if !self.checksum_is_ok() {
            return None;
        }

        let bits = SerializedBits::from_bytes(self);
        let bits = bits.as_bitslice();

        let (age_timer_hi, bits) = deserialize_bits(bits, 8);
        let (purse_hi, bits) = deserialize_bits(bits, 8);
        let (age, bits) = deserialize_bits(bits, 8);
        let (purse_lo, bits) = deserialize_bits(bits, 8);
        let (xp_lo, bits) = deserialize_bits(bits, 8);
        let (deposit, bits) = deserialize_bits(bits, 6);
        let (xp_hi, bits) = deserialize_bits(bits, 8);
        let (spells, bits) = deserialize_bits(bits, 8);
        let (treasures, bits) = deserialize_bits(bits, 5);
        let (respawn, bits) = deserialize_bits(bits, 4);
        let (bookmarks1, bits) = deserialize_bits(bits, 2);
        let (minions, bits) = deserialize_bits(bits, 3);
        let (bookmarks0, bits) = deserialize_bits(bits, 8);
        let (events, bits) = deserialize_bits(bits, 8);
        let (equipment, bits) = deserialize_equipment(bits);
        let (inventory, _) = deserialize_inventory(bits);

        let xp = u16::from(xp_lo) | (u16::from(xp_hi) << 8);
        let purse = u16::from(purse_lo) | (u16::from(purse_hi) << 8);
        let deposit = unsafe { Deposit::new_unchecked(deposit) };
        let spells = unpack_spells(spells);
        let events = unpack_events(events);
        let treasures = unpack_treasures(treasures);
        let minions = unpack_minions(minions);
        let bookmarks = unpack_bookmarks([bookmarks0, bookmarks1]);
        let respawn = unsafe { RespawnId::new_unchecked(respawn) };

        Some(Savedata {
            xp,
            purse,
            deposit,
            age,
            age_timer_hi,
            spells,
            events,
            treasures,
            minions,
            bookmarks,
            respawn,
            equipment,
            inventory,
        })
    }

    /// 内部バッファを返す。
    pub fn into_inner(self) -> SerializedBytesInner {
        self.0
    }

    /// バイト列全体を含むスライスを返す。
    pub fn as_slice(&self) -> &[SerializedByte] {
        self.0.as_slice()
    }

    /// バイト列全体を含む可変スライスを返す。
    pub fn as_mut_slice(&mut self) -> &mut [SerializedByte] {
        self.0.as_mut_slice()
    }

    /// バイト数を返す。
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// バイト列に格納されたチェックサムを返す。
    pub fn checksum_embed(&self) -> Checksum {
        // 最低でも 1 バイトある。
        // また、足りないビットは全て 1 になる。
        let sum_add = self[0];
        let sum_xor = if self.len() >= 2 {
            self[1]
        } else {
            ChecksumXor::MAX
        };

        Checksum::new(sum_add, sum_xor)
    }

    /// バイト列の内容から計算されたチェックサムを返す。
    pub fn checksum_calculated(&self) -> Checksum {
        // self[2..] 以降が計算対象(先頭 2 バイトはチェックサム格納領域)。
        // 2 バイト以下しかない場合、0x3F が 1 個あるものとして扱う。

        if self.len() <= 2 {
            return Checksum::new(ChecksumAdd::MAX, ChecksumXor::MAX);
        }

        // 演算は u8 で行う。最終結果は 6bit 値になる。
        let mut sum_add: u8 = 0;
        let mut sum_xor: u8 = 0;
        for b in self[2..].iter().map(|b| b.get()) {
            sum_add = sum_add.wrapping_add(b);
            sum_xor ^= b;
        }
        sum_add &= 0x3F;

        unsafe {
            Checksum::new(
                ChecksumAdd::new_unchecked(sum_add),
                ChecksumXor::new_unchecked(sum_xor),
            )
        }
    }

    /// バイト列に格納されたチェックサムと計算されたチェックサムが一致するかどうかを返す。
    pub fn checksum_is_ok(&self) -> bool {
        self.checksum_embed() == self.checksum_calculated()
    }
}

impl std::ops::Deref for SerializedBytes {
    type Target = [SerializedByte];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl std::ops::DerefMut for SerializedBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl AsRef<[SerializedByte]> for SerializedBytes {
    fn as_ref(&self) -> &[SerializedByte] {
        self
    }
}

impl AsMut<[SerializedByte]> for SerializedBytes {
    fn as_mut(&mut self) -> &mut [SerializedByte] {
        self
    }
}

impl std::borrow::Borrow<[SerializedByte]> for SerializedBytes {
    fn borrow(&self) -> &[SerializedByte] {
        self
    }
}

impl std::borrow::BorrowMut<[SerializedByte]> for SerializedBytes {
    fn borrow_mut(&mut self) -> &mut [SerializedByte] {
        self
    }
}

impl IntoIterator for SerializedBytes {
    type Item = SerializedByte;
    type IntoIter = arrayvec::IntoIter<SerializedByte, { Password::MAX_LEN }>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<'a> IntoIterator for &'a SerializedBytes {
    type Item = &'a SerializedByte;
    type IntoIter = std::slice::Iter<'a, SerializedByte>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

type SerializedBitArray = BitArr!(for SerializedBits::CAPACITY, in usize, Msb0);
type SerializedBitSlice = BitSlice<usize, Msb0>;

/// ゲーム状態のシリアライズ用ビットベクター。容量固定。チェックサムは含まない。
#[derive(Debug, Default)]
struct SerializedBits {
    inner: SerializedBitArray,
    len: usize,
}

impl SerializedBits {
    // チェックサムを除いたゲーム状態は最大 159bit。
    // 簡単のため、6 の倍数に切り上げて 162bit とする。
    const CAPACITY: usize = 6 * 27;

    /// 空の `SerializedBits` を返す。
    fn new() -> Self {
        Self::default()
    }

    /// `SerializedBytes` から変換する。
    fn from_bytes(bytes: &SerializedBytes) -> Self {
        let mut this = Self::default();

        // チェックサムを除いたバイト列をビット列に変換する。
        // 意味を持つのは CAPACITY/6 バイトまでなので、それより多くは読まない。
        if bytes.len() > 2 {
            for &b in bytes[2..].iter().take(Self::CAPACITY / 6) {
                this.push_bits(6, b.get());
            }
        }

        // 長さ CAPACITY になるまで 1 を追加する。
        this.inner[this.len..Self::CAPACITY].fill(true);
        this.len = Self::CAPACITY;

        this
    }

    /// `SerializedBytes` に変換する。
    #[allow(clippy::wrong_self_convention)]
    fn to_bytes(&mut self) -> SerializedBytes {
        // 長さが 6 の倍数になるまで 0 を追加する。
        let len = (self.len + 6 - 1) / 6 * 6;
        self.inner[self.len..len].fill(false);
        self.len = len;

        // ビット列をバイト列に変換する。先頭にチェックサム格納用の 2 バイトを補う。
        let inner: SerializedBytesInner = [0; 2]
            .into_iter()
            .chain(
                self.as_bitslice()
                    .chunks_exact(6)
                    .map(|chunk| chunk.load_be::<u8>()),
            )
            .map(|b| unsafe { SerializedByte::new_unchecked(b) })
            .collect();
        let mut bytes = unsafe { SerializedBytes::new_unchecked(&inner) };

        // チェックサムを計算し、格納する。
        let checksum = bytes.checksum_calculated();
        bytes[0] = checksum.sum_add();
        bytes[1] = checksum.sum_xor();

        bytes
    }

    /// ビットベクター全体を表すビットスライスを返す。
    fn as_bitslice(&self) -> &SerializedBitSlice {
        &self.inner[..self.len]
    }

    /// 1 個のビットを末尾に追加する。
    fn push_bit(&mut self, bit: bool) {
        self.inner.set(self.len, bit);
        self.len += 1;
    }

    /// `n` 個のビットを末尾に追加する。
    ///
    /// `bits` は追加するビットたちを右詰めした値。
    /// たとえば `[1, 0, 1, 1, 0]` を追加するなら `0b00010110` を渡す。
    fn push_bits(&mut self, n: usize, bits: u8) {
        self.inner[self.len..][..n].store_be(bits);
        self.len += n;
    }
}

fn serialize_spells(bits: &mut SerializedBits, spells: Spells) {
    bits.push_bit(spells.houhi);
    bits.push_bit(spells.dadadidi);
    bits.push_bit(spells.fuyuu);
    bits.push_bit(spells.mankintan);
    bits.push_bit(spells.hien);
    bits.push_bit(spells.inazuma);
    bits.push_bit(spells.rokkaku);
    bits.push_bit(spells.kintan);
}

fn serialize_events(bits: &mut SerializedBits, events: Events) {
    bits.push_bit(events.hohoemi);
    bits.push_bit(events.dragon);
    bits.push_bit(events.sarukani);
    bits.push_bit(events.murata);
    bits.push_bit(events.netaro);
    bits.push_bit(events.urashima);
    bits.push_bit(events.kintaro);
    bits.push_bit(events.hanasaka);
}

fn serialize_treasures(bits: &mut SerializedBits, treasures: Treasures) {
    bits.push_bit(treasures.swallow);
    bits.push_bit(treasures.hourai);
    bits.push_bit(treasures.hotoke);
    bits.push_bit(treasures.fur);
    bits.push_bit(treasures.dragon);
}

fn serialize_minions(bits: &mut SerializedBits, minions: Minions) {
    bits.push_bit(minions.monkey);
    bits.push_bit(minions.pheasant);
    bits.push_bit(minions.dog);
}

fn serialize_bookmarks0(bits: &mut SerializedBits, bookmarks: Bookmarks) {
    bits.push_bit(bookmarks.taketori);
    bits.push_bit(bookmarks.sarukani);
    bits.push_bit(bookmarks.kibou);
    bits.push_bit(bookmarks.netaro);
    bits.push_bit(bookmarks.urashima);
    bits.push_bit(bookmarks.kintaro);
    bits.push_bit(bookmarks.hanasaka);
    bits.push_bit(bookmarks.tabidachi);
}

fn serialize_bookmarks1(bits: &mut SerializedBits, bookmarks: Bookmarks) {
    bits.push_bit(bookmarks.hien);
    bits.push_bit(bookmarks.hohoemi);
}

fn serialize_equipment(bits: &mut SerializedBits, equipment: Equipment) {
    bits.push_bits(2, equipment.helm.get());
    bits.push_bits(4, equipment.weapon.get());
    bits.push_bits(4, equipment.armor.get());
    bits.push_bits(3, equipment.shoes.get());
    bits.push_bits(2, equipment.accessory0.get());
    bits.push_bits(2, equipment.accessory1.get());
    bits.push_bits(1, equipment.accessory2.get());
    bits.push_bits(1, equipment.accessory3.get());
}

fn serialize_inventory(bits: &mut SerializedBits, inventory: &Inventory) {
    for item in inventory {
        bits.push_bits(6, item.get());
    }

    if !inventory.is_full() {
        bits.push_bits(6, 0);
    }
}

fn unpack_spells(spells: u8) -> Spells {
    let bits = spells.view_bits::<Lsb0>();

    Spells {
        kintan: bits[0],
        rokkaku: bits[1],
        inazuma: bits[2],
        hien: bits[3],
        mankintan: bits[4],
        fuyuu: bits[5],
        dadadidi: bits[6],
        houhi: bits[7],
    }
}

fn unpack_events(events: u8) -> Events {
    let bits = events.view_bits::<Lsb0>();

    Events {
        hanasaka: bits[0],
        kintaro: bits[1],
        urashima: bits[2],
        netaro: bits[3],
        murata: bits[4],
        sarukani: bits[5],
        dragon: bits[6],
        hohoemi: bits[7],
    }
}

fn unpack_treasures(treasures: u8) -> Treasures {
    let bits = treasures.view_bits::<Lsb0>();

    Treasures {
        dragon: bits[0],
        fur: bits[1],
        hotoke: bits[2],
        hourai: bits[3],
        swallow: bits[4],
    }
}

fn unpack_minions(minions: u8) -> Minions {
    let bits = minions.view_bits::<Lsb0>();

    Minions {
        dog: bits[0],
        pheasant: bits[1],
        monkey: bits[2],
    }
}

fn unpack_bookmarks(bookmarks: [u8; 2]) -> Bookmarks {
    let bits = bookmarks.view_bits::<Lsb0>();

    Bookmarks {
        tabidachi: bits[0],
        hanasaka: bits[1],
        kintaro: bits[2],
        urashima: bits[3],
        netaro: bits[4],
        kibou: bits[5],
        sarukani: bits[6],
        taketori: bits[7],
        hohoemi: bits[8],
        hien: bits[9],
    }
}

fn deserialize_equipment(bits: &SerializedBitSlice) -> (Equipment, &SerializedBitSlice) {
    let (helm, bits) = deserialize_bits(bits, 2);
    let (weapon, bits) = deserialize_bits(bits, 4);
    let (armor, bits) = deserialize_bits(bits, 4);
    let (shoes, bits) = deserialize_bits(bits, 3);
    let (accessory0, bits) = deserialize_bits(bits, 2);
    let (accessory1, bits) = deserialize_bits(bits, 2);
    let (accessory2, bits) = deserialize_bits(bits, 1);
    let (accessory3, bits) = deserialize_bits(bits, 1);

    let helm = unsafe { HelmIndex::new_unchecked(helm) };
    let weapon = unsafe { WeaponIndex::new_unchecked(weapon) };
    let armor = unsafe { ArmorIndex::new_unchecked(armor) };
    let shoes = unsafe { ShoesIndex::new_unchecked(shoes) };
    let accessory0 = unsafe { Accessory0Index::new_unchecked(accessory0) };
    let accessory1 = unsafe { Accessory1Index::new_unchecked(accessory1) };
    let accessory2 = unsafe { Accessory2Index::new_unchecked(accessory2) };
    let accessory3 = unsafe { Accessory3Index::new_unchecked(accessory3) };

    let equipment = Equipment {
        helm,
        weapon,
        armor,
        shoes,
        accessory0,
        accessory1,
        accessory2,
        accessory3,
    };

    (equipment, bits)
}

fn deserialize_inventory(mut bits: &SerializedBitSlice) -> (Inventory, &SerializedBitSlice) {
    let mut inventory = Inventory::default();

    for _ in 0..8 {
        let item;
        (item, bits) = deserialize_bits(bits, 6);
        if item == 0 {
            break;
        }
        let item = unsafe { ItemId::new_unchecked(item) };
        inventory.push(item);
    }

    (inventory, bits)
}

fn deserialize_bits(bits: &SerializedBitSlice, n: usize) -> (u8, &SerializedBitSlice) {
    debug_assert!(matches!(n, 1..=8));

    let (bits, remain) = bits.split_at(n);
    let value = bits.load_be::<u8>();

    (value, remain)
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    #[test]
    fn test_bytes_new() {
        assert_equal(
            SerializedBytes::new(&[SerializedByte::MIN]).unwrap(),
            [SerializedByte::MIN],
        );
        assert_equal(
            SerializedBytes::new(&[SerializedByte::MIN, SerializedByte::MAX]).unwrap(),
            [SerializedByte::MIN, SerializedByte::MAX],
        );
        assert_equal(
            SerializedBytes::new(&[SerializedByte::MIN; Password::MAX_LEN]).unwrap(),
            [SerializedByte::MIN; Password::MAX_LEN],
        );

        assert_eq!(SerializedBytes::new(&[]), None);
        assert_eq!(
            SerializedBytes::new(&[SerializedByte::MIN; Password::MAX_LEN + 1]),
            None
        );
    }

    #[test]
    fn test_bytes_password_roundtrip() {
        fn f(s: &str) {
            let password_orig = Password::parse(s).unwrap();
            let bytes = SerializedBytes::from_password(&password_orig);
            let password = bytes.to_password();
            assert_eq!(password, password_orig);
        }

        f("あ");
        f("す");
        f("ふ");
        f("おにのばか");
        f("すべてのてきがみたいな");
    }

    #[test]
    fn test_bytes_checksum() {
        fn f(s: &str) -> SerializedBytes {
            SerializedBytes::from_password(&Password::parse(s).unwrap())
        }

        assert!(f("ふ").checksum_is_ok());
        assert!(f("おにのばか").checksum_is_ok());

        assert!(!f("あ").checksum_is_ok());
        assert!(!f("ああああ").checksum_is_ok());
    }

    #[test]
    fn test_load_fu() {
        let bytes = SerializedBytes::from_password(&Password::parse("ふ").unwrap());
        let savedata = bytes.to_savedata().unwrap();

        assert_eq!(
            savedata,
            Savedata {
                xp: 0xFFFF,
                purse: 0xFFFF,
                deposit: Deposit::MAX,
                age: 0xFF,
                age_timer_hi: 0xFF,
                spells: Spells::ALL,
                events: Events::ALL,
                treasures: Treasures::ALL,
                minions: Minions::ALL,
                bookmarks: Bookmarks::ALL,
                respawn: RespawnId::MAX,
                equipment: Equipment {
                    helm: HelmIndex::MAX,
                    weapon: WeaponIndex::MAX,
                    armor: ArmorIndex::MAX,
                    shoes: ShoesIndex::MAX,
                    accessory0: Accessory0Index::MAX,
                    accessory1: Accessory1Index::MAX,
                    accessory2: Accessory2Index::MAX,
                    accessory3: Accessory3Index::MAX,
                },
                inventory: Inventory::from([ItemId::MAX; 8]),
            }
        );
    }

    #[test]
    fn test_load_various_len() {
        assert!(SerializedBytes::from_password(
            &Password::parse("いいいいいいいいいいあえす",).unwrap(),
        )
        .to_savedata()
        .is_some());
        assert!(SerializedBytes::from_password(
            &Password::parse(
                "いいいいいいいいいいいいいいいいいいいいいいいいいいいいいいいいいいいえいた",
            )
            .unwrap(),
        )
        .to_savedata()
        .is_some());
    }

    #[test]
    fn test_save() {
        let bytes = SerializedBytes::from_password(&Password::parse("ふ").unwrap());
        let savedata = bytes.to_savedata().unwrap().normalize();
        let bytes = SerializedBytes::from_savedata(&savedata);
        let password = bytes.to_password();

        // パスワード「ふ」で始めてすぐ神主に話しかけたときのパスワード。
        assert_eq!(
            password.display().to_string(),
            "おしぼひまきびねとしぼひまきびねとひげがけちめいかほがすざ"
        );
    }
}

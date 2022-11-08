use arrayvec::ArrayVec;

use crate::bounded::BoundedU8;
use crate::macros::unreachable_unchecked;

/// パスワードに記録されるゲーム状態。
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Savedata {
    /// 経験値。
    pub xp: u16,
    /// 所持金。
    pub purse: u16,
    /// 預金。
    pub deposit: Deposit,
    /// 年齢。
    pub age: u8,
    /// 加齢タイマー上位バイト。
    pub age_timer_hi: u8,
    /// 術習得状態。
    pub spells: Spells,
    /// イベント進行状態。
    pub events: Events,
    /// 宝物所持状態。
    pub treasures: Treasures,
    /// お供存在状態。
    pub minions: Minions,
    /// ひえんブックマーク。
    pub bookmarks: Bookmarks,
    /// 復活地点ID。
    pub respawn: RespawnId,
    /// 装備。
    pub equipment: Equipment,
    /// インベントリ。
    pub inventory: Inventory,
}

impl Savedata {
    /// このセーブデータを実際にロードした後の状態を返す。
    ///
    /// 装備品のインデックスが不正な場合、装備が変化する。
    pub fn normalize(&self) -> Self {
        Self {
            equipment: self.equipment.normalize(),
            inventory: self.inventory.clone(),
            ..*self
        }
    }
}

/// 預金 (6bit)。
pub type Deposit = BoundedU8<0, 0x3F>;

/// 術習得状態。
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Spells {
    /// きんたん
    pub kintan: bool,
    /// ろっかく
    pub rokkaku: bool,
    /// いなずま
    pub inazuma: bool,
    /// ひえん
    pub hien: bool,
    /// まんきんたん
    pub mankintan: bool,
    /// ふゆう
    pub fuyuu: bool,
    /// だだぢぢ
    pub dadadidi: bool,
    /// ほうひ
    pub houhi: bool,
}

impl Spells {
    /// 術を何も覚えていない状態。
    pub const NONE: Self = Self {
        kintan: false,
        rokkaku: false,
        inazuma: false,
        hien: false,
        mankintan: false,
        fuyuu: false,
        dadadidi: false,
        houhi: false,
    };

    /// 全ての術を覚えた状態。
    pub const ALL: Self = Self {
        kintan: true,
        rokkaku: true,
        inazuma: true,
        hien: true,
        mankintan: true,
        fuyuu: true,
        dadadidi: true,
        houhi: true,
    };
}

/// イベント進行状態。
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Events {
    /// 花咲かの村で銀の鬼を倒した
    pub hanasaka: bool,
    /// 金太郎の村で金の鬼を倒した
    pub kintaro: bool,
    /// 浦島の村でパールの鬼を倒した
    pub urashima: bool,
    /// 寝太郎を起こした
    pub netaro: bool,
    /// 寝太郎の村で村田の情報を聞いた
    pub murata: bool,
    /// やまんばを倒した
    pub sarukani: bool,
    /// 寝太郎の村でリュウのくびかざりを盗まれた
    pub dragon: bool,
    /// 微笑みの村の通行許可を得た
    pub hohoemi: bool,
}

impl Events {
    /// どのイベントも終えていない状態。
    pub const NONE: Self = Self {
        hanasaka: false,
        kintaro: false,
        urashima: false,
        netaro: false,
        murata: false,
        sarukani: false,
        dragon: false,
        hohoemi: false,
    };

    /// 全てのイベントを終えた状態。
    pub const ALL: Self = Self {
        hanasaka: true,
        kintaro: true,
        urashima: true,
        netaro: true,
        murata: true,
        sarukani: true,
        dragon: true,
        hohoemi: true,
    };
}

/// 宝物所持状態。
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Treasures {
    /// リュウのくびかざり
    pub dragon: bool,
    /// キンいろのけがわ
    pub fur: bool,
    /// ホトケのおはち
    pub hotoke: bool,
    /// ホウライのタマ
    pub hourai: bool,
    /// ツバメのこやすがい
    pub swallow: bool,
}

impl Treasures {
    /// どの宝物も持っていない状態。
    pub const NONE: Self = Self {
        dragon: false,
        fur: false,
        hotoke: false,
        hourai: false,
        swallow: false,
    };

    /// 全ての宝物を持っている状態。
    pub const ALL: Self = Self {
        dragon: true,
        fur: true,
        hotoke: true,
        hourai: true,
        swallow: true,
    };
}

/// お供存在状態。
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Minions {
    /// 犬
    pub dog: bool,
    /// キジ
    pub pheasant: bool,
    /// 猿
    pub monkey: bool,
}

impl Minions {
    /// どのお供も連れていない状態。
    pub const NONE: Self = Self {
        dog: false,
        pheasant: false,
        monkey: false,
    };

    /// 全てのお供を連れている状態。
    pub const ALL: Self = Self {
        dog: true,
        pheasant: true,
        monkey: true,
    };
}

/// ひえんブックマーク。
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Bookmarks {
    /// 旅立ちの村
    pub tabidachi: bool,
    /// 花咲かの村
    pub hanasaka: bool,
    /// 金太郎の村
    pub kintaro: bool,
    /// 浦島の村
    pub urashima: bool,
    /// 寝太郎の村
    pub netaro: bool,
    /// 希望の都
    pub kibou: bool,
    /// 猿蟹の村
    pub sarukani: bool,
    /// 竹取の村
    pub taketori: bool,
    /// 微笑みの村
    pub hohoemi: bool,
    /// 飛燕の城
    pub hien: bool,
}

impl Bookmarks {
    /// どの場所もブックマークしていない状態。
    pub const NONE: Self = Self {
        tabidachi: false,
        hanasaka: false,
        kintaro: false,
        urashima: false,
        netaro: false,
        kibou: false,
        sarukani: false,
        taketori: false,
        hohoemi: false,
        hien: false,
    };

    /// 全ての場所をブックマークした状態。
    pub const ALL: Self = Self {
        tabidachi: true,
        hanasaka: true,
        kintaro: true,
        urashima: true,
        netaro: true,
        kibou: true,
        sarukani: true,
        taketori: true,
        hohoemi: true,
        hien: true,
    };
}

/// 復活地点ID (4bit)。
pub type RespawnId = BoundedU8<0, 0xF>;

/// 装備。
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Equipment {
    pub helm: HelmIndex,
    pub weapon: WeaponIndex,
    pub armor: ArmorIndex,
    pub shoes: ShoesIndex,
    pub accessory0: Accessory0Index,
    pub accessory1: Accessory1Index,
    pub accessory2: Accessory2Index,
    pub accessory3: Accessory3Index,
}

impl Equipment {
    /// このセーブデータ内装備を実際にロードした後の装備を返す。
    ///
    /// 装備品のインデックスが不正な場合、装備が変化する。
    pub fn normalize(&self) -> Self {
        let mut res = Self::default();

        match self.helm.get() {
            0..=2 => res.helm = self.helm,
            3 => {}
            4.. => unsafe { unreachable_unchecked!() },
        }
        match self.weapon.get() {
            0..=10 => res.weapon = self.weapon,
            11..=12 => {}
            x @ 13..=15 => res.armor = unsafe { ArmorIndex::new_unchecked(x - 12) },
            16.. => unsafe { unreachable_unchecked!() },
        }
        match self.armor.get() {
            0..=9 => res.armor = self.armor,
            10..=11 => {}
            x @ 12..=15 => res.shoes = unsafe { ShoesIndex::new_unchecked(x - 11) },
            16.. => unsafe { unreachable_unchecked!() },
        }
        match self.shoes.get() {
            0..=4 => res.shoes = self.shoes,
            5..=6 => {}
            7 => res.accessory0 = unsafe { Accessory0Index::new_unchecked(1) },
            8.. => unsafe { unreachable_unchecked!() },
        }
        match self.accessory0.get() {
            0..=2 => res.accessory0 = self.accessory0,
            3 => {}
            4.. => unsafe { unreachable_unchecked!() },
        }
        match self.accessory1.get() {
            0..=2 => res.accessory1 = self.accessory1,
            3 => {}
            4.. => unsafe { unreachable_unchecked!() },
        }
        res.accessory2 = self.accessory2;
        res.accessory3 = self.accessory3;

        res
    }
}

/// 兜インデックス (2bit)。
pub type HelmIndex = BoundedU8<0, 3>;

/// 武器インデックス (4bit)。
pub type WeaponIndex = BoundedU8<0, 0xF>;

/// 鎧インデックス (4bit)。
pub type ArmorIndex = BoundedU8<0, 0xF>;

/// 靴インデックス (3bit)。
pub type ShoesIndex = BoundedU8<0, 7>;

/// いでたち0インデックス (2bit)。
pub type Accessory0Index = BoundedU8<0, 3>;

/// いでたち1インデックス (2bit)。
pub type Accessory1Index = BoundedU8<0, 3>;

/// いでたち2インデックス (1bit)。
pub type Accessory2Index = BoundedU8<0, 1>;

/// いでたち3インデックス (1bit)。
pub type Accessory3Index = BoundedU8<0, 1>;

/// インベントリ。
pub type Inventory = ArrayVec<ItemId, 8>;

/// アイテムID (nonzero, 6bit)。
pub type ItemId = BoundedU8<1, 0x3F>;

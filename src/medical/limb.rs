// limb.rs
use bevy::prelude::*;

// ── ТИПЫ КОНЕЧНОСТЕЙ ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LimbType {
    // Туловище — неотделимые, всегда есть
    Thorax,
    Abdomen,

    // Ноги — опциональные
    LeftThigh,
    LeftCrus,
    LeftFoot,
    RightThigh,
    RightCrus,
    RightFoot,

    // Руки — опциональные
    LeftShoulder,
    LeftForearm,
    LeftHand,
    RightShoulder,
    RightForearm,
    RightHand,

    // Голова — опциональная (для роботов без головы например)
    Head,
}

impl LimbType {
    // Родительская конечность — для распространения урона вверх по цепочке
    pub fn parent(&self) -> Option<LimbType> {
        match self {
            LimbType::LeftCrus      => Some(LimbType::LeftThigh),
            LimbType::LeftFoot      => Some(LimbType::LeftCrus),
            LimbType::RightCrus     => Some(LimbType::RightThigh),
            LimbType::RightFoot     => Some(LimbType::RightCrus),
            LimbType::LeftForearm   => Some(LimbType::LeftShoulder),
            LimbType::LeftHand      => Some(LimbType::LeftForearm),
            LimbType::RightForearm  => Some(LimbType::RightShoulder),
            LimbType::RightHand     => Some(LimbType::RightForearm),
            _ => None,
        }
    }

    // Есть ли костяная защита органов (грудная клетка, череп)
    pub fn has_bone_organ_shield(&self) -> bool {
        matches!(self, LimbType::Head | LimbType::Thorax)
    }
}

// ── БАЗОВЫЙ КОМПОНЕНТ КОНЕЧНОСТИ ────────────────────────────

#[derive(Component, Debug)]
pub struct Limb {
    pub limb_type:  LimbType,
    pub skin_max:   u16,
    pub muscle_max: u16,
    pub bone_max:   u16,
}

impl Limb {
    pub fn thorax() -> Self {
        Self { limb_type: LimbType::Thorax,   skin_max: 10000, muscle_max: 40000, bone_max: 30000 }
    }
    pub fn abdomen() -> Self {
        Self { limb_type: LimbType::Abdomen,  skin_max: 10000, muscle_max: 35000, bone_max: 20000 }
    }
    pub fn head() -> Self {
        Self { limb_type: LimbType::Head,     skin_max: 5000,  muscle_max: 10000, bone_max: 15000 }
    }
    pub fn thigh() -> Self {
        Self { limb_type: LimbType::LeftThigh, skin_max: 8000, muscle_max: 30000, bone_max: 25000 }
    }
    pub fn shoulder() -> Self {
        Self { limb_type: LimbType::LeftShoulder, skin_max: 6000, muscle_max: 20000, bone_max: 18000 }
    }
    // добавляй остальные по аналогии
}

// ── СЛОИ УРОНА ──────────────────────────────────────────────
// Отсутствие компонента = слой полностью здоров

#[derive(Component, Debug, Default)]
pub struct SkinLayer {
    pub slash_damage: u16,  // лечится бинтами
    pub burn_damage:  u16,  // лечится мазью
    pub acid_damage:  u16,  // лечится нейтрализатором
    pub frost_damage: u16,  // лечится теплом
}

impl SkinLayer {
    pub fn total(&self) -> u16 {
        self.slash_damage
            .saturating_add(self.burn_damage)
            .saturating_add(self.acid_damage)
            .saturating_add(self.frost_damage)
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

#[derive(Component, Debug, Default)]
pub struct MuscleLayer {
    pub blunt_damage:  u16,  // лечится покоем
    pub slash_damage:  u16,  // лечится швами
    pub pierce_damage: u16,  // лечится швами
    pub burn_damage:   u16,  // лечится мазью
    pub frost_damage:  u16,  // лечится теплом
}

impl MuscleLayer {
    pub fn total(&self) -> u16 {
        self.blunt_damage
            .saturating_add(self.slash_damage)
            .saturating_add(self.pierce_damage)
            .saturating_add(self.burn_damage)
            .saturating_add(self.frost_damage)
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

// Кость как u8 0-100 — плавный прогресс повреждения
#[derive(Component, Debug)]
pub struct BoneLayer(pub u8);

impl BoneLayer {
    pub fn new() -> Self { Self(0) }

    pub fn add_damage(&mut self, amount: u8) {
        self.0 = self.0.saturating_add(amount).min(100);
    }

    // Пороги состояния
    pub fn is_healthy(&self)   -> bool { self.0 == 0 }
    pub fn has_crack(&self)    -> bool { self.0 > 0  && self.0 < 50 }
    pub fn is_fractured(&self) -> bool { self.0 >= 50 && self.0 < 100 }
    pub fn is_shattered(&self) -> bool { self.0 >= 100 }

    // Штраф к функциональности конечности 0.0-1.0
    pub fn function_penalty(&self) -> f32 {
        self.0 as f32 / 100.0
    }
}

// ── ОСТАЛЬНЫЕ СОСТОЯНИЯ ─────────────────────────────────────

#[derive(Component, Debug)]
pub struct Bleeding(pub u16);

#[derive(Component, Debug)]
pub struct Frostbite {
    pub severity: u8,  // 0-100
}

#[derive(Component, Debug)]
pub struct BurnDamage {
    pub severity: u8,  // 0-100
}

#[derive(Component, Debug)]
pub struct OrganDamage {
    pub name:       String,
    pub damage:     u16,
    pub max_damage: u16,
    pub is_heart:   bool,
}
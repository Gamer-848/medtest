// limb.rs
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side { Left, Right }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LimbKind {
    Head, Thorax, Abdomen,
    Thigh(Side), Crus(Side), Foot(Side),
    Shoulder(Side), Forearm(Side), Hand(Side),
}

impl LimbKind {
    pub fn parent(&self) -> Option<LimbKind> {
        match self {
            LimbKind::Crus(s)    => Some(LimbKind::Thigh(*s)),
            LimbKind::Foot(s)    => Some(LimbKind::Crus(*s)),
            LimbKind::Forearm(s) => Some(LimbKind::Shoulder(*s)),
            LimbKind::Hand(s)    => Some(LimbKind::Forearm(*s)),
            _ => None,
        }
    }

    pub fn has_bone_organ_shield(&self) -> bool {
        matches!(self, LimbKind::Head | LimbKind::Thorax)
    }

    pub fn is_paired(&self) -> bool {
        !matches!(self, LimbKind::Head | LimbKind::Thorax | LimbKind::Abdomen)
    }

    pub fn default_stats(&self) -> LimbStats {
        match self {
            LimbKind::Head        => LimbStats { skin_max: 5000,  muscle_max: 10000, bone_max: 15000 },
            LimbKind::Thorax      => LimbStats { skin_max: 10000, muscle_max: 40000, bone_max: 30000 },
            LimbKind::Abdomen     => LimbStats { skin_max: 10000, muscle_max: 35000, bone_max: 20000 },
            LimbKind::Thigh(_)    => LimbStats { skin_max: 8000,  muscle_max: 30000, bone_max: 25000 },
            LimbKind::Crus(_)     => LimbStats { skin_max: 6000,  muscle_max: 20000, bone_max: 20000 },
            LimbKind::Foot(_)     => LimbStats { skin_max: 3000,  muscle_max: 8000,  bone_max: 10000 },
            LimbKind::Shoulder(_) => LimbStats { skin_max: 6000,  muscle_max: 20000, bone_max: 18000 },
            LimbKind::Forearm(_)  => LimbStats { skin_max: 4000,  muscle_max: 12000, bone_max: 12000 },
            LimbKind::Hand(_)     => LimbStats { skin_max: 2000,  muscle_max: 5000,  bone_max: 8000  },
        }
    }
}

pub struct LimbStats {
    pub skin_max:   u16,
    pub muscle_max: u16,
    pub bone_max:   u16,
}

#[derive(Component, Debug)]
pub struct Limb {
    pub kind:       LimbKind,
    pub skin_max:   u16,
    pub muscle_max: u16,
    pub bone_max:   u16,
}

impl Limb {
    pub fn new(kind: LimbKind) -> Self {
        let stats = kind.default_stats();
        Self { kind, skin_max: stats.skin_max, muscle_max: stats.muscle_max, bone_max: stats.bone_max }
    }
}

// Маркер — может быть заражено биологической инфекцией
// Кибернетика без этого маркера не заражается
#[derive(Component, Debug)]
pub struct Infectable;

#[derive(Component, Debug, Default)]
pub struct SkinLayer {
    pub slash_damage: u16,
    pub burn_damage:  u16,
    pub acid_damage:  u16,
    pub frost_damage: u16,
}

impl SkinLayer {
    pub fn total(&self) -> u16 {
        self.slash_damage.saturating_add(self.burn_damage)
            .saturating_add(self.acid_damage).saturating_add(self.frost_damage)
    }
    pub fn is_empty(&self) -> bool { self.total() == 0 }
}

#[derive(Component, Debug, Default)]
pub struct MuscleLayer {
    pub blunt_damage:  u16,
    pub slash_damage:  u16,
    pub pierce_damage: u16,
    pub burn_damage:   u16,
    pub frost_damage:  u16,
}

impl MuscleLayer {
    pub fn total(&self) -> u16 {
        self.blunt_damage.saturating_add(self.slash_damage)
            .saturating_add(self.pierce_damage)
            .saturating_add(self.burn_damage).saturating_add(self.frost_damage)
    }
    pub fn is_empty(&self) -> bool { self.total() == 0 }
}

#[derive(Component, Debug)]
pub struct BoneLayer(pub u8);

impl BoneLayer {
    pub fn new() -> Self { Self(0) }
    pub fn add_damage(&mut self, amount: u8) { self.0 = self.0.saturating_add(amount).min(100); }
    pub fn is_healthy(&self)   -> bool { self.0 == 0 }
    pub fn has_crack(&self)    -> bool { self.0 > 0   && self.0 < 50  }
    pub fn is_fractured(&self) -> bool { self.0 >= 50 && self.0 < 100 }
    pub fn is_shattered(&self) -> bool { self.0 >= 100 }
    pub fn function_penalty(&self) -> f32 { self.0 as f32 / 100.0 }
}

#[derive(Component, Debug)]
pub struct Bleeding(pub u16);

#[derive(Component, Debug)]
pub struct Frostbite { pub severity: u8 }

#[derive(Component, Debug)]
pub struct BurnDamage { pub severity: u8 }

#[derive(Component, Debug)]
pub struct Infection { pub severity: u16 }
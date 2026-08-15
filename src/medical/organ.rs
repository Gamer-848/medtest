// organ.rs
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganType {
    Heart, Lungs, Liver, Kidney, Stomach, Brain, Spleen,
}

impl OrganType {
    pub fn max_damage(&self) -> u16 {
        match self {
            OrganType::Heart   => 40000,
            OrganType::Brain   => 30000,
            OrganType::Lungs   => 30000,
            OrganType::Liver   => 25000,
            OrganType::Kidney  => 20000,
            OrganType::Spleen  => 15000,
            OrganType::Stomach => 15000,
        }
    }

    pub fn is_critical(&self) -> bool {
        matches!(self, OrganType::Heart | OrganType::Brain)
    }

    pub fn default_limb(&self) -> super::limb::LimbKind {
        use super::limb::LimbKind;
        match self {
            OrganType::Heart | OrganType::Lungs => LimbKind::Thorax,
            OrganType::Brain                    => LimbKind::Head,
            _                                   => LimbKind::Abdomen,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            OrganType::Heart   => "Сердце",
            OrganType::Brain   => "Мозг",
            OrganType::Lungs   => "Лёгкие",
            OrganType::Liver   => "Печень",
            OrganType::Kidney  => "Почка",
            OrganType::Spleen  => "Селезёнка",
            OrganType::Stomach => "Желудок",
        }
    }
}

#[derive(Component, Debug)]
pub struct Organ {
    pub organ_type: OrganType,
    pub damage:     u16,
}

impl Organ {
    pub fn new(organ_type: OrganType) -> Self {
        Self { organ_type, damage: 0 }
    }

    pub fn max_damage(&self) -> u16       { self.organ_type.max_damage() }
    pub fn is_critical(&self) -> bool     { self.organ_type.is_critical() }
    pub fn name(&self) -> &'static str    { self.organ_type.name() }
    pub fn is_destroyed(&self) -> bool    { self.damage >= self.max_damage() }

    pub fn health_pct(&self) -> f32 {
        1.0 - (self.damage as f32 / self.max_damage() as f32)
    }

    // Функциональность 0-100
    pub fn function_pct(&self) -> u8 {
        let pct = self.health_pct();
        if pct > 0.5 {
            100
        } else {
            ((pct * 2.0).powf(2.0) * 100.0) as u8
        }
    }

    pub fn add_damage(&mut self, amount: u16) {
        self.damage = self.damage.saturating_add(amount).min(self.max_damage());
    }
}

// ── ЛЁГКИЕ ──────────────────────────────────────────────────

#[derive(Component, Debug)]
pub struct Lungs {
    pub stamina:          u8,  // 0-100, текущий запас
    pub respiratory_rate: u8,  // вдохов/мин, норма 16
    pub max_stamina:      u8,  // снижается от урона
}

impl Lungs {
    pub fn new() -> Self {
        Self { stamina: 100, respiratory_rate: 16, max_stamina: 100 }
    }

    pub fn tick_recovery(&mut self) {
        let recovery = (self.respiratory_rate / 32).max(1);
        self.stamina = self.stamina.saturating_add(recovery).min(self.max_stamina);
    }

    pub fn consume(&mut self, amount: u8) {
        self.stamina = self.stamina.saturating_sub(amount);
    }

    pub fn function_pct(&self) -> u8 {
        if self.max_stamina == 0 { return 0; }
        (self.stamina as u16 * 100 / self.max_stamina as u16) as u8
    }
}

// ── СЕРДЦЕ ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeartRhythm {
    Normal,
    Tachycardia,
    Bradycardia,
    Fibrillation,
    Arrest,
}

#[derive(Debug, Clone, Copy)]
pub enum HeartType {
    Human,
    Augmented,
    Animal,
    Synthetic,
}

// Пресет характеристик сердца — задаётся при создании
// кастомные сердца (импланты, мутации) создаются с другими значениями
#[derive(Debug, Clone, Copy)]
pub struct HeartStats {
    pub max_bpm:        u16,  // абсолютный максимум
    pub safe_bpm_pct:   u8,   // % от max_bpm до которого безопасно (0-100)
    pub regen_peak_bpm: u8,   // bpm при котором реген максимален
    pub fib_resistance: u8,   // сопротивляемость фибрилляции 0-100
}

impl HeartStats {
    pub fn human() -> Self {
        Self { max_bpm: 180, safe_bpm_pct: 75, regen_peak_bpm: 65, fib_resistance: 50 }
    }
    pub fn augmented() -> Self {
        Self { max_bpm: 220, safe_bpm_pct: 80, regen_peak_bpm: 80, fib_resistance: 80 }
    }
    pub fn animal() -> Self {
        Self { max_bpm: 200, safe_bpm_pct: 78, regen_peak_bpm: 100, fib_resistance: 60 }
    }
    pub fn synthetic() -> Self {
        Self { max_bpm: 300, safe_bpm_pct: 95, regen_peak_bpm: 120, fib_resistance: 100 }
    }

    // Безопасный лимит bpm с учётом повреждений 0-100%
    pub fn safe_bpm(&self, damage_pct: u8) -> u16 {
        let base = self.max_bpm * self.safe_bpm_pct as u16 / 100;
        // при 100% урона лимит падает до 60 bpm
        let floor = 60u16;
        let reduction = (base - floor) * damage_pct as u16 / 100;
        base - reduction
    }

    // Эффективность регена 0-100 для данного bpm
    pub fn regen_efficiency(&self, bpm: u16) -> u8 {
        let peak = self.regen_peak_bpm as f32;
        let width = 25.0f32;
        let b = bpm as f32;
        let eff = (-(b - peak).powi(2) / (2.0 * width.powi(2))).exp();
        (eff * 100.0) as u8
    }
}

#[derive(Component, Debug)]
pub struct Heart {
    pub stats:             HeartStats,
    pub bpm:               u16,
    pub target_bpm:        u16,
    pub stroke_volume:     u8,    // мл за удар, норма 70
    pub rhythm:            HeartRhythm,
    pub fibrillation_risk: u8,    // 0-100
}

impl Heart {
    pub fn new(stats: HeartStats) -> Self {
        Self {
            stats,
            bpm:               70,
            target_bpm:        70,
            stroke_volume:     70,
            rhythm:            HeartRhythm::Normal,
            fibrillation_risk: 0,
        }
    }

    pub fn human() -> Self { Self::new(HeartStats::human()) }

    // cardiac output в мл/мин
    pub fn cardiac_output(&self) -> u32 {
        self.bpm as u32 * self.stroke_volume as u32
    }

    pub fn regen_efficiency(&self) -> u8 {
        if matches!(self.rhythm, HeartRhythm::Arrest | HeartRhythm::Fibrillation) {
            return 0;
        }
        self.stats.regen_efficiency(self.bpm)
    }

    pub fn update_rhythm(&mut self) {
        self.rhythm = match self.bpm {
            0        => HeartRhythm::Arrest,
            1..=59   => HeartRhythm::Bradycardia,
            60..=100 => HeartRhythm::Normal,
            _        => HeartRhythm::Tachycardia,
        };
    }

    pub fn tick(&mut self, heart_damage_pct: u8, rng: &mut impl rand::RngExt) {
        // Плавно движемся к цели
        if self.bpm < self.target_bpm {
            self.bpm = self.bpm.saturating_add(3).min(self.target_bpm);
        } else if self.bpm > self.target_bpm {
            self.bpm = self.bpm.saturating_sub(3).max(self.target_bpm);
        }

        // Зоны риска фибрилляции
        let safe_limit = self.stats.safe_bpm(heart_damage_pct);
        if self.bpm > safe_limit {
            let over = self.bpm - safe_limit;
            let risk_gain: u8 = if over < 20 { 1 } else { 5 };
            // fib_resistance снижает накопление риска
            let actual_gain = risk_gain.saturating_sub(self.stats.fib_resistance / 20);
            self.fibrillation_risk = self.fibrillation_risk.saturating_add(actual_gain);
        } else {
            self.fibrillation_risk = self.fibrillation_risk.saturating_sub(1);
        }

        // Бросок на фибрилляцию
        if self.fibrillation_risk > 10 {
            let threshold = self.fibrillation_risk as u32 * 10
                / (self.stats.fib_resistance as u32 + 1);
            if rng.random_range(0..1000u32) < threshold {
                self.rhythm = HeartRhythm::Fibrillation;
                self.bpm = 0;
                println!("💔 ФИБРИЛЛЯЦИЯ!");
                return;
            }
        }

        self.update_rhythm();
    }
}
// damage.rs
use bevy::prelude::*;
use rand::RngExt;
use super::limb::*;
use super::organ::Organ;  // ← добавили

pub enum DamageComponent {
    Blunt(u16),
    Slashing(u16),
    Piercing(u16),
    Burn(u16),
    Acid(u16),
    Frost(u16),
}

pub struct DamageHit {
    pub components: Vec<DamageComponent>,
}

impl DamageHit {
    pub fn new() -> Self { Self { components: Vec::new() } }

    pub fn with(mut self, c: DamageComponent) -> Self {
        self.components.push(c);
        self
    }

    pub fn punch(val: u16)  -> Self { Self::new().with(DamageComponent::Blunt(val)) }
    pub fn slash(val: u16)  -> Self { Self::new().with(DamageComponent::Slashing(val)) }
    pub fn bullet(val: u16) -> Self {
        Self::new()
            .with(DamageComponent::Piercing(val))
            .with(DamageComponent::Burn(val / 20))
    }
    pub fn claw(blunt: u16, slash: u16) -> Self {
        Self::new()
            .with(DamageComponent::Blunt(blunt))
            .with(DamageComponent::Slashing(slash))
    }
}

pub struct LimbContext<'a> {
    pub entity:  Entity,
    pub limb:    &'a Limb,
    pub skin:    Option<&'a mut SkinLayer>,
    pub muscle:  Option<&'a mut MuscleLayer>,
    pub bone:    Option<&'a mut BoneLayer>,
    pub bleed:   Option<&'a mut Bleeding>,
    pub frost:   Option<&'a mut Frostbite>,
    pub burn:    Option<&'a mut BurnDamage>,
    pub organ:   Option<&'a mut Organ>,  // ← было OrganDamage
}

pub fn apply_damage(commands: &mut Commands, mut ctx: LimbContext, hit: DamageHit) {
    let mut rng = rand::rng();

    let skin_total   = ctx.skin.as_ref().map(|s| s.total()).unwrap_or(0);
    let muscle_total = ctx.muscle.as_ref().map(|m| m.total()).unwrap_or(0);
    let bone_damage  = ctx.bone.as_ref().map(|b| b.0).unwrap_or(0);
    let frost_sev    = ctx.frost.as_ref().map(|f| f.severity).unwrap_or(0);
    let skin_gone    = skin_total   >= ctx.limb.skin_max;
    let bone_fraction = bone_damage;

    let mut skin_slash:   u16 = 0;
    let mut skin_burn:    u16 = 0;
    let mut skin_acid:    u16 = 0;
    let mut skin_frost:   u16 = 0;
    let mut muscle_blunt:  u16 = 0;
    let mut muscle_slash:  u16 = 0;
    let mut muscle_pierce: u16 = 0;
    let mut muscle_burn:   u16 = 0;
    let mut muscle_frost:  u16 = 0;
    let mut bone_delta:  u8  = 0;
    let mut bleed_delta: u16 = 0;

    for component in &hit.components {
        match component {

            DamageComponent::Blunt(val) => {
                let bonus = if skin_gone { *val / 5 } else { 0 };
                muscle_blunt += val + bonus;

                let muscle_buffer  = ctx.limb.muscle_max.saturating_sub(muscle_total);
                let bone_threshold = if frost_sev > 50 { 1500u16 } else { 3000u16 };
                if *val > bone_threshold && (*val > muscle_buffer || frost_sev > 50) {
                    let bone_dmg = ((*val - bone_threshold) / 100).min(50) as u8;
                    bone_delta = bone_delta.saturating_add(bone_dmg);
                }
            }

            DamageComponent::Slashing(val) => {
                let to_skin = if *val < 500 { *val } else { 500 + (*val - 500) / 4 };
                let to_muscle = val.saturating_sub(to_skin);

                if skin_gone {
                    muscle_slash += val;
                } else {
                    skin_slash   += to_skin;
                    muscle_slash += to_muscle;
                }
                bleed_delta += val / 8;
            }

            DamageComponent::Piercing(val) => {
                if skin_gone {
                    muscle_pierce += val / 3;
                } else {
                    skin_slash    += val / 6;
                    muscle_pierce += val / 4;
                }
                bleed_delta += val / 12;

                let bone_chance = if frost_sev > 50 { 30u32 } else { 10u32 };
                if *val > 500 && rng.random_range(0..100u32) < bone_chance {
                    bone_delta = bone_delta.saturating_add((*val / 200).min(30) as u8);
                    println!("BONE_PIERCED");
                }

                if let Some(ref mut organ) = ctx.organ {
                    let organ_chance: u32 = if bone_fraction >= 50 { 60 } else { 20 };
                    if rng.random_range(0..100u32) < organ_chance {
                        let organ_dmg = if bone_fraction >= 50 { val / 2 } else { val / 5 };
                        organ.add_damage(organ_dmg);  // ← используем метод из Organ
                        println!("ORGAN_PIERCED: -{}", organ_dmg);
                    }
                }
            }

            DamageComponent::Burn(val) => {
                if skin_gone { muscle_burn += val; }
                else {
                    skin_burn += val;
                    if *val > 1000 { muscle_burn += val / 3; }
                }

                let sev_gain = (*val / 100).min(100) as u8;
                if let Some(ref mut burn) = ctx.burn {
                    burn.severity = burn.severity.saturating_add(sev_gain).min(100);
                } else {
                    commands.entity(ctx.entity).insert(BurnDamage { severity: sev_gain });
                }
            }

            DamageComponent::Acid(val) => {
                if skin_gone { muscle_slash += val; }
                else { skin_acid += val; }

                if *val > 3000 {
                    bone_delta = bone_delta.saturating_add(40);
                    println!("BONE_CHEMICAL_NECROSIS");
                }
            }

            DamageComponent::Frost(val) => {
                if skin_gone { muscle_frost += val / 2; }
                else { skin_frost += val / 2; }

                let sev_gain = (*val / 50).min(100) as u8;
                if let Some(ref mut frost) = ctx.frost {
                    frost.severity = frost.severity.saturating_add(sev_gain).min(100);
                } else {
                    commands.entity(ctx.entity).insert(Frostbite { severity: sev_gain });
                }

                if *val > 2000 { muscle_frost += val / 4; }
            }
        }
    }

    let any_skin = skin_slash > 0 || skin_burn > 0 || skin_acid > 0 || skin_frost > 0;
    if any_skin {
        if let Some(skin) = ctx.skin {
            skin.slash_damage = skin.slash_damage.saturating_add(skin_slash).min(ctx.limb.skin_max);
            skin.burn_damage  = skin.burn_damage .saturating_add(skin_burn) .min(ctx.limb.skin_max);
            skin.acid_damage  = skin.acid_damage .saturating_add(skin_acid) .min(ctx.limb.skin_max);
            skin.frost_damage = skin.frost_damage.saturating_add(skin_frost).min(ctx.limb.skin_max);
        } else {
            commands.entity(ctx.entity).insert(SkinLayer {
                slash_damage: skin_slash.min(ctx.limb.skin_max),
                burn_damage:  skin_burn .min(ctx.limb.skin_max),
                acid_damage:  skin_acid .min(ctx.limb.skin_max),
                frost_damage: skin_frost.min(ctx.limb.skin_max),
            });
        }
    }

    let any_muscle = muscle_blunt > 0 || muscle_slash > 0 || muscle_pierce > 0
                  || muscle_burn  > 0 || muscle_frost > 0;
    if any_muscle {
        if let Some(muscle) = ctx.muscle {
            muscle.blunt_damage  = muscle.blunt_damage .saturating_add(muscle_blunt) .min(ctx.limb.muscle_max);
            muscle.slash_damage  = muscle.slash_damage .saturating_add(muscle_slash) .min(ctx.limb.muscle_max);
            muscle.pierce_damage = muscle.pierce_damage.saturating_add(muscle_pierce).min(ctx.limb.muscle_max);
            muscle.burn_damage   = muscle.burn_damage  .saturating_add(muscle_burn)  .min(ctx.limb.muscle_max);
            muscle.frost_damage  = muscle.frost_damage .saturating_add(muscle_frost) .min(ctx.limb.muscle_max);
        } else {
            commands.entity(ctx.entity).insert(MuscleLayer {
                blunt_damage:  muscle_blunt .min(ctx.limb.muscle_max),
                slash_damage:  muscle_slash .min(ctx.limb.muscle_max),
                pierce_damage: muscle_pierce.min(ctx.limb.muscle_max),
                burn_damage:   muscle_burn  .min(ctx.limb.muscle_max),
                frost_damage:  muscle_frost .min(ctx.limb.muscle_max),
            });
        }
    }

    if bone_delta > 0 {
        if let Some(bone) = ctx.bone {
            bone.add_damage(bone_delta);
            println!("BONE: {}%", bone.0);
        } else {
            let mut b = BoneLayer::new();
            b.add_damage(bone_delta);
            commands.entity(ctx.entity).insert(b);
        }
    }

    if bleed_delta > 0 {
        if let Some(bleed) = ctx.bleed {
            bleed.0 = bleed.0.saturating_add(bleed_delta);
        } else {
            commands.entity(ctx.entity).insert(Bleeding(bleed_delta));
        }
        println!("BLEEDING: {}", bleed_delta);
    }
}
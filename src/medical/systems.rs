// systems.rs
use bevy::prelude::*;
use rand::RngExt;
use super::body::{Body, InternalBleeding, Hemothorax};
use super::limb::{Limb, SkinLayer, MuscleLayer, BoneLayer,
                  Bleeding, Frostbite, BurnDamage, Infection, Infectable};
use super::organ::{Organ, OrganType, Heart, HeartRhythm, Lungs};

const INF_SKIN_BLOCK:   u16 = 19660;
const INF_MUSCLE_BLOCK: u16 = 39321;
const INF_SPREAD:       u16 = 52428;
const INF_GANGRENE:     u16 = 65535;

pub fn for_each_limb_child<F>(
    body_children: &Children,
    limb_query: &Query<&Children, With<Limb>>,
    mut f: F,
) where F: FnMut(Entity) {
    for limb_entity in body_children.iter() {
        if let Ok(limb_children) = limb_query.get(limb_entity) {
            for child in limb_children.iter() {
                f(child);
            }
        }
    }
}

pub fn heart_system(
    mut body_query:  Query<(&mut Body, &Children)>,
    limb_query:      Query<&Children, With<Limb>>,
    mut heart_query: Query<&mut Heart>,
    organ_query:     Query<&Organ>,
) {
    let mut rng = rand::rng();

    for (mut body, body_children) in body_query.iter_mut() {
        if !body.is_alive { continue; }

        let mut heart_entity     = None;
        let mut heart_damage_pct = 0u8;

        for_each_limb_child(body_children, &limb_query, |child| {
            if let Ok(organ) = organ_query.get(child) {
                if organ.organ_type == OrganType::Heart {
                    heart_damage_pct = ((1.0 - organ.health_pct()) * 100.0) as u8;
                }
            }
            if heart_query.get(child).is_ok() {
                heart_entity = Some(child);
            }
        });

        let Some(h_entity) = heart_entity else { continue };
        let Ok(mut heart) = heart_query.get_mut(h_entity) else { continue };

        let blood_ratio = (body.blood_volume as u32 * 100 / 5000).min(100) as u8;
        heart.stroke_volume = (70u32 * blood_ratio as u32 / 100).min(70) as u8;

        let co = heart.cardiac_output();
        body.blood_pressure = (co / 49).min(255) as u8;

        if body.blood_pressure < 100 {
            let pressure_deficit  = 100u32 - body.blood_pressure as u32;
            let viscosity_penalty = (body.viscosity as u32).saturating_sub(100) / 10;
            heart.target_bpm = (70 + (pressure_deficit + viscosity_penalty) as u16 * 4 / 5)
                .min(heart.stats.max_bpm);
        } else {
            heart.target_bpm = 70;
        }

        if body.body_temp < 36 {
            let cold = (36u16 - body.body_temp as u16) * 3;
            heart.target_bpm = heart.target_bpm.saturating_sub(cold);
        } else if body.body_temp > 38 {
            let heat = (body.body_temp as u16 - 38) * 5;
            heart.target_bpm = (heart.target_bpm + heat).min(heart.stats.max_bpm);
        }

        heart.tick(heart_damage_pct, &mut rng);

        if matches!(heart.rhythm, HeartRhythm::Fibrillation | HeartRhythm::Arrest) {
            body.blood_pressure = 0;
        }

        println!("❤️  [{}] bpm={} stroke={}мл давление={} риск_фиб={}%",
            body.name, heart.bpm, heart.stroke_volume,
            body.blood_pressure, heart.fibrillation_risk);
    }
}

pub fn oxygen_system(
    mut body_query:  Query<(&mut Body, &Children)>,
    limb_query:      Query<&Children, With<Limb>>,
    organ_query:     Query<&Organ>,
    mut lungs_query: Query<&mut Lungs>,
    hemothorax_q:    Query<&Hemothorax>,
) {
    for (mut body, body_children) in body_query.iter_mut() {
        if !body.is_alive { continue; }

        let mut lung_fn:       u32 = 100;
        let mut lung_dmg:      u32 = 0;
        let mut hemo_compress: u32 = 0;

        for limb_entity in body_children.iter() {
            if let Ok(hemo) = hemothorax_q.get(limb_entity) {
                hemo_compress = hemo.lung_compression() as u32;
            }
        }

        for_each_limb_child(body_children, &limb_query, |child| {
            if let Ok(organ) = organ_query.get(child) {
                if organ.organ_type == OrganType::Lungs {
                    lung_dmg = ((1.0 - organ.health_pct()) * 100.0) as u32;
                }
            }
            if let Ok(mut lungs) = lungs_query.get_mut(child) {
                lungs.tick_recovery();
                lung_fn = lungs.function_pct() as u32;
            }
        });

        let eff_lung = (lung_fn
            * (100 - lung_dmg) / 100
            * (100 - hemo_compress * 80 / 100) / 100
        ).min(100);

        let perfusion_factor = match body.blood_pressure {
            p if p >= 80 => 100u32,
            p if p >= 50 => p as u32 * 100 / 80,
            p if p >= 30 => p as u32 * 80 / 50,
            p            => p as u32 * 50 / 30,
        };

        let raw_spo2 = (eff_lung * perfusion_factor / 100).min(98) as u8;

        if raw_spo2 < body.spo2 {
            body.spo2 = body.spo2.saturating_sub(2).max(raw_spo2);
        } else {
            body.spo2 = body.spo2.saturating_add(1).min(raw_spo2);
        }

        if body.spo2 < 92 {
            let deficit = 92u32 - body.spo2 as u32;
            body.hypoxia = body.hypoxia.saturating_add((deficit / 3 + 1).min(20) as u8).min(100);
        } else {
            body.hypoxia = body.hypoxia.saturating_sub(3);
        }

        if body.spo2 < 92 {
            println!("⚠️  [{}] SpO2: {}% | Перфузия: {}% | Лёгкие: {}%",
                body.name, body.spo2, perfusion_factor, eff_lung);
        }
        if body.hypoxia > 20 {
            println!("🫁 [{}] Гипоксия: {}%", body.name, body.hypoxia);
        }
    }
}

pub fn temperature_system(
    mut body_query:  Query<(&mut Body, &Children)>,
    limb_query:      Query<&Children, With<Limb>>,
    frostbite_q:     Query<&Frostbite>,
    burn_q:          Query<&BurnDamage>,
    infection_q:     Query<&Infection>,
    mut organ_query: Query<&mut Organ>,
) {
    for (mut body, body_children) in body_query.iter_mut() {
        if !body.is_alive { continue; }

        let mut total_frost:  u32 = 0;
        let mut total_burn:   u32 = 0;
        let mut total_infect: u32 = 0;

        for limb_entity in body_children.iter() {
            if let Ok(frost) = frostbite_q.get(limb_entity) { total_frost  += frost.severity as u32; }
            if let Ok(burn)  = burn_q.get(limb_entity)      { total_burn   += burn.severity  as u32; }
            if let Ok(inf)   = infection_q.get(limb_entity) { total_infect += inf.severity   as u32; }
        }

        if total_frost > total_burn {
            let cooling = ((total_frost - total_burn) / 50).min(2) as u8;
            body.body_temp = body.body_temp.saturating_sub(cooling).max(20);
        } else if total_burn > total_frost {
            let heating = ((total_burn - total_frost) / 50).min(2) as u8;
            body.body_temp = body.body_temp.saturating_add(heating).min(45);
        } else {
            if body.body_temp < 37 { body.body_temp = body.body_temp.saturating_add(1); }
            else if body.body_temp > 37 { body.body_temp = body.body_temp.saturating_sub(1); }
        }

        if total_infect > 0 {
            let fever = ((total_infect / 32767) + 1).min(3) as u8;
            body.body_temp = body.body_temp.saturating_add(fever).min(45);
        }

        if body.body_temp < 28 {
            for_each_limb_child(body_children, &limb_query, |child| {
                if let Ok(mut organ) = organ_query.get_mut(child) {
                    if organ.organ_type == OrganType::Brain {
                        organ.add_damage(50);
                        println!("🧊 [{}] Критическая гипотермия!", body.name);
                    }
                }
            });
        }

        if body.body_temp >= 40 {
            let heat_dmg = ((body.body_temp - 39) as u32 * 30).min(500) as u16;
            for_each_limb_child(body_children, &limb_query, |child| {
                if let Ok(mut organ) = organ_query.get_mut(child) {
                    if organ.organ_type == OrganType::Brain {
                        organ.add_damage(heat_dmg);
                        println!("🔥 [{}] Гипертермия {}°C!", body.name, body.body_temp);
                    }
                }
            });
        }

        if body.body_temp < 35 || body.body_temp > 38 {
            println!("🌡️  [{}] Температура: {}°C", body.name, body.body_temp);
        }
    }
}

pub fn blood_loss_system(
    mut commands:   Commands,
    mut body_query: Query<(&mut Body, &Children)>,
    mut bleed_q:    Query<(Entity, &mut Bleeding)>,
) {
    for (mut body, body_children) in body_query.iter_mut() {
        if !body.is_alive { continue; }

        let mut total_loss = 0u16;

        for limb_entity in body_children.iter() {
            if let Ok((bleed_entity, mut bleeding)) = bleed_q.get_mut(limb_entity) {
                total_loss = total_loss.saturating_add(bleeding.0);
                let decay = (bleeding.0 / 20).max(1);
                bleeding.0 = bleeding.0.saturating_sub(decay);
                if bleeding.0 == 0 {
                    commands.entity(bleed_entity).remove::<Bleeding>();
                    println!("🩹 Кровотечение остановилось");
                }
            }
        }

        if total_loss > 0 {
            body.blood_volume = body.blood_volume.saturating_sub(total_loss);
            let blood_ratio = body.blood_volume as u32 * 100 / 5000;
            if blood_ratio < 80 {
                let extra = ((80 - blood_ratio) * 2).min(150) as u8;
                body.viscosity = 100u8.saturating_add(extra);
            } else {
                body.viscosity = body.viscosity.saturating_sub(1).max(100);
            }
            println!("🩸 [{}] -{} мл. Осталось: {} | Давление: {}",
                body.name, total_loss, body.blood_volume, body.blood_pressure);
        }
    }
}

pub fn internal_bleeding_system(
    mut body_query: Query<(&mut Body, &Children)>,
    mut internal_q: Query<&mut InternalBleeding>,
    hemo_q:         Query<&Hemothorax>,
) {
    for (mut body, body_children) in body_query.iter_mut() {
        if !body.is_alive { continue; }
        for limb_entity in body_children.iter() {
            if let Ok(mut internal) = internal_q.get_mut(limb_entity) {
                body.blood_volume = body.blood_volume.saturating_sub(internal.rate);
                internal.volume   = internal.volume.saturating_add(internal.rate);
                println!("🩸 [{}] Внутреннее -{} мл (полость: {} мл)",
                    body.name, internal.rate, internal.volume);
            }
            if let Ok(hemo) = hemo_q.get(limb_entity) {
                if hemo.lung_compression() > 30 {
                    println!("🫁 [{}] Гемоторакс: {} мл (сжатие {}%)",
                        body.name, hemo.volume, hemo.lung_compression());
                }
            }
        }
    }
}

// ── ИНФЕКЦИЯ ────────────────────────────────────────────────

fn collect_infected_recursive(
    children:  &Children,
    inf_query: &Query<(Entity, &Limb, &Infection)>,
    ch_query:  &Query<&Children>,
    result:    &mut Vec<Entity>,
) {
    for child in children.iter() {
        if inf_query.get(child).is_ok() {
            result.push(child);
        }
        if let Ok(grandchildren) = ch_query.get(child) {
            collect_infected_recursive(grandchildren, inf_query, ch_query, result);
        }
    }
}

pub fn infection_system(
    mut commands:   Commands,
    body_query:     Query<(&Body, &Children)>,
    mut inf_query:  Query<(Entity, &Limb, &mut Infection)>,
    inf_read:       Query<(Entity, &Limb, &Infection)>,
    infectable_q:   Query<&Infectable>,
    parent_query:   Query<&ChildOf>,
    children_query: Query<&Children>,
    limb_check:     Query<&Limb>,
) {
    for (body, body_children) in body_query.iter() {
        if !body.is_alive { continue; }

        // Собираем все заражённые конечности через немутабельный query
        let mut all_infected: Vec<Entity> = Vec::new();
        collect_infected_recursive(body_children, &inf_read, &children_query, &mut all_infected);

        let mut spread_to: Vec<(Entity, u16)> = Vec::new();

        for entity in &all_infected {
            if let Ok((_, _, inf)) = inf_read.get(*entity) {
                if inf.severity >= INF_SPREAD {
                    // Вверх — на родителя
                    if let Ok(parent_of) = parent_query.get(*entity) {
                        let parent = parent_of.parent();
                        if infectable_q.get(parent).is_ok()
                            && inf_read.get(parent).is_err()
                        {
                            spread_to.push((parent, inf.severity / 2));
                        }
                    }

                    // Вниз — на дочерние конечности
                    if let Ok(children) = children_query.get(*entity) {
                        for child in children.iter() {
                            if limb_check.get(child).is_ok()
                                && infectable_q.get(child).is_ok()
                                && inf_read.get(child).is_err()
                            {
                                spread_to.push((child, inf.severity / 3));
                            }
                        }
                    }

                    // На соседей через общего родителя
                    if let Ok(parent_of) = parent_query.get(*entity) {
                        let parent = parent_of.parent();
                        if let Ok(siblings) = children_query.get(parent) {
                            for sibling in siblings.iter() {
                                if sibling == *entity { continue; }
                                if limb_check.get(sibling).is_ok()
                                    && infectable_q.get(sibling).is_ok()
                                    && inf_read.get(sibling).is_err()
                                {
                                    spread_to.push((sibling, inf.severity / 4));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Обновляем severity
        for entity in &all_infected {
            if let Ok((_, limb, mut inf)) = inf_query.get_mut(*entity) {
                inf.severity = inf.severity.saturating_add(200).min(INF_GANGRENE);
                if inf.severity >= INF_GANGRENE {
                    println!("☠️  [{:?}] ГАНГРЕНА!", limb.kind);
                } else {
                    println!("🦠 [{:?}] Инфекция: {}", limb.kind, inf.severity);
                }
            }
        }

        // Заражаем новые конечности
        for (target, severity) in spread_to {
            commands.entity(target).insert(Infection { severity });
            if let Ok(limb) = limb_check.get(target) {
                println!("🦠 Инфекция → [{:?}] severity={}", limb.kind, severity);
            }
        }
    }
}

pub fn regen_system(
    body_query:     Query<(&Body, &Children)>,
    limb_query:     Query<&Children, With<Limb>>,
    heart_query:    Query<&Heart>,
    inf_query:      Query<&Infection>,
    mut skin_query: Query<(&Limb, &mut SkinLayer)>,
    mut musc_query: Query<(&Limb, &mut MuscleLayer)>,
    mut bone_query: Query<(&Limb, &mut BoneLayer)>,
    mut commands:   Commands,
    skin_entities:  Query<Entity, With<SkinLayer>>,
    musc_entities:  Query<Entity, With<MuscleLayer>>,
    bone_entities:  Query<Entity, With<BoneLayer>>,
) {
    for (body, body_children) in body_query.iter() {
        if !body.is_alive { continue; }

        let mut heart_eff: u32 = 0;
        for_each_limb_child(body_children, &limb_query, |child| {
            if let Ok(heart) = heart_query.get(child) {
                heart_eff = heart.regen_efficiency() as u32;
            }
        });

        let spo2_factor    = body.spo2 as u32 * 100 / 98;
        let blood_factor   = (body.blood_volume as u32 * 100 / 5000).min(100);
        let hypoxia_factor = 100u32.saturating_sub(body.hypoxia as u32);
        let temp_factor    = match body.body_temp {
            t if t < 35 => 50u32,
            35           => 70,
            36           => 90,
            37           => 100,
            38           => 90,
            39           => 80,
            _            => 60,
        };

        let regen_rate = heart_eff
            * spo2_factor    / 100
            * blood_factor   / 100
            * hypoxia_factor / 100
            * temp_factor    / 100;

        if regen_rate == 0 { continue; }

        for limb_entity in body_children.iter() {
            regen_limb(limb_entity, regen_rate, &inf_query,
                &mut skin_query, &mut musc_query, &mut bone_query,
                &mut commands, &skin_entities, &musc_entities, &bone_entities);
        }
    }
}

fn regen_limb(
    limb_entity: Entity,
    regen_rate:  u32,
    inf_query:   &Query<&Infection>,
    skin_query:  &mut Query<(&Limb, &mut SkinLayer)>,
    musc_query:  &mut Query<(&Limb, &mut MuscleLayer)>,
    bone_query:  &mut Query<(&Limb, &mut BoneLayer)>,
    commands:    &mut Commands,
    _s: &Query<Entity, With<SkinLayer>>,
    _m: &Query<Entity, With<MuscleLayer>>,
    _b: &Query<Entity, With<BoneLayer>>,
) {
    let inf_sev = inf_query.get(limb_entity).map(|i| i.severity).unwrap_or(0);
    let skin_blocked   = inf_sev >= INF_SKIN_BLOCK;
    let muscle_blocked = inf_sev >= INF_MUSCLE_BLOCK;

    if !skin_blocked {
        if let Ok((_, mut skin)) = skin_query.get_mut(limb_entity) {
            let r = (regen_rate * 10).min(500) as u16;
            skin.slash_damage = skin.slash_damage.saturating_sub(r / 2);
            skin.burn_damage  = skin.burn_damage .saturating_sub(r / 4);
            skin.acid_damage  = skin.acid_damage .saturating_sub(r / 4);
            skin.frost_damage = skin.frost_damage.saturating_sub(r / 3);
            if skin.is_empty() { commands.entity(limb_entity).remove::<SkinLayer>(); }
        }
    }

    if !muscle_blocked {
        if let Ok((_, mut muscle)) = musc_query.get_mut(limb_entity) {
            let r = (regen_rate * 5).min(200) as u16;
            muscle.blunt_damage  = muscle.blunt_damage .saturating_sub(r / 2);
            muscle.slash_damage  = muscle.slash_damage .saturating_sub(r / 3);
            muscle.pierce_damage = muscle.pierce_damage.saturating_sub(r / 3);
            muscle.burn_damage   = muscle.burn_damage  .saturating_sub(r / 5);
            muscle.frost_damage  = muscle.frost_damage .saturating_sub(r / 4);
            if muscle.is_empty() { commands.entity(limb_entity).remove::<MuscleLayer>(); }
        }
    }

    if !muscle_blocked && musc_query.get(limb_entity).is_ok() {
        if let Ok((_, mut bone)) = bone_query.get_mut(limb_entity) {
            let r = (regen_rate / 10).min(5) as u8;
            bone.0 = bone.0.saturating_sub(r);
            if bone.is_healthy() { commands.entity(limb_entity).remove::<BoneLayer>(); }
        }
    }
}

pub fn brain_death_system(
    mut body_query:  Query<(&mut Body, &Children)>,
    limb_query:      Query<&Children, With<Limb>>,
    mut organ_query: Query<&mut Organ>,
) {
    for (mut body, body_children) in body_query.iter_mut() {
        if !body.is_alive { continue; }
        let hypoxia = body.hypoxia;
        for_each_limb_child(body_children, &limb_query, |child| {
            if let Ok(mut organ) = organ_query.get_mut(child) {
                if organ.organ_type == OrganType::Brain {
                    if hypoxia > 30 {
                        let brain_dmg = ((hypoxia - 30) as u32 * 5).min(500) as u16;
                        organ.add_damage(brain_dmg);
                        println!("🧠 [{}] Мозг -{} от гипоксии ({}/{})",
                            body.name, brain_dmg, organ.damage, organ.max_damage());
                    }
                    if organ.is_destroyed() {
                        body.is_alive = false;
                        println!("💀 [{}] СМЕРТЬ: Мозг разрушен", body.name);
                    }
                }
            }
        });
    }
}
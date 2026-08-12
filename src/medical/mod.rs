// mod.rs
pub mod body;
pub mod limb;
pub mod damage;

use bevy::prelude::*;
pub use body::Body;
pub use limb::{
    Limb, LimbType,
    SkinLayer, MuscleLayer, BoneLayer,
    Bleeding, Frostbite, BurnDamage, OrganDamage,
};
pub use damage::{DamageHit, DamageComponent, LimbContext, apply_damage};

pub struct MedicalPlugin;

impl Plugin for MedicalPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_seconds(1.0))
           .add_systems(FixedUpdate, (medical_tick_system, blood_loss_system));
    }
}

fn medical_tick_system(
    mut body_query: Query<(Entity, &mut Body, &Children)>,
    limb_query: Query<&Children, With<Limb>>,
    organ_query: Query<&OrganDamage>,
) {
    for (_body_entity, mut body, body_children) in body_query.iter_mut() {
        if !body.is_alive { continue; }

        let mut heart_working = false;

        // Органы теперь дети конечностей а не тела напрямую
        // поэтому ищем через два уровня: тело → конечность → орган
        for limb_entity in body_children.iter() {
            if let Ok(limb_children) = limb_query.get(limb_entity) {
                for child in limb_children.iter() {
                    if let Ok(organ) = organ_query.get(child) {
                        if organ.is_heart && organ.damage < organ.max_damage {
                            heart_working = true;
                        }
                    }
                }
            }
        }

        if !heart_working {
            body.is_alive = false;
            println!("💀 [{}] УМЕР: Сердце разрушено!", body.name);
            continue;
        }

        if body.radiation > 0 {
            println!("☢️ [{}] облучён. Уровень: {}", body.name, body.radiation);
        }
    }
}

fn blood_loss_system(
    mut commands: Commands,
    mut body_query: Query<(&mut Body, &Children)>,
    mut bleeding_query: Query<(Entity, &mut Bleeding)>,
    limb_query: Query<&Children, With<Limb>>,
    mut organ_query: Query<&mut OrganDamage>,
) {
    for (mut body, body_children) in body_query.iter_mut() {
        if !body.is_alive { continue; }

        let mut total_loss = 0u16;

        // Собираем кровотечение со всех конечностей
        for limb_entity in body_children.iter() {
            if let Ok((bleed_entity, mut bleeding)) = bleeding_query.get_mut(limb_entity) {
                total_loss = total_loss.saturating_add(bleeding.0);

                let decay = (bleeding.0 / 10).max(1);
                bleeding.0 = bleeding.0.saturating_sub(decay);

                if bleeding.0 == 0 {
                    commands.entity(bleed_entity).remove::<Bleeding>();
                    println!("🩹 Кровотечение остановилось");
                }
            }
        }

        if total_loss > 0 {
            body.blood_volume = body.blood_volume.saturating_sub(total_loss);
            println!("🩸 [{}] -{} мл крови. Осталось: {}", body.name, total_loss, body.blood_volume);

            // Гиповолемический шок через два уровня
            for limb_entity in body_children.iter() {
                if let Ok(limb_children) = limb_query.get(limb_entity) {
                    for child in limb_children.iter() {
                        if let Ok(mut organ) = organ_query.get_mut(child) {
                            if organ.is_heart {
                                organ.damage = organ.damage
                                    .saturating_add(total_loss / 2)
                                    .min(organ.max_damage);
                            }
                        }
                    }
                }
            }

            if body.blood_volume < 1500 {
                body.is_alive = false;
                println!("💀 [{}] УМЕР: Обескровливание!", body.name);
            }
        }
    }
}
// main.rs
use bevy::prelude::*;

mod medical;
use medical::{Body, MedicalPlugin, Infection};
use medical::limb::{Side, Limb, LimbKind, SkinLayer, MuscleLayer, BoneLayer,
                    Bleeding, Frostbite, BurnDamage, Infectable};
use medical::organ::Organ;
use medical::damage::{apply_damage, DamageHit, LimbContext};

#[derive(Resource)]
struct DamageApplied(bool);

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(MedicalPlugin)
        .insert_resource(DamageApplied(false))
        .add_systems(Startup, setup_game)
        .add_systems(FixedUpdate, (apply_test_damage, print_status).chain())
        .run();
}

fn setup_game(mut commands: Commands) {
    println!("--- Медицинская система ---\n");

    let petya = Body::spawn(&mut commands, "Игрок Петя");
    Body::attach_head(&mut commands, petya);
    Body::attach_arm(&mut commands, petya, Side::Left);
    Body::attach_arm(&mut commands, petya, Side::Right);
    Body::attach_leg(&mut commands, petya, Side::Left);
    Body::attach_leg(&mut commands, petya, Side::Right);

    println!("✅ Петя заспавнен\n");
}

fn apply_test_damage(
    mut commands: Commands,
    mut flag: ResMut<DamageApplied>,
    body_query: Query<&Children, With<Body>>,
    // Ищем кисть левой руки — она дочерняя к предплечью
    // которое дочернее к плечу которое дочернее к Body
    // поэтому ищем через все конечности рекурсивно
    limb_query: Query<(Entity, &Limb)>,
    children_query: Query<&Children>,
) {
    if flag.0 { return; }
    flag.0 = true;

    for body_children in body_query.iter() {
        // Рекурсивно ищем левую кисть
        if let Some(hand_entity) = find_limb_recursive(
            body_children,
            LimbKind::Hand(medical::limb::Side::Left),
            &limb_query,
            &children_query,
        ) {
            // Накидываем инфекцию на кисть
            commands.entity(hand_entity).insert(Infection { severity: 40 });
            println!("🦠 Инфекция появилась на левой кисти!\n");
        }
    }
}

// Рекурсивный поиск конечности по типу
fn find_limb_recursive(
    children: &Children,
    target: LimbKind,
    limb_query: &Query<(Entity, &Limb)>,
    children_query: &Query<&Children>,
) -> Option<Entity> {
    for child in children.iter() {
        if let Ok((entity, limb)) = limb_query.get(child) {
            if limb.kind == target {
                return Some(entity);
            }
        }
        if let Ok(grandchildren) = children_query.get(child) {
            if let Some(found) = find_limb_recursive(
                grandchildren, target, limb_query, children_query
            ) {
                return Some(found);
            }
        }
    }
    None
}

fn print_status(
    body_query:     Query<(&Body, &Children)>,
    limb_query:     Query<(&Limb, Option<&SkinLayer>, Option<&MuscleLayer>,
                           Option<&BoneLayer>, Option<&Bleeding>, Option<&Infection>)>,
    organ_query:    Query<&Organ>,
    children_query: Query<&Children>,
) {
    for (body, body_children) in body_query.iter() {
        println!("══════════════════════════════");
        println!("👤 {} | Кровь: {}мл | SpO2: {}% | Давление: {} | Гипоксия: {}% | Темп: {}°C{}",
            body.name, body.blood_volume, body.spo2,
            body.blood_pressure, body.hypoxia, body.body_temp,
            if !body.is_alive { " | 💀 МЁРТВ" } else { "" }
        );

        // Рекурсивный вывод всех конечностей
        print_limbs_recursive(body_children, 1, &limb_query, &organ_query, &children_query);
        println!();
    }
}

fn print_limbs_recursive(
    children:       &Children,
    depth:          usize,
    limb_query:     &Query<(&Limb, Option<&SkinLayer>, Option<&MuscleLayer>,
                            Option<&BoneLayer>, Option<&Bleeding>, Option<&Infection>)>,
    organ_query:    &Query<&Organ>,
    children_query: &Query<&Children>,
) {
    let indent = "  ".repeat(depth);

    for child in children.iter() {
        if let Ok((limb, skin, muscle, bone, bleed, infection)) = limb_query.get(child) {
            let has_damage = skin.is_some() || muscle.is_some()
                || bone.is_some() || bleed.is_some() || infection.is_some();

            if has_damage {
                println!("{}🦴 {:?}", indent, limb.kind);

                if let Some(inf) = infection {
                    let stage = if inf.severity >= 100 { "ГАНГРЕНА☠️" }
                           else if inf.severity >= 80  { "тяжёлая" }
                           else if inf.severity >= 50  { "средняя" }
                           else                        { "лёгкая" };
                    println!("{}  🦠 Инфекция: {}% ({})", indent, inf.severity, stage);
                }
                if let Some(s) = skin {
                    println!("{}  Кожа:  порезы={} ожоги={} кислота={} мороз={} / макс={}",
                        indent, s.slash_damage, s.burn_damage,
                        s.acid_damage, s.frost_damage, limb.skin_max);
                }
                if let Some(m) = muscle {
                    println!("{}  Мышцы: тупой={} порез={} пробой={} ожог={} мороз={} / макс={}",
                        indent, m.blunt_damage, m.slash_damage, m.pierce_damage,
                        m.burn_damage, m.frost_damage, limb.muscle_max);
                }
                if let Some(b) = bone {
                    let state = if b.is_shattered() { "РАЗДРОБЛЕНА" }
                           else if b.is_fractured() { "СЛОМАНА" }
                           else if b.has_crack()    { "ТРЕЩИНА" }
                           else                     { "здорова" };
                    println!("{}  Кость: {}% ({})", indent, b.0, state);
                }
                if let Some(bl) = bleed {
                    println!("{}  🩸 Кровотечение: {}", indent, bl.0);
                }
            }

            // Органы
            if let Ok(limb_children) = children_query.get(child) {
                for lc in limb_children.iter() {
                    if let Ok(organ) = organ_query.get(lc) {
                        if organ.damage > 0 {
                            println!("{}  💔 {}: {}/{} ({:.0}% повреждён)",
                                indent, organ.name(), organ.damage, organ.max_damage(),
                                (1.0 - organ.health_pct()) * 100.0);
                        }
                    }
                }
                // Рекурсивно дочерние конечности
                print_limbs_recursive(limb_children, depth + 1,
                    limb_query, organ_query, children_query);
            }
        }
    }
}
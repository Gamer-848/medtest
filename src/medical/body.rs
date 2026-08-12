// body.rs
use bevy::prelude::*;
use super::limb::{Limb, OrganDamage};

#[derive(Component, Debug)]
pub struct Body {
    pub name:         String,
    pub is_alive:     bool,
    pub blood_volume: u16,   // норма 5000 мл
    pub oxygen_level: u8,    // норма 100%
    pub radiation:    u16,
}

impl Body {
    // Конструктор спавнит минимальное тело — торакс и абдомен
    // всё остальное добавляется отдельно
    pub fn spawn(commands: &mut Commands, name: &str) -> Entity {
        commands.spawn((
            Body {
                name:         name.to_string(),
                is_alive:     true,
                blood_volume: 5000,
                oxygen_level: 100,
                radiation:    0,
            },
        )).with_children(|parent| {
            // Торакс — всегда есть, содержит сердце
            parent.spawn(Limb::thorax()).with_children(|thorax| {
                thorax.spawn(OrganDamage {
                    name:       "Сердце".to_string(),
                    damage:     0,
                    max_damage: 40000,
                    is_heart:   true,
                });
                thorax.spawn(OrganDamage {
                    name:       "Лёгкие".to_string(),
                    damage:     0,
                    max_damage: 30000,
                    is_heart:   false,
                });
            });

            // Абдомен — всегда есть
            parent.spawn(Limb::abdomen()).with_children(|abdomen| {
                abdomen.spawn(OrganDamage {
                    name:       "Печень".to_string(),
                    damage:     0,
                    max_damage: 25000,
                    is_heart:   false,
                });
            });
        }).id()
    }

    // Добавить голову после спавна
    pub fn attach_head(commands: &mut Commands, body_entity: Entity) -> Entity {
        let head = commands.spawn(Limb::head()).id();
        commands.entity(body_entity).add_child(head);
        head
    }

    // Добавить руку
    pub fn attach_arm(commands: &mut Commands, body_entity: Entity, is_left: bool) -> Entity {
        use super::limb::{LimbType};

        let mut shoulder = Limb::shoulder();
        let mut forearm  = Limb { limb_type: if is_left { LimbType::LeftForearm  } else { LimbType::RightForearm  }, ..Limb::shoulder() };
        let mut hand     = Limb { limb_type: if is_left { LimbType::LeftHand     } else { LimbType::RightHand     }, ..Limb::shoulder() };
        shoulder.limb_type = if is_left { LimbType::LeftShoulder } else { LimbType::RightShoulder };

        let hand_e     = commands.spawn(hand).id();
        let forearm_e  = commands.spawn(forearm).add_child(hand_e).id();
        let shoulder_e = commands.spawn(shoulder).add_child(forearm_e).id();

        commands.entity(body_entity).add_child(shoulder_e);
        shoulder_e
    }

    // Добавить ногу
    pub fn attach_leg(commands: &mut Commands, body_entity: Entity, is_left: bool) -> Entity {
        use super::limb::LimbType;

        let thigh_type = if is_left { LimbType::LeftThigh } else { LimbType::RightThigh };
        let crus_type  = if is_left { LimbType::LeftCrus  } else { LimbType::RightCrus  };
        let foot_type  = if is_left { LimbType::LeftFoot  } else { LimbType::RightFoot  };

        let foot_e  = commands.spawn(Limb { limb_type: foot_type,  ..Limb::thigh() }).id();
        let crus_e  = commands.spawn(Limb { limb_type: crus_type,  ..Limb::thigh() }).add_child(foot_e).id();
        let thigh_e = commands.spawn(Limb { limb_type: thigh_type, ..Limb::thigh() }).add_child(crus_e).id();

        commands.entity(body_entity).add_child(thigh_e);
        thigh_e
    }
}
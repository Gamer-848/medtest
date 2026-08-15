// body.rs
use bevy::prelude::*;
use super::limb::{Limb, LimbKind, Side, Infectable};
use super::organ::{Organ, OrganType, Heart};

#[derive(Component, Debug)]
pub struct Body {
    pub name:           String,
    pub is_alive:       bool,
    pub blood_volume:   u16,
    pub viscosity:      u8,
    pub blood_pressure: u8,
    pub spo2:           u8,
    pub hypoxia:        u8,
    pub body_temp:      u8,
    pub o2_debt:        u8,
    pub radiation:      u16,
}

impl Body {
    pub fn spawn(commands: &mut Commands, name: &str) -> Entity {
        commands.spawn(Body {
            name:           name.to_string(),
            is_alive:       true,
            blood_volume:   5000,
            viscosity:      100,
            blood_pressure: 100,
            spo2:           98,
            hypoxia:        0,
            body_temp:      37,
            o2_debt:        0,
            radiation:      0,
        }).with_children(|body| {
            // Торакс и абдомен — всегда Infectable
            body.spawn((Limb::new(LimbKind::Thorax), Infectable)).with_children(|thorax| {
                thorax.spawn(Organ::new(OrganType::Heart));
                thorax.spawn(Organ::new(OrganType::Lungs));
                thorax.spawn(Heart::human());
            });
            body.spawn((Limb::new(LimbKind::Abdomen), Infectable)).with_children(|abdomen| {
                abdomen.spawn(Organ::new(OrganType::Liver));
                abdomen.spawn(Organ::new(OrganType::Stomach));
                abdomen.spawn(Organ::new(OrganType::Spleen));
            });
        }).id()
    }

    pub fn attach_head(commands: &mut Commands, body: Entity) -> Entity {
        let head = commands.spawn((Limb::new(LimbKind::Head), Infectable))
            .with_children(|h| { h.spawn(Organ::new(OrganType::Brain)); })
            .id();
        commands.entity(body).add_child(head);
        head
    }

    pub fn attach_arm(commands: &mut Commands, body: Entity, side: Side) -> Entity {
        let hand     = commands.spawn((Limb::new(LimbKind::Hand(side)),     Infectable)).id();
        let forearm  = commands.spawn((Limb::new(LimbKind::Forearm(side)),  Infectable)).add_child(hand).id();
        let shoulder = commands.spawn((Limb::new(LimbKind::Shoulder(side)), Infectable)).add_child(forearm).id();
        commands.entity(body).add_child(shoulder);
        shoulder
    }

    pub fn attach_leg(commands: &mut Commands, body: Entity, side: Side) -> Entity {
        let foot  = commands.spawn((Limb::new(LimbKind::Foot(side)),  Infectable)).id();
        let crus  = commands.spawn((Limb::new(LimbKind::Crus(side)),  Infectable)).add_child(foot).id();
        let thigh = commands.spawn((Limb::new(LimbKind::Thigh(side)), Infectable)).add_child(crus).id();
        commands.entity(body).add_child(thigh);
        thigh
    }
}

#[derive(Component, Debug)]
pub struct InternalBleeding { pub rate: u16, pub volume: u16 }

#[derive(Component, Debug)]
pub struct Hemothorax { pub volume: u16 }

impl Hemothorax {
    pub fn lung_compression(&self) -> u8 {
        ((self.volume as u32 * 100 / 1500).min(100)) as u8
    }
}
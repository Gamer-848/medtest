// mod.rs
pub mod body;
pub mod limb;
pub mod organ;
pub mod damage;
pub mod systems;

use bevy::prelude::*;

pub use body::{Body, InternalBleeding, Hemothorax};
pub use limb::{Limb, LimbKind, Side, SkinLayer, MuscleLayer, BoneLayer,
               Bleeding, Frostbite, BurnDamage, Infection};
pub use organ::{Organ, OrganType, Heart, HeartRhythm, HeartStats, Lungs};
pub use damage::{DamageHit, DamageComponent, LimbContext, apply_damage};

use self::systems::*;

pub struct MedicalPlugin;

impl Plugin for MedicalPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_seconds(1.0))
           .add_systems(FixedUpdate, (
               heart_system,
               oxygen_system,
               temperature_system,
               blood_loss_system,
               internal_bleeding_system,
               infection_system,
               regen_system,
               brain_death_system,
           ).chain());
    }
}
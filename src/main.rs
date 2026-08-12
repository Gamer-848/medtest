// main.rs
use bevy::prelude::*;

mod medical;
use medical::{Body, MedicalPlugin};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(MedicalPlugin)
        .add_systems(Startup, setup_game)
        .run();
}

fn setup_game(mut commands: Commands) {
    println!("--- Медицинская система ---\n");

    // Петя — здоровый, полный комплект конечностей
    let petya = Body::spawn(&mut commands, "Игрок Петя");
    Body::attach_head(&mut commands, petya);
    Body::attach_arm(&mut commands, petya, true);   // левая
    Body::attach_arm(&mut commands, petya, false);  // правая
    Body::attach_leg(&mut commands, petya, true);
    Body::attach_leg(&mut commands, petya, false);

    println!("✅ Петя заспавнен с головой, руками и ногами");

    // Вася — только торакс и абдомен (минимальное тело)
    // руки нет — ампутирована, голова есть
    let vasya = Body::spawn(&mut commands, "Рейдер Вася");
    Body::attach_head(&mut commands, vasya);
    Body::attach_leg(&mut commands, vasya, true);
    Body::attach_leg(&mut commands, vasya, false);
    // левой руки нет — не добавляем, система не сломается

    println!("✅ Вася заспавнен без рук");
}
use bevy::prelude::*;

use crate::eprintln;

pub fn run() {
    App::new().add_systems(Startup, hello_world).run();
}

fn hello_world() {
    eprintln!("Hello World!");
}

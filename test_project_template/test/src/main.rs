use bevy::prelude::*;
mod modules;
mod lua_bindings;
mod aethaum_predefined;
fn main() {
    App::new().add_plugins(DefaultPlugins).add_plugins(lua_bindings::LuaPlugin).run();
}

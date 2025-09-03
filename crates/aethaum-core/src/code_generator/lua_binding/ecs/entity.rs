use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{AppTypeRegistry, Commands};
use bevy_ecs::reflect::ReflectCommandExt;
use crate::code_generator::lua_binding::execution_context::{LuaExecutionContext, LuaReflectInitRegistry};
use crate::code_generator::lua_binding::reflection::get_type_id_by_str;

///实体操作相关将使用一个预定义的bevy Event，并定义一个LuaEcsBridge，通过mpsc channel返回实际的返回值给lua
pub fn spawn_empty_fn(commands: &mut Commands) -> Entity {
    commands.spawn_empty().id()
}
pub fn spawn_fn(commands: &mut Commands, bundle: impl Bundle) -> Entity {
    commands.spawn(bundle).id()
}
pub fn spawn_reflect_fn(
    commands: &mut Commands,
    proto_name: &str,
    lua_value: Option<mlua::Value>,
    lua_execution_context: &LuaExecutionContext,
    type_registration: &AppTypeRegistry,
    lua_bundle_registry: &LuaReflectInitRegistry,
) -> Option<Entity> {
    let registry = type_registration.read();
    let type_id = get_type_id_by_str(proto_name, &registry)?;

    if let Some(lua_bundle_fn) = lua_bundle_registry.get_constructor(type_id) {
        let entity = commands.spawn_empty().id();
        let bundle = if let Some(value) = lua_value {
            //TODO: better error handling
            (lua_bundle_fn.from_lua)(value, &lua_execution_context.lua).ok()?
        }else {
            (lua_bundle_fn.default)()
        };
        commands.entity(entity).insert_reflect(bundle);
        Some(entity)
    }else {
        None
    }
}
pub fn despawn_fn(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn();
}
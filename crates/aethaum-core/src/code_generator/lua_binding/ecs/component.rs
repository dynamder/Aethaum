use bevy_ecs::prelude::*;
use bevy_ecs::reflect::ReflectCommandExt;
use bevy_ecs::world::reflect::GetComponentReflectError;
use bevy_reflect::TypeRegistry;
use crate::code_generator::lua_binding::ecs::{AethaumEcsCommandResponse};
use crate::code_generator::lua_binding::execution_context::LuaReflectInitRegistry;
use crate::code_generator::lua_binding::reflection::{get_type_id_by_str, ReflectFromLua, ReflectToLua};

///Get Component related
pub fn handle_get_component_result(result: GetComponentResult) -> AethaumEcsCommandResponse {
    match result {
        GetComponentResult::Ok(value) => AethaumEcsCommandResponse::GetComponents {
            components: vec![Some(value)]
        },
        GetComponentResult::ComponentNotHave => AethaumEcsCommandResponse::GetComponents {
            components: vec![None]
        },
        GetComponentResult::EntityNotExist(e) => {
            eprintln!("Warning: Entity {e} not exist, returning nil");
            AethaumEcsCommandResponse::GetComponents {
                components: vec![None]
            }
        },
        GetComponentResult::NoSuchComponent(component_name) => {
            eprintln!("Warning: No such component {component_name}, returning nil");
            AethaumEcsCommandResponse::GetComponents {
                components: vec![None]
            }
        },
        GetComponentResult::ConversionErr(err) => {
            eprintln!("Warning: Failed to convert component to Lua value: {err}, returning nil");
            AethaumEcsCommandResponse::GetComponents {
                components: vec![None]
            }
        },
        GetComponentResult::Multiple(multiple) => {
            AethaumEcsCommandResponse::GetComponents {
                components: multiple.into_iter()
                    .flat_map(|component_result| {
                        match handle_get_component_result(component_result) {
                            AethaumEcsCommandResponse::GetComponents { components } => components,
                            _ => unreachable!()
                        }
                    })
                    .collect()
            }
        }
    }
}
pub enum GetComponentResult {
    Ok(mlua::Value),
    ComponentNotHave,
    NoSuchComponent(String),
    EntityNotExist(Entity),
    ConversionErr(mlua::Error),
    Multiple(Vec<GetComponentResult>)
}
impl FromIterator<GetComponentResult> for GetComponentResult {
    fn from_iter<I: IntoIterator<Item = GetComponentResult>>(iter: I) -> Self {
        let mut result = Vec::new();
        for item in iter {
            match item {
                GetComponentResult::Ok(value) => result.push(
                    GetComponentResult::Ok(value)
                ),
                GetComponentResult::ComponentNotHave => result.push(
                    GetComponentResult::ComponentNotHave
                ),
                GetComponentResult::NoSuchComponent(name) => result.push(
                    GetComponentResult::NoSuchComponent(name)
                ),
                GetComponentResult::EntityNotExist(e) => return GetComponentResult::EntityNotExist(e),
                GetComponentResult::ConversionErr(err) => result.push(
                    GetComponentResult::ConversionErr(err)
                ),
                GetComponentResult::Multiple(vec) => result.extend(vec),
            }
        }
        if result.len() == 1 {
            result.pop().unwrap()
        }else {
            GetComponentResult::Multiple(result)
        }
    }
}

pub fn get_components_fn(
    world: &World,
    entity: Entity,
    components: Option<&Vec<String>>,
    registry: &TypeRegistry,
    lua: &mlua::Lua,
) -> GetComponentResult {
    if let Some(components) = components {
        match world.get_entity(entity) {
            Ok(_) => {},
            Err(_) => return GetComponentResult::EntityNotExist(entity),
        };
        components.iter()
            .map(|component_name| {
                get_single_component(world, entity, component_name, registry, lua)
            })
            .collect()
    }else {
        match world.inspect_entity(entity) {
            Ok(components_info) => {
                components_info
                    .map(|component_info| {
                        get_single_component(world, entity, component_info.name(), registry, lua)
                    })
                    .collect()
            },
            Err(_) => GetComponentResult::EntityNotExist(entity),
        }
    }
}
fn get_single_component(
    world: &World,
    entity: Entity,
    component_name: &str,
    registry: &TypeRegistry,
    lua: &mlua::Lua,
) -> GetComponentResult {
    let component_type_id = match get_type_id_by_str(component_name, registry) {
        Some(type_id) => type_id,
        None => {
            return GetComponentResult::NoSuchComponent(component_name.to_string());
        }
    };

    let reflect_component = match world.get_reflect(entity, component_type_id) {
        Ok(reflect_component) => reflect_component,
        Err(err) => {
            match err {
                GetComponentReflectError::NoCorrespondingComponentId(_) => unreachable!("This error will return early before."),
                GetComponentReflectError::EntityDoesNotHaveComponent {..} => {
                    return GetComponentResult::ComponentNotHave;
                },
                GetComponentReflectError::MissingAppTypeRegistry => panic!("Missing AppTypeRegistry in generated Rust Bevy, unexpected error."),
                GetComponentReflectError::MissingReflectFromPtrTypeData(_) => unreachable!("All component are auto registered by Aethaum.")
            }
        }
    };

    let lua_registration = registry.get_type_data::<ReflectToLua>(component_type_id)
        .expect("Missing ReflectToLua type data in generated Rust Bevy, unexpected error.");
    // 获取组件反射数据
    if let Some(to_lua_object) = lua_registration.get(reflect_component) {
        match to_lua_object.to_lua(lua) {
            Ok(value) => GetComponentResult::Ok(value),
            Err(err) => GetComponentResult::ConversionErr(err)
        }
    }else {
        unreachable!("All components have an auto implemented ToLua Trait by Aethaum.")
    }
}
/// Add Component Related
pub enum AddComponentResult {
    Ok,
    EntityDoesNotExist(Entity),
    NoSuchComponent(String),
    ConversionErr(mlua::Error),
}
pub fn add_component_fn(
    commands: &mut Commands,
    entity: Entity,
    component_name: &str,
    init_value: Option<mlua::Value>,
    type_registry: &TypeRegistry,
    lua_reflect_init_registry: &LuaReflectInitRegistry,
    lua: &mlua::Lua
) -> AddComponentResult {
    let component_type_id = match get_type_id_by_str(component_name, type_registry) {
        Some(type_id) => type_id,
        None => {
            return AddComponentResult::NoSuchComponent(component_name.to_string());
        }
    };
    let component_constructor = match lua_reflect_init_registry.get_constructor(component_type_id) {
        Some(constructor) => constructor,
        None => {
            return AddComponentResult::NoSuchComponent(component_name.to_string());
        }
    };
    let constructed_component = match init_value {
        Some(value) => {
            match (component_constructor.from_lua)(value, lua) {
                Ok(init_value) => init_value,
                Err(err) => return AddComponentResult::ConversionErr(err)
            }
        },
        None => (component_constructor.default)()
    };
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.insert_reflect(constructed_component);
    }else {
        return AddComponentResult::EntityDoesNotExist(entity);
    }
    AddComponentResult::Ok
}
pub enum RemoveComponentResult {
    Ok,
    EntityDoesNotExist(Entity),
    NoSuchComponent(String),
}
pub fn remove_component_fn( //TODO: Not Sure if this function works properly
    commands: &mut Commands,
    entity: Entity,
    component_name: &str,
    type_registry: &TypeRegistry,
) -> RemoveComponentResult {
    let component_type_id = match get_type_id_by_str(component_name, type_registry) {
        Some(type_id) => type_id,
        None => {
            return RemoveComponentResult::NoSuchComponent(component_name.to_string());
        }
    };
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.remove_reflect(component_name.to_string());
    }else {
        return RemoveComponentResult::EntityDoesNotExist(entity);
    }
    RemoveComponentResult::Ok
}
pub enum SetComponentResult {
    Ok,
    EntityDoesNotExist(Entity),
    NoSuchComponent(String),
    ConversionErr(mlua::Error),
}
///set a component's value, if it not exists, it will be added.
/// note that you must give a value, or you should use add_component_fn
pub fn set_component_fn(
    commands: &mut Commands,
    entity: Entity,
    component_name: &str,
    new_value: mlua::Value,
    type_registry: &TypeRegistry,
    lua: &mlua::Lua
) -> SetComponentResult {
    let component_type_id = match get_type_id_by_str(component_name, type_registry) {
        Some(type_id) => type_id,
        None => {
            return SetComponentResult::NoSuchComponent(component_name.to_string());
        }
    };
    let registry = type_registry.get_type_data::<ReflectFromLua>(component_type_id);
    if let Some(registry) = registry {
        match registry.from_lua(new_value, lua) {
            Ok(new_value) => {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.insert_reflect(new_value);
                }else {
                    return SetComponentResult::EntityDoesNotExist(entity);
                }
                SetComponentResult::Ok
            },
            Err(err) => SetComponentResult::ConversionErr(err)
        }
    }else {
        SetComponentResult::NoSuchComponent(component_name.to_string())
    }
}
pub fn has_component_fn(
    world: &World,
    entity: Entity,
    component_name: &str,
    type_registry: &TypeRegistry,
) -> bool {
    let component_type_id = match get_type_id_by_str(component_name, type_registry) {
        Some(type_id) => type_id,
        None => {
            return false;
        }
    };
    world.get_reflect(entity, component_type_id).is_ok()
}
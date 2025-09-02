///这些代码是在设计架构的时候，方便审查，测试用的，后续将会拆分至一系列输出TokenStream的函数用于代码生成
use std::any::{Any, TypeId};
use std::collections::HashMap;
use bevy_ecs::bundle::DynamicBundle;
use bevy_ecs::entity::EntityDoesNotExistError;
use bevy_ecs::prelude::*;
use bevy_ecs::reflect::{ReflectBundle, ReflectCommandExt};
use bevy_ecs::world::reflect::GetComponentReflectError;
use bevy_reflect::prelude::*;
use bevy_reflect::{TypeData, TypeRegistry};
use mlua::FromLua;
use thiserror::Error;
use crate::code_generator::lua_binding::execution_context::LuaExecutionContext;
use crate::code_generator::lua_binding::reflection::{get_type_id_by_str, ReflectToLua};
pub enum AethaumEcsCommandType {
    SpawnEntity {
        proto_name: Option<String>,
        init_value: Option<mlua::Value>,
    },
    DespawnEntity {
        entity: Entity,
    },
    GetComponents {
        entity: Entity,
        components: Option<Vec<String>>
    },
}
pub enum AethaumEcsCommandResponse {
    SpawnEntity {
        entity: Entity,
    },
    DespawnEntity,
    GetComponents {
        components: Vec<Option<mlua::Value>>
    },
    Error(String)
}
#[derive(Event)]
pub struct AethaumEcsCommand {
    pub command: AethaumEcsCommandType,
    pub response_sender: std::sync::mpsc::Sender<AethaumEcsCommandResponse>,
}


pub fn process_aethaum_ecs_command(
    world: &World,
    mut commands: Commands,
    mut events: EventReader<AethaumEcsCommand>,
    lua_execution_context: Res<LuaExecutionContext>,
    type_registration: Res<AppTypeRegistry>,
    lua_bundle_registry: Res<ReflectLuaBundleRegistry>,
) {
    for event in events.read() {
        let result = match &event.command {
            AethaumEcsCommandType::SpawnEntity { proto_name, init_value } => {
                if let Some(proto_name) = proto_name {
                    let entity = spawn_reflect_fn(
                        &mut commands,
                        &proto_name,
                        init_value.clone(), //TODO: 是否可以避免此处的clone
                        lua_execution_context.as_ref(),
                        type_registration.as_ref(),
                        lua_bundle_registry.as_ref(),
                    );
                    match entity {
                        Some(entity) => {
                            AethaumEcsCommandResponse::SpawnEntity {
                                entity
                            }
                        },
                        None => {
                            AethaumEcsCommandResponse::Error("Failed to spawn entity".to_string())
                        }
                    }
                }else {
                    AethaumEcsCommandResponse::SpawnEntity { entity: spawn_empty_fn(&mut commands)}
                }

            },
            AethaumEcsCommandType::DespawnEntity { entity } => {
                despawn_fn(&mut commands, *entity);
                AethaumEcsCommandResponse::DespawnEntity
            }
            AethaumEcsCommandType::GetComponents { entity, components } => {
                let type_registry = type_registration.read();
                let components = get_components_fn(
                    world,
                    *entity,
                    components.as_ref(),
                    &type_registry,
                    &lua_execution_context.lua
                );
                handle_get_component_result(components)
            }
        };
        if let Err(_) = event.response_sender.send(result) {
            eprintln!("Failed to send ECS command response");
        }
    }
}
fn handle_get_component_result(result: GetComponentResult) -> AethaumEcsCommandResponse {
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
    lua_bundle_registry: &ReflectLuaBundleRegistry,
) -> Option<Entity> {
    let registry = type_registration.read();
    let type_id = get_type_id_by_str(proto_name, &registry)?;

    if let Some(lua_bundle_fn) = lua_bundle_registry.get_bundle(type_id) {
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

#[derive(Error, Debug)]
pub enum LuaBridgeError {
    #[error("Error in Reflection: {0}")]
    Reflection(#[from] GetComponentReflectError),
    #[error("Error in ECS: {0}")]
    EntityNotExist(#[from] EntityDoesNotExistError),
    #[error("No such component, did you declare it?")]
    NoSuchComponent,
    #[error("Error when converting to Lua: {0}")]
    ConversionToLua(#[from] mlua::Error)
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

///仅用于说明
#[derive(Bundle)]
pub struct TestBundle {
    pub test_component: TestComponent
}
#[derive(Component)]
pub struct TestComponent {
    pub a: i32,
}
impl FromLua for TestBundle {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        Ok(TestBundle {
            test_component: TestComponent {
                a: value.as_integer().unwrap() as i32,
            },
        })
    }
}
pub struct ReflectLuaBundle {
    pub from_lua: fn(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Box<dyn Reflect>>,
    pub default: fn() -> Box<dyn Reflect>,
}
#[derive(Resource)]
pub struct ReflectLuaBundleRegistry {
    bundles: HashMap<TypeId, ReflectLuaBundle>,
}
impl ReflectLuaBundleRegistry {
    pub fn new() -> Self {
        Self {
            bundles: HashMap::new(),
        }
    }
    pub fn register_bundle<T>(&mut self)
    where
        T: Bundle + Reflect + Default + for<'lua> FromLua + 'static
    {
        let from_lua_fn = |value: mlua::Value, lua: &mlua::Lua| -> mlua::Result<Box<dyn Reflect>> {
            let bundle = T::from_lua(value, lua)?;
            Ok(Box::new(bundle))
        };
        let default_fn = || -> Box<dyn Reflect> {
            Box::new(T::default())
        };

        self.bundles.insert(
            TypeId::of::<T>(),
            ReflectLuaBundle { from_lua: from_lua_fn, default: default_fn}
        );
    }
    pub fn get_bundle(&self, type_id: TypeId) -> Option<&ReflectLuaBundle> {
        self.bundles.get(&type_id)
    }
}
mod entity;
mod component;

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
use crate::code_generator::lua_binding::ecs::component::{get_components_fn, handle_get_component_result};
use crate::code_generator::lua_binding::ecs::entity::{despawn_fn, spawn_empty_fn, spawn_reflect_fn};
use crate::code_generator::lua_binding::execution_context::{LuaExecutionContext, LuaReflectInitRegistry};
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
    lua_bundle_registry: Res<LuaReflectInitRegistry>,
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

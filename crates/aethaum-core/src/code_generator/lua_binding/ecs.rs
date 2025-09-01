///这些代码是在设计架构的时候，方便审查，测试用的，后续将会拆分至一系列输出TokenStream的函数用于代码生成
use std::any::{Any, TypeId};
use std::collections::HashMap;
use bevy_ecs::bundle::DynamicBundle;
use bevy_ecs::prelude::*;
use bevy_ecs::reflect::{ReflectBundle, ReflectCommandExt};
use bevy_reflect::prelude::*;
use bevy_reflect::TypeData;
use mlua::FromLua;
use crate::code_generator::lua_binding::execution_context::LuaExecutionContext;
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
    lua_execution_context: Res<LuaExecutionContext>,
    type_registration: Res<AppTypeRegistry>,
    lua_bundle_registry: Res<ReflectLuaBundleRegistry>,
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
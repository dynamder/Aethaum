use std::any::TypeId;
use std::collections::HashMap;
///这些代码是在设计架构的时候，方便审查，测试用的，后续将会拆分至一系列输出TokenStream的函数用于代码生成
use bevy_ecs::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};
use bevy_reflect::Reflect;

#[derive(Resource)]
pub struct LuaExecutionContext {
    pub lua: Arc<Lua>,
}
pub struct LuaReflectInit {
    pub from_lua: fn(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Box<dyn Reflect>>,
    pub default: fn() -> Box<dyn Reflect>,
}
#[derive(Resource)]
pub struct LuaReflectInitRegistry {
    constructors: HashMap<TypeId, LuaReflectInit>,
}
impl LuaReflectInitRegistry {
    pub fn new() -> Self {
        Self {
            constructors: HashMap::new(),
        }
    }
    pub fn register_constructor<T>(&mut self)
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

        self.constructors.insert(
            TypeId::of::<T>(),
            LuaReflectInit { from_lua: from_lua_fn, default: default_fn}
        );
    }
    pub fn get_constructor(&self, type_id: TypeId) -> Option<&LuaReflectInit> {
        self.constructors.get(&type_id)
    }
}
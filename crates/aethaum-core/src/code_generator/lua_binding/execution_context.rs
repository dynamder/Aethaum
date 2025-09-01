
///这些代码是在设计架构的时候，方便审查，测试用的，后续将会拆分至一系列输出TokenStream的函数用于代码生成
use bevy_ecs::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};
#[derive(Resource)]
pub struct LuaExecutionContext {
    pub lua: Arc<Lua>,
}

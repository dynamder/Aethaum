use std::any::Any;
use bevy_ecs::prelude::AppTypeRegistry;

// use bevy_ecs::prelude::*;
// use mlua::prelude::*;
// use std::sync::{Arc, Mutex};
//
// pub struct LuaExecutionContext<'a> {
//     pub lua: &'a Lua,
//     pub world: &'a mut World,
//     pub commands: &'a mut Commands<'a, 'a>,
//     pub query_access: QueryAccess,
// }
//
// pub struct QueryAccess {
//     // 提供对查询结果的安全访问
// }
//
// impl<'a> LuaExecutionContext<'a> {
//     pub fn new(lua: &'a Lua, world: &'a mut World, commands: &'a mut Commands<'a, 'a>) -> Self {
//         Self {
//             lua,
//             world,
//             commands,
//             query_access: QueryAccess::default(),
//         }
//     }
//
//     // 提供组件访问API给Lua
//     pub fn get_component<T: Component>(&mut self, entity: Entity) -> Option<&T> {
//         self.world.get::<T>(entity)
//     }
//
//     pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<Mut<T>> {
//         self.world.get_mut::<T>(entity)
//     }
//
//     // 提供事件发送功能
//     pub fn send_event<T: Event + Clone>(&mut self, event: T) {
//         self.commands.add(move |world: &mut World| {
//             world.send_event(event);
//         });
//     }
// }


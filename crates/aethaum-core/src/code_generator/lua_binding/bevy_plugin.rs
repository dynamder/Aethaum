use mlua::prelude::*;
use bevy_ecs::prelude::*;
use proc_macro2::TokenStream;
use quote::quote;
pub fn test() {
    let lua = Lua::new();
}
pub fn lua_bevy_plugin() -> TokenStream {
    quote! {
        use bevy_ecs::prelude::*;
        use bevy_app::prelude::*;
        use std::sync::{Arc, Mutex};
        use mlua::prelude::*;

        pub struct LuaPlugin;
        impl Plugin for LuaPlugin {
            fn build(&self, app: &mut App) {
                app.init_resource::<LuaRuntime>()
                    .add_systems(Update, execute_lua_systems);
            }
        }
        #[derive(Resource)]
        pub struct LuaRuntime {
            pub lua: Arc<Mutex<Lua>>,
        }
        impl Default for LuaRuntime {
            fn default() -> Self {
                Self {
                    lua: Arc::new(Mutex::new(Lua::new())),
                }
            }
        }
        fn execute_lua_systems(runtime: ResMut<LuaRuntime>) {

        }
    }
}
pub fn lua_script() -> TokenStream {
    quote! {
        pub enum LuaScript {
            Embed(Chunk),
            File(ScriptFile)
        }
        pub struct ScriptFile {
            path: PathBuf,
            content: Chunk
        }
        impl LuaScript {
            pub fn load_embed(script: &str, lua: &Lua) -> Self {
                let chunk = lua.load(script);
                LuaScript::Embed(chunk)
            }
            pub fn load_file(path: &Path, lua: &Lua) -> io::Result<Self> {
                let content = std::fs::read_to_string(path)?;
                let chunk = lua.load(file);
                LuaScript::File(ScriptFile {
                    path: path.into(),
                    content: chunk
                })
            }
        }
    }
}
pub fn lua_script_server() -> TokenStream {
    quote! {
        pub struct LuaScriptServer {
            pub registered: Vec<LuaScript>
        }
    }
}
#[derive(Resource)]
pub struct LuaCommands {

}
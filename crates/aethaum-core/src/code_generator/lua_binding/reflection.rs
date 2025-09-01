use bevy_ecs::prelude::Res;
use bevy_ecs::reflect::AppTypeRegistry;
use bevy_reflect::TypeRegistry;
use proc_macro2::TokenStream;
use quote::quote;

pub fn reflection_token() -> TokenStream {
    get_type_id_by_str_token()
}
pub fn get_type_id_by_str_token() -> TokenStream {
    quote! {
        pub fn get_type_id_by_str(type_name: &str, app_type_registry: Res<AppTypeRegistry>) -> Option<std::any::TypeId> {
            let registry = app_type_registry.read();
            registry.get_with_type_path(type_name)
                .or_else(|| registry.get_with_short_type_path(type_name))
                .map(|registration| registration.type_id())
        }
    }
}
///以下代码是在设计架构的时候，方便审查，测试用的，后续将会拆分至一系列输出TokenStream的函数用于代码生成
pub fn get_type_id_by_str(type_name: &str, type_registry: &TypeRegistry) -> Option<std::any::TypeId> {
    type_registry.get_with_type_path(type_name)
        .or_else(|| type_registry.get_with_short_type_path(type_name))
        .map(|registration| registration.type_id())
}
// 其他反射相关功能


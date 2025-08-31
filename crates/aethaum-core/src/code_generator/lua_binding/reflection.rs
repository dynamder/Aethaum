use bevy_ecs::prelude::Res;
use bevy_ecs::reflect::AppTypeRegistry;
use proc_macro2::TokenStream;
use quote::quote;

pub fn reflection_token() -> TokenStream {
    get_type_id_by_str()
}
pub fn get_type_id_by_str() -> TokenStream {
    quote! {
        pub fn get_type_id_by_str(type_name: &str, app_type_registry: Res<AppTypeRegistry>) -> Option<std::any::TypeId> {
            let registry = app_type_registry.read();
            registry.get_with_type_path(type_name)
                .or_else(|| registry.get_with_short_type_path(type_name))
                .map(|registration| registration.type_id())
        }
    }
}


use proc_macro2::TokenStream;
use quote::quote;

pub fn predefined() -> TokenStream {
    let mut predefined = trait_describe();
    predefined.extend(event_aethaum_spawn_entity());
    predefined
 }
pub fn trait_describe() -> TokenStream {
    quote! {
        pub trait Describe {
            fn describe(&self) -> &'static str {
                 ""
            }
            fn describe_field(&self, field_name: &str) -> &'static str {
                 ""
            }
        }
    }
}
//Reserved Events
pub fn event_aethaum_spawn_entity() -> TokenStream {
    quote! {
        use bevy_ecs::prelude::*;
        use tokio::sync::oneshot;
        use std::sync::atomic::AtomicU64;
        use std::sync::atomic::Ordering;

        // 全局请求ID生成器
        static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

        #[derive(Event)]
        pub struct AethaumSpawnEntity {
            pub prototype_name: String,
            pub request_id: u64,
        }
        #[derive(Event)]
        pub struct AethaumSpawnEntityResponse {
            pub request_id: u64,
            pub entity: Entity,
        }
        impl AethaumSpawnEntity {
            pub fn new(prototype_name: String) -> (Self, u64) {
                let request_id = REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
                (Self {
                    prototype_name,
                    request_id,
                }, request_id)
            }
        }
    }
}

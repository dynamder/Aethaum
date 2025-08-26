use std::collections::HashSet;
use std::fmt::format;
use convert_case::Casing;
use quote::quote;
use crate::toml_parser::parsed::{Component, Describable, EntityProto, Event, Field, System, SystemEventHandler, SystemQuery, SystemUpdate};
use proc_macro2::{Span, TokenStream};
use syn::Ident;
use thiserror::Error;
use crate::ecs::module::EcsModule;

#[derive(Debug,Error)]
pub enum TranspileError {
    #[error("Error to write generated code, {0}")]
    WriteError(#[from] core::fmt::Error),
    #[error("Error to format generated code, {0}")]
    FormatError(#[from] syn::Error),
    #[error("Multiple errors occurred during transpiling:\n{}",
        .errors.iter().map(|e| format!("  - {}", e)).collect::<Vec<_>>().join("\n"))]
    Multiple {
        errors: Vec<TranspileError>,
    }
}

pub trait Transpile {
    fn transpile(&self) -> Result<TokenStream, TranspileError>;
    fn transpile_into(&self, output: &mut TokenStream) -> Result<(), TranspileError> {
        output.extend(self.transpile()?);
        Ok(())
    }
}
fn transpile_fields<T, FieldIter>(fields: FieldIter) -> impl Iterator<Item = TokenStream>
where
    T: Field,
    FieldIter: IntoIterator<Item = T>,
{
    fields.into_iter().map(|field| {
        let field_name = field.name_as_rust_ident();
        let field_type = field.type_as_rust_ident();
        quote! {
            pub #field_name: #field_type,
        }
    })
}
fn transpile_descriptions<T: Describable>(to_transpile: &T, name: &str) -> TokenStream {
    let struct_desc = to_transpile.description()
        .map(|d| {
            quote! { #d }
        })
        .unwrap_or_else(|| quote! { "" });

    let field_desc_impl = if let Some(fields) = to_transpile.field_description() {
        let field_matches = fields.map(|(field_name, desc)| {
            quote! {
                stringify!(#field_name) => #desc,
            }
        }).collect::<Vec<_>>();

        if !field_matches.is_empty() {
            quote! {
                    match field_name {
                        #(#field_matches)*
                        _ => "",
                    }
                }
        } else {
            quote! { "" }
        }
    } else {
        quote! { "" }
    };

    let name = Ident::new(name, Span::call_site());

    quote! {
        impl Describe for #name {
            fn describe(&self) -> &'static str {
                #struct_desc
            }

            fn describe_field(&self, field_name: &str) -> &'static str {
                #field_desc_impl
            }
        }
    }
}
impl Transpile for Component {
    fn transpile(&self) -> Result<TokenStream, TranspileError> {
        let name = Ident::new(self.name.as_str(), Span::call_site());
        let fields = if let Some(fields) = &self.fields {
            transpile_fields(fields).collect()
        } else {
            vec![]
        };
        // 生成 Default 实现（如果有默认值的话）
        let default_impl = if let Some(fields) = &self.fields {
            if fields.iter().any(|f| f.default_value.is_some()) {
                let default_fields = fields.iter().map(|field| {
                    let field_name = Ident::new(field.name.as_str(), Span::call_site());
                    if let Some(default_value) = &field.default_value {
                        // 将 TOML 值转换为 Rust 字面量
                        let default_literal = match default_value {
                            toml::Value::Boolean(b) => quote! { #b },
                            toml::Value::Integer(i) => quote! { #i },
                            toml::Value::Float(f) => quote! { #f },
                            toml::Value::String(s) => quote! { #s },
                            // 其他类型需要进一步处理
                            _ => quote! { Default::default() },
                        };
                        quote! { #field_name: #default_literal }
                    } else {
                        quote! { #field_name: Default::default() }
                    }
                }).collect::<Vec<_>>();

                quote! {
                    impl Default for #name {
                        fn default() -> Self {
                            Self {
                                #(#default_fields),*
                            }
                        }
                    }
                }
            } else {
                quote! {}
            }
        } else {
            quote! {}
        };
        //生成Describe trait
        let description_impl = transpile_descriptions(self,self.name.as_str());

        Ok(quote! {
            #[derive(Component, Reflect)]
            pub struct #name {
                #(#fields)*
            }

            #default_impl

            #description_impl
        })
    }
}
impl Transpile for Event {
    fn transpile(&self) -> Result<TokenStream, TranspileError> {
        let name = Ident::new(self.name.as_str(), Span::call_site());
        let fields = if let Some( fields) = self.fields.as_ref() {
            transpile_fields(fields).collect()
        } else {
            vec![]
        };
        let description_impl = transpile_descriptions(self, self.name.as_str());

        Ok(
            quote! {
                #[derive(Event)]
                pub struct #name {
                    #(#fields)*
                }

                #description_impl
            }
        )
    }
}
impl Transpile for EntityProto {
    fn transpile(&self) -> Result<TokenStream, TranspileError> {
        let name = Ident::new(self.name.as_str(), Span::call_site());
        let bundle_name = Ident::new(&format!("{}Bundle", self.name), Span::call_site());
        let spawn_system_name = Ident::new(&format!("spawn_{}_system", self.name.to_lowercase()), Span::call_site());

        // 生成 Bundle 字段
        let mut errors = Vec::new();

        let bundle_fields = self.components.iter().map(|component_ref| {
            let component_name = Ident::new(component_ref.name.as_str(), Span::call_site());
            let component_type_str = match &component_ref.module_name {
                Some(module_name) => format!("{}::components::{}", module_name, component_name),
                None => format!("components::{}", component_name),
            };
            let component_type = syn::parse_str::<syn::Type>(&component_type_str);
            match component_type {
                Ok(component_type) => {
                    quote! {
                        #component_name: #component_type,
                    }
                },
                Err(err) => {
                    errors.push(
                        TranspileError::FormatError(err)
                    );
                    quote! {}
                }
            }
        }).collect::<Vec<_>>();
        if !errors.is_empty() {
            if errors.len() == 1 {
                return Err(errors.pop().unwrap());
            }else {
                return Err(TranspileError::Multiple{errors});
            }
        }
        
        // 生成描述实现
        let description_impl = transpile_descriptions(self, self.name.as_str());

        Ok(quote! {
            #[derive(Bundle, Default)]
            pub struct #bundle_name {
                #(#bundle_fields)*
            }

            pub struct #name;

            impl #name {
                pub fn bundle() -> #bundle_name {
                    #bundle_name::default()
                }

                pub fn spawn(commands: &mut Commands) -> Entity {
                    commands.spawn(Self::bundle()).id()
                }
            }

            // 为这个原型生成对应的处理系统
            pub fn #spawn_system_name(
                mut events: EventReader<AethaumSpawnEntity>,
                mut spawn_responses: EventWriter<AethaumSpawnEntityResponse>,
                mut commands: Commands,
            ) {
                for event in events.read() {
                    if event.prototype_name == stringify!(#name) {
                        let entity = #name::spawn(&mut commands);
                        // 发送响应事件
                        spawn_responses.send(AethaumSpawnEntityResponse {
                            request_id: event.request_id,
                            entity,
                        });
                    }
                }
            }

            #description_impl
        })
    }
}
impl Transpile for SystemQuery {
    fn transpile(&self) -> Result<TokenStream, TranspileError> {
        let mut filters = {
            let mut filters = Vec::new();

            // 处理 With 过滤器（包含的组件）
            if let Some(include_components) = self.component_constraint.get_include() {
                for component_ref in include_components {
                    let component_module_path = match &component_ref.module_name {
                        Some(module_name) => format!("{}::components::{}", module_name, component_ref.name),
                        None => format!("components::{}", component_ref.name),
                    };
                    let component_name = syn::parse_str::<syn::Type>(&component_module_path)?;
                    filters.push(quote! { With<#component_name> });
                }
            }

            // 处理 Without 过滤器（排除的组件）
            if let Some(exclude_components) = self.component_constraint.get_exclude() {
                for component_ref in exclude_components {
                    let component_module_path = match &component_ref.module_name {
                        Some(module_name) => format!("{}::components::{}", module_name, component_ref.name),
                        None => format!("components::{}", component_ref.name),
                    };
                    let component_name = syn::parse_str::<syn::Type>(&component_module_path)?;
                    filters.push(quote! { Without<#component_name> });
                }
            }
            filters
        };
        match filters.len() {
            0 => Ok(quote! { Query<Entity> }),
            1 => {
                let filter = filters.pop().unwrap();
                Ok(quote! { Query<Entity, #filter> })
            },
            _ => {
                Ok(quote! {
                    Query<Entity, (#(#filters),*)>
                })
            }
        }
    }
}
impl Transpile for System {
    fn transpile(&self) -> Result<TokenStream, TranspileError> {
        let system_name = Ident::new(self.normal.name.as_str(), Span::call_site());

        // 生成查询参数
        let queries = self.queries.iter()
            .map(|query| query.transpile().unwrap())
            .collect::<Vec<_>>();

        let query_params = self.queries.iter()
            .map(|s_query| Ident::new(s_query.name.as_str(), Span::call_site()))
            .collect::<Vec<_>>();

        // 生成 update 系统（如果存在）
        let update_system = if let Some(update) = &self.update {
            let update_system_name = Ident::new("update", Span::call_site());
            quote! {
                pub fn #update_system_name(
                    mut commands: Commands,
                    #(#query_params: #queries,)*
                ) {
                    // Update logic would go here
                }
            }
        } else {
            quote! {}
        };
        let mut errors = Vec::new();

        // 生成事件处理系统
        let event_handler_systems = self.event_handlers.iter()
            .map(|event_handler| {
                let handler_system_name = Ident::new(
                    &format!("{}_on_{}",
                             self.normal.name.to_lowercase(),
                             event_handler.watch_for.name.as_str().to_lowercase()),
                    Span::call_site()
                );
                let event_module_path = match &event_handler.watch_for.module_name {
                    Some(module_name) => format!("{}::events::{}", module_name, event_handler.watch_for),
                    None => format!("events::{}", event_handler.watch_for),
                };
                let event_type = syn::parse_str::<syn::Type>(&event_module_path);
                match event_type {
                    Ok(event_type) => {
                        quote! {
                            pub fn #handler_system_name(
                                mut commands: Commands,
                                mut event_reader: EventReader<#event_type>,
                                #(#query_params: #queries,)*
                            ) {
                                for event in event_reader.read() {
                                    // Event handling logic would go here
                                }
                            }
                        }
                    },
                    Err(err) => {
                        errors.push(
                            TranspileError::FormatError(err)
                        );
                        quote! {}
                    }
                }
            })
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            if errors.len() == 1 {
                return Err(errors.pop().unwrap()); //ROBUST: must have one element
            } else {
                return Err(TranspileError::Multiple{errors});
            }
        }

        let description_impl = transpile_descriptions(self, self.normal.name.as_str());

        Ok(quote! {
            pub struct #system_name;

            impl #system_name {
                #update_system

                #(#event_handler_systems)*
            }

            #description_impl
        })
    }
}
impl Transpile for EcsModule {
    fn transpile(&self) -> Result<TokenStream, TranspileError> {
        let mut external_module = vec![];
        let mut recorded_external_modules = HashSet::new();
        //components
        let (components_token, components_to_register) = if let Some(components) = &self.components {
            components.iter()
                .map(|component| {
                    let component_name = Ident::new(component.name.as_str(), Span::call_site());
                    (component.transpile().unwrap(),quote! {components::#component_name})
                })
                .collect::<(Vec<_>,Vec<_>)>()
            //ROBUST: this transpile could never fail
        }else {
            (vec![],vec![])
        };
        //events
        let (events_token, events_to_register) = if let Some(events) = &self.events {
            events.iter()
                .map(|event| {
                    let event_name = Ident::new(event.name.as_str(), Span::call_site());
                    (event.transpile().unwrap(),quote! {events::#event_name})
                })
                .collect::<(Vec<_>,Vec<_>)>()
        }else {
            (vec![],vec![])
        };
        //entity_protos
        let (entity_proto_token,entity_protos_to_register) = if let Some(entity_protos) = &self.entity_protos {
            entity_protos.iter()
                .map(|entity_prototype| {
                    entity_prototype.components.iter()
                        .for_each(|component_ref| {
                            if let Some(module_name) = &component_ref.module_name {
                                if module_name.as_str() != self.name.as_str() && recorded_external_modules.insert(module_name.as_str()){
                                    let extern_module = Ident::new(module_name.as_str(), Span::call_site());
                                    external_module.push(
                                        quote! {use crate::modules::#extern_module;}
                                    );
                                }
                            }
                        });
                    let spawn_entity_system = format!("spawn_{}_system", entity_prototype.name.to_lowercase());
                    let spawn_entity_system = Ident::new(spawn_entity_system.as_str(), Span::call_site());
                    (entity_prototype.transpile().unwrap(),quote! {entity_protos::#spawn_entity_system})
                })
                .collect::<(Vec<_>,Vec<_>)>()
        }else {
            (vec![],vec![])
        };

        //systems
        let mut errors = vec![];
        let mut systems_to_register = vec![];
        let systems_token = if let Some(systems) = &self.systems {
            systems.iter()
                .map(|system| {
                    system.queries.iter()
                    .for_each(|query| {
                        query.component_constraint.chained_iter()
                            .for_each(|component_ref| {
                                if let Some(module_name) = &component_ref.module_name {
                                    if module_name.as_str() != self.name.as_str() && recorded_external_modules.insert(module_name.as_str()){
                                        let extern_module = Ident::new(module_name.as_str(), Span::call_site());
                                        external_module.push(
                                            quote! {use crate::modules::#extern_module;}
                                        );
                                    }
                                }
                            })
                    });
                    //record system names for bevy registering
                    let system_ident = Ident::new(system.normal.name.as_str(), Span::call_site());
                    let update_system_ident = {

                        quote! {systems::#system_ident::update}
                    };
                    systems_to_register.push(update_system_ident);
                    let system_event_handlers_ident = system.event_handlers.iter()
                        .map(|event_handler| {
                            let event_handler_str = format!("{}_on_{}", system.normal.name.as_str().to_lowercase(), event_handler.watch_for.name.as_str().to_lowercase());
                            let system_event_handler_ident = Ident::new(event_handler_str.as_str(), Span::call_site());
                            quote! {systems::#system_ident::#system_event_handler_ident}
                        });
                    systems_to_register.extend(system_event_handlers_ident);
                    //do the transpile
                    match system.transpile() {
                        Ok(token) => token,
                        Err(err) => {
                            errors.push(err);
                            quote! {}
                        }
                    }
                })
                .collect::<Vec<_>>()
        }else {
            vec![]
        };
        if !errors.is_empty() {
            if errors.len() == 1 {
                return Err(errors.pop().unwrap()); //ROBUST: must have one element
            }else {
                return Err(TranspileError::Multiple { errors});
            }
        }
        let module_name = Ident::new(&self.name, Span::call_site());
        let plugin_name = Ident::new(&format!("{}Plugin", self.name.as_str()), Span::call_site());

        //Plugin registration tokens
        let components_registration = if !components_to_register.is_empty() {
            quote! {
                #(
                    app.register_type::<#components_to_register>();
                )*
            }
        }else {
            quote! {}
        };
        let events_registration = if !events_to_register.is_empty() {
            quote! {
                #(
                    app.add_event::<#events_to_register>();
                )*
            }
        }else {
            quote! {}
        };
        let systems_registration = if !systems_to_register.is_empty() {
            quote! {
                app.add_systems(Update, (#(#systems_to_register),*));
            }
        }else {
            quote! {}
        };
        let entity_protos_registration = if !entity_protos_to_register.is_empty() {
            quote! {
                #(
                    app.add_systems(Update, #entity_protos_to_register);
                )*
            }
        }else {
            quote! {}
        };

        Ok(
            quote! {
                //! Auto-generated by Aethaum
                use bevy_ecs::prelude::*;
                use bevy_app::{Plugin, App, Update};
                use bevy_reflect::Reflect;
                use crate::aethaum_predefined::*;
                #(#external_module)*

                pub mod components {
                    use super::*;
                    #(#components_token)*
                }

                pub mod events {
                    use super::*;
                    #(#events_token)*
                }
                pub mod entity_protos {
                    use super::*;
                    #(#entity_proto_token)*
                }
                pub mod systems {
                    use super::*;
                    #(#systems_token)*
                }
                pub struct #plugin_name;

                impl Plugin for #plugin_name {
                    fn build(&self, app: &mut App) {
                        // 注册组件
                        #components_registration

                        // 注册事件
                        #events_registration

                        // 注册系统组
                        #systems_registration

                        // 注册实体原型系统
                        #entity_protos_registration
                    }
                }
            }
        )
    }
}


#[cfg(test)]
mod tests {
    use smart_string::SmartString;
    use crate::code_generator::utils::format_rust_code;
    use crate::ecs::loader::ModuleFileLoader;
    use crate::toml_parser::parsed::{AethaumType, ComponentConstraint, ComponentField, ComponentRef, EventField, EventRef, PrimitiveType, SystemNormal};
    use super::*;
    #[test]
    fn test_transpile_component() {
        let component = Component {
            name: SmartString::from("TestComponent".to_string()),
            description: Some(SmartString::from("This is a test component".to_string())),
            fields: Some(vec![
                ComponentField {
                    name: SmartString::from("test_field".to_string()),
                    type_spec: AethaumType::Primitive(PrimitiveType::Bool),
                    default_value: Some(toml::Value::Boolean(true)),
                    description: Some(SmartString::from("This is a test field".to_string())),
                },
                ComponentField {
                    name: SmartString::from("test_field2".to_string()),
                    type_spec: AethaumType::Primitive(PrimitiveType::Int),
                    default_value: None,
                    description: None,
                },
            ]),
        };
        let transpiled = component.transpile().unwrap();
        let transpiled = format_rust_code(transpiled).unwrap();
        println!("{}", transpiled);
        let parsed_result = syn::parse_str::<syn::File>(&transpiled);
        assert!(parsed_result.is_ok(), "Generated code has syntax errors: {:?}", parsed_result.err());
    }
    #[test]
    fn test_transpile_event() {
        let event = Event {
            name: SmartString::from("click"),
            description: Some("Click event".into()),
            fields: Option::from(vec![
                EventField {
                    name: SmartString::from("target"),
                    description: Some("The element that was clicked".into()),
                    type_spec: AethaumType::Primitive(PrimitiveType::Str),
                },
                EventField {
                    name: SmartString::from("value"),
                    description: None,
                    type_spec: AethaumType::Primitive(PrimitiveType::Int),
                },
            ]),
        };
        let transpiled = event.transpile().unwrap();
        println!("{}", transpiled);
        let transpiled = format_rust_code(transpiled).unwrap();
        println!("{}", transpiled);
        let parsed_result = syn::parse_str::<syn::File>(&transpiled);
        assert!(parsed_result.is_ok(), "Generated code has syntax errors: {:?}", parsed_result.err());
    }
    #[test]
    fn test_transpile_entity_protos() {
        let event = EntityProto {
            name: "TestEntity".into(),
            description: Some("This is a test entity".into()),
            components: vec![
                ComponentRef::new(None::<&str>, "position"),
                ComponentRef::new(Some("TestComponent"), "test_component")
            ]
        };
        let transpiled = event.transpile().unwrap();
        println!("{}", transpiled);
        let transpiled = format_rust_code(transpiled).unwrap();
        println!("{}", transpiled);
        let parsed_result = syn::parse_str::<syn::File>(&transpiled);
        assert!(parsed_result.is_ok(), "Generated code has syntax errors: {:?}", parsed_result.err());
    }
    #[test]
    fn test_transpile_system() {
        let system = System {
           normal: SystemNormal {
               name: "TestSystem".into(),
               description: Some("This is a test system".into()),
               category: None,
               priority: None,
           },
            queries: vec![
                SystemQuery {
                    name: SmartString::from("TestQuery"),
                    description: None,
                    component_constraint: ComponentConstraint::new_empty().with_include(
                        vec![ComponentRef::new(Some("TestComponent"), "test_component")]
                    ).with_exclude(
                        vec![ComponentRef::new(Some("TestComponent222"), "test_component")]
                    )
                }
            ],
            update: Some(SystemUpdate {
                interval: Default::default(),
                condition: None,
                logic: None,
            }),
            event_handlers: vec![
                SystemEventHandler {
                    watch_for: EventRef::new(None::<&str>, "click"),
                    priority: 0,
                    logic: None,
                }
            ]
        };
        let transpiled = system.transpile().unwrap();
        println!("{}", transpiled);
        let transpiled = format_rust_code(transpiled).unwrap();
        println!("{}", transpiled);
        let parsed_result = syn::parse_str::<syn::File>(&transpiled);
        assert!(parsed_result.is_ok(), "Generated code has syntax errors: {:?}", parsed_result.err());
    }
    #[test]
    fn test_transpile_module() {
        let module = ModuleFileLoader::new(
            r#"D:\Aethaum\test_project\modules\explore"#.into(),
            "explore".into()
        ).load().unwrap();
        let transpiled = module.transpile().unwrap();
        println!("{}", transpiled);
        let transpiled = format_rust_code(transpiled).unwrap();
        println!("{}", transpiled);
        let parsed_result = syn::parse_str::<syn::File>(&transpiled);
        assert!(parsed_result.is_ok(), "Generated code has syntax errors: {:?}", parsed_result.err());
    }
}

use thiserror::Error;
use crate::ecs::module::AethaumProject;
use std::fs;
use std::path::{Path, PathBuf};
use quote::quote;
use crate::code_generator::aethaum_predefined;
use crate::code_generator::transpile::Transpile;
use crate::code_generator::utils::format_rust_code;

/// 项目生成模块
/// 生成的项目架构示例：
/// project/
/// ├── Cargo.toml
/// ├── src/
/// │   ├── main.rs
/// │   ├── modules.rs
/// │   ├── modules/
/// │   │   ├── player.rs
/// │   │   ├── combat.rs
/// │   │   └── ui.rs
/// │   ├── lua_bindings.rs
/// │   └── lib.rs
/// ├── assets/
/// │   ├── modules/
/// │   │   ├── player/
/// │   │   │   └── scripts/
/// │   │   ├── combat/
/// │   │   │   └── scripts/
/// │   │   └── ui/
/// │   │       └── scripts/
/// │   └── scripts/
/// └── config/
#[derive(Debug, Error)]
pub enum TemplateGenerationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to create directory: {0}")]
    DirectoryCreationError(PathBuf),
}

pub struct ProjectTemplateGenerator;

impl ProjectTemplateGenerator {
    pub fn generate(project_path: &Path, project: &AethaumProject) -> Result<(), TemplateGenerationError> {
        // 创建项目根目录
        fs::create_dir_all(project_path)?;

        // 构建文件夹结构
        Self::build_folder_structure(project_path, project)?;

        // 生成 Cargo.toml
        Self::generate_cargo_toml(project_path, project)?;

        // 生成 src 目录下的文件
        Self::generate_source_files(project_path, project)?;

        // 生成 assets 目录结构
        Self::generate_assets_structure(project_path, project)?;

        // 生成 config 目录
        Self::generate_config_structure(project_path)?;

        Ok(())
    }

    fn build_folder_structure(project_path: &Path, project: &AethaumProject) -> Result<(), TemplateGenerationError> {
        // 创建根目录
        let root_dirs: [PathBuf;3] = ["src".into(), "assets".into(), "config".into()];
        for dir in &root_dirs {
            let full_path = project_path.join(dir);
            fs::create_dir_all(&full_path)?;
        }

        // 创建 src/modules 目录
        let src_modules_path = project_path.join("src").join("modules");
        fs::create_dir_all(&src_modules_path)?;

        // 为每个模块创建 assets 目录
        for module in project.module_tree.get_modules() {
            let module_name = module.name.as_str();
            let module_assets_path = project_path.join("assets").join("modules").join(module_name);
            fs::create_dir_all(&module_assets_path)?;
        }

        // 创建 assets/scripts 目录
        let scripts_path = project_path.join("assets").join("scripts");
        fs::create_dir_all(&scripts_path)?;

        Ok(())
    }

    fn generate_cargo_toml(project_path: &Path, project: &AethaumProject) -> Result<(), TemplateGenerationError> {
        let cargo_toml_content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
mlua = {{ version = "0.11.2", features = ["lua54", "vendored"] }}
bevy_ecs = "0.16.1"
bevy_app = "0.16.1"
smart-string = {{ version = "0.1.3", features = ["serde"]}}
itertools = "0.14.0"
one-or-many = "0.4.0"
miette = "7.6.0"
thiserror = "2.0.16"
            "#,
            project.world.normal.name.as_str()
        );

        let cargo_toml_path = project_path.join("Cargo.toml");
        fs::write(cargo_toml_path, cargo_toml_content)?;

        Ok(())
    }

    fn generate_source_files(project_path: &Path, project: &AethaumProject) -> Result<(), TemplateGenerationError> {
        // 生成 main.rs
        let main_rs_content = quote! {
            use bevy::prelude::*;

            mod modules;
            mod lua_bindings;
            mod aethaum_predefined;

            fn main() {
                App::new()
                    .add_plugins(DefaultPlugins)
                    .add_plugins(lua_bindings::LuaPlugin)
                    .run();
            }
        };

        let main_rs_path = project_path.join("src").join("main.rs");
        println!("formated: \n {}", format_rust_code(main_rs_content.clone()).unwrap());
        fs::write(main_rs_path, format_rust_code(main_rs_content).unwrap())?;

        // 生成 lib.rs
        let lib_rs_content = quote! {
            pub mod modules;
            pub mod lua_bindings;
            pub mod aethaum_predefined;
        };

        let lib_rs_path = project_path.join("src").join("lib.rs");
        fs::write(lib_rs_path, format_rust_code(lib_rs_content).unwrap())?;

        let src_aethaum_predefined_path = project_path.join("src").join("aethaum_predefined.rs");
        let predefined = aethaum_predefined::predefined();
        fs::write(src_aethaum_predefined_path, format_rust_code(predefined).unwrap())?;

        // 生成 modules.rs
        let mut module_declarations = Vec::new();
        for module in project.module_tree.get_modules() {
            let module_name = proc_macro2::Ident::new(module.name.as_str(), proc_macro2::Span::call_site());
            module_declarations.push(quote! {
                pub mod #module_name;
            });
        }

        let modules_content = quote! {
            //! Aethaum modules
            #(#module_declarations)*
        };

        let modules_path = project_path.join("src").join("modules.rs");
        fs::write(modules_path, format_rust_code(modules_content).unwrap())?;

        // 为每个模块生成单个文件
        for module in project.module_tree.get_modules() {
            let module_name = &module.name;

            // 生成模块文件，包含所有组件类型
            let module_ident = proc_macro2::Ident::new(module_name, proc_macro2::Span::call_site());

            let module_content = quote! {
                //! Module: #module_ident
                //! Auto-generated by Aethaum
            };

            let module_file_path = project_path.join("src").join("modules").join(format!("{}.rs", module_name));
            fs::write(module_file_path, format_rust_code(module_content).unwrap())?;
        }

        // 生成 lua_bindings.rs
        let lua_bindings_content = quote! {
            //! Lua bindings
            //! Auto-generated by Aethaum
        };

        let lua_bindings_path = project_path.join("src").join("lua_bindings.rs");
        fs::write(lua_bindings_path, format_rust_code(lua_bindings_content).unwrap())?;

        Ok(())
    }

    fn generate_assets_structure(project_path: &Path, project: &AethaumProject) -> Result<(), TemplateGenerationError> {
        // 为每个模块创建 scripts 目录
        for module in &project.module_tree.get_modules() {
            let module_name = module.name.as_str();
            let module_scripts_path = project_path.join("assets").join("modules").join(module_name).join("scripts");
            fs::create_dir_all(&module_scripts_path)?;
        }

        Ok(())
    }

    fn generate_config_structure(project_path: &Path) -> Result<(), TemplateGenerationError> {
        // 创建一个示例配置文件
        let example_config_content = r#"# Aethaum Configuration File

[engine]
tick_rate = 60

[logging]
level = "info"
        "#;

        let config_path = project_path.join("config").join("config.toml");
        fs::write(config_path, example_config_content)?;

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use crate::ecs::loader::ProjectLoader;
    use super::*;
    #[test]
    fn test_generate_project_template() {
        let project_path = Path::new(r#"D:\Aethaum\test_project_template/test"#);
        let project = ProjectLoader::new(r#"D:\Aethaum\test_project"#.into()).load().unwrap();
        ProjectTemplateGenerator::generate(project_path, &project).unwrap();
    }
}
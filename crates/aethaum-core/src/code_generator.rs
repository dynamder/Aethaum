mod transpile;
mod lua_binding;
mod aethaum_predefined;
mod utils;
mod project_template;

use std::fs;
use std::path::Path;
use bevy_ecs::event::Events;
use proc_macro2::TokenStream;
use thiserror::Error;
use crate::code_generator::project_template::ProjectTemplateGenerator;
use crate::code_generator::transpile::Transpile;
use crate::ecs::module::AethaumProject;

#[derive(Error, Debug)]
pub enum CodeGenerationError {
    #[error("IO error when generating code: {0}")]
    Io(#[from] std::io::Error),
    #[error("Transpile error when generating code: {0}")]
    Transpile(#[from] transpile::TranspileError),
    #[error("Template Generation error when generating code: {0}")]
    TemplateGeneration(#[from] project_template::TemplateGenerationError),
}
pub struct CodeGenerator {
    project: AethaumProject
}
impl CodeGenerator {
    pub fn new(project: AethaumProject) -> Self {
        Self {
            project
        }
    }
    pub fn generate(&self) -> Result<(), CodeGenerationError> {
        let generated_root = self.project.root.join("generated");
        ProjectTemplateGenerator::generate(&generated_root, &self.project)?;
        for module in self.project.module_tree.get_modules() {
            let module_path = generated_root.join("src").join("modules").join(format!("{}.rs", module.name));
            let module_code = module.transpile()?;
            Self::write_code_to_file(&module_path, module_code)?;
        }
        Ok(())
    }
    fn write_code_to_file(path: &Path, content: TokenStream) -> Result<(), CodeGenerationError> {
        let formatted_code = utils::format_rust_code(content)?;
        fs::write(path, formatted_code)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use crate::code_generator::CodeGenerator;
    #[test]
    fn test_generate_code() {
        let project = crate::ecs::loader::ProjectLoader::new("D:\\Aethaum\\test_project".into()).load().unwrap();
        let code_generator = CodeGenerator::new(project);
        code_generator.generate().unwrap();
    }
}
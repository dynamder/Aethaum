use proc_macro2::TokenStream;
use crate::code_generator::transpile::TranspileError;

pub fn format_rust_code(stream : TokenStream) -> Result<String, TranspileError> {
    let syntax_tree = syn::parse2::<syn::File>(stream)?;
    Ok(prettyplease::unparse(&syntax_tree))
}
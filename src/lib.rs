use std::fs::read_to_string;

use naga::{
    back::spv::{self, Options},
    front::wgsl,
    valid::{Capabilities, ValidationFlags, Validator},
};
use quote::quote;
use syn::{parse_macro_input, LitStr};

extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn load_wgsl_shader(path: TokenStream) -> TokenStream {
    let path = parse_macro_input!(path as LitStr);
    let shader_str = read_to_string(&path.value()).expect("Failed to read shader file");
    let shader = wgsl::parse_str(&shader_str).expect("Failed to parse file into shader");
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::default());
    let mod_info = validator.validate(&shader).expect("Invalid shader");
    let options = Options {
        lang_version: (1, 6),
        ..Default::default()
    };
    let out =
        spv::write_vec(&shader, &mod_info, &options, None).expect("Failed to create spir-v shader");
    quote! {
        vec![ #( #out ), *]
    }
    .into()
}

extern crate proc_macro;
use quote::quote;
use proc_macro::TokenStream;
use syn::{parse::{Error, ParseStream}, parse_macro_input, LitStr};

#[proc_macro]
pub fn module_version(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as LitStr);

    let major: LitStr = input.parse()?;
    input.parse::<Token![.]>()?;
    let minor: LitStr = input.parse()?;
    input.parse::<Token![.]>()?;
    let patch: LitStr = input.parse()?;



    let value = quote! {
        vlcb_module::ModuleVersion::new(major, minor, patch)
    };

    TokenStream::from(value)
}
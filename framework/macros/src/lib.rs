extern crate proc_macro;
use quote::quote;
use proc_macro::TokenStream;
use syn::{parse::{Error, ParseStream}, parse_macro_input, LitStr};

/// Helper macro for converting string to
/// a module name and ensuring it has the correct length
// #[macro_export]
// macro_rules! module_name {
//     ($s:expr) => {{
//         let mut array = [b' '; MODULE_NAME_LEN];
//         let bytes = $s.as_bytes();
//         for i in 0..bytes.len().min(7) {
//             array[i] = bytes[i];
//         }
//         array
//     }};
// }

// /// Helper macro for converting string to
// /// a module name and ensuring it has the correct length
// /// and format
// #[proc_macro]
// pub fn module_name(item: TokenStream) -> TokenStream {
//     // TODO: make sure the string is padded, has length of max 7(excl the prefix CAN|ETH)

//     let input = parse_macro_input!(item as LitStr);


//     let value = input.value();
//     // todo: this stripping code should probably be shared somewhere because
//     // it will be easier to maintain
//     let prefixes = ["CAN", "ETH"];

//     let found_prefix = prefixes.iter().find(|v| value.to_uppercase().starts_with(*v));

//     let sanitized_val;
//     // strip the prefix length from the literal
//     if let Some(prefix) = found_prefix {
//         sanitized_val = &value[prefix.len()..];
//     } else {
//         sanitized_val = &value;
//     }

//     if sanitized_val.len() > 7 {
//         return Error::new_spanned(
//             input,
//             "The name literal should be maximum of 7 characters long (this excludes prefixes 'CAN', 'ETH')"
//         ).to_compile_error().into();
//     }

//     let value = value.pad_to_width_with_char(value.len() + (7 - sanitized_val.len()), ' ');

//     let value = quote! {
//         #value
//     };

//     TokenStream::from(value)
// }

#[proc_macro]
pub fn str_to_array(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let value = input.value();

    let chars: Vec<char> = value.chars().collect();

    let expanded = quote! {
        [#(#chars),*]
    };

    TokenStream::from(expanded)
}
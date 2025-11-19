use proc_macro::TokenStream;
use quote::{ToTokens, quote};

#[proc_macro_attribute]
pub fn enum_meta(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemEnum);
    let enum_name = &input.ident;
    let variants = &input.variants;

    let variant_names: Vec<String> = variants.iter().map(|v| v.ident.to_string()).collect();

    let expanded = quote! {
        #input

        impl #enum_name {
            pub fn variants() -> &'static [&'static str] {
                &[#(#variant_names),*]
            }
        }
    };

    expanded.into()
}

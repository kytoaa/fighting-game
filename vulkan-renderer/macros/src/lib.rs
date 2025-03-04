use image::get_raw_image_data_from_file;
use proc_macro::TokenStream;

#[proc_macro]
pub fn my_proc_macro(input: TokenStream) -> TokenStream {
    let ast: syn::Lit = syn::parse(input).unwrap();
    let s = match ast {
        syn::Lit::Str(s) => s,
        _ => panic!("must be a string literal"),
    };

    get_raw_image_data_from_file(s);

    todo!();
}

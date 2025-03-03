use proc_macro::TokenStream;

#[proc_macro]
pub fn my_proc_macro(input: TokenStream) -> TokenStream {
    TokenStream::new()
}

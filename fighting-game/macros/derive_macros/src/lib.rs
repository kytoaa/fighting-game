use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Builder, attributes(no_builder))]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_builder(&ast)
}

fn impl_builder(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            struct_token: _,
            fields,
            semi_token: _,
        }) => fields,
        _ => panic!("not a named field struct"),
    };

    let functions = fields.iter().map(|field| {
        if field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("no_builder"))
            .is_some()
        {
            return quote! {};
        }
        let ident = &field.ident.as_ref().expect("not a named field struct");
        let ty = &field.ty;
        quote! {
            pub const fn #ident(mut self, val: #ty) -> Self {
                self.#ident = val;
                self
            }
        }
    });

    let src = quote! {
        impl Builder for #name {}
        impl #name {
            #(#functions)*
        }
    };

    src.into()
}

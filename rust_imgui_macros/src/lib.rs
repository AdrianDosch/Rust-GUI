extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Callback)]
pub fn callback_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_callback(&ast)
}

fn impl_callback(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Callback for #name {
            fn set_callback(&mut self, callback: impl Fn() + 'static){
                self.callback = Box::new(callback);
            }
            fn call_callback(&self) {
                (self.callback)();
            }
        }
    };
    gen.into()
}
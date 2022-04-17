extern crate proc_macro;
extern crate syn;

use proc_macro::{TokenStream};
use quote::{quote, format_ident};
use syn::{Fields, Data, Ident};

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

#[proc_macro_derive(ImgGuiGlue)]
pub fn imgGuiGlue_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_imgGuiGlue(&ast)
}

fn impl_imgGuiGlue(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = &ast.data;

    let fields = match fields {
        Data::Struct(s) => &s.fields,
        _ => unimplemented!(),
    };

    let named = match fields {
        Fields::Named(named) => named,
        _ => unimplemented!(),
    };

    let members = named.named.clone();
    let mut contains_value = false;
    let mut contains_callback = false;
    for member in members {
        // contains = Some(member);
        match member.ident {
            Some(i) => {
                if i == Ident::new("value", syn::__private::Span::call_site()) {
                    contains_value = true;
                } else if i == Ident::new("callback", syn::__private::Span::call_site()) {
                    contains_callback = true;
                }
            },
            None => unimplemented!()
        }
        
    }

    let imgui_fn = format_ident!("ImGui_{}", name);
    
    let gen;
    if contains_callback && contains_value {
        gen = quote! {
            impl ImgGuiGlue for #name {
                fn render(&self) {
                    let mut label = self.label.clone();
                    label.push('\0');
                    unsafe {
                        #imgui_fn (label.as_ptr(), &self.value);
                    }
                    if self.value {
                        self.call_callback();
                    }
                }
            }
        };
    } else if contains_value {
        gen = quote! {
            impl ImgGuiGlue for #name {
                fn render(&self) {
                    let mut label = self.label.clone();
                    label.push('\0');
                    unsafe { #imgui_fn (label.as_ptr(), &self.value) }
                }
            }
        };
    } else {
        gen = quote! {
            impl ImgGuiGlue for #name {
                fn render(&self) {
                    let mut label = self.label.clone();
                    label.push('\0');
                    unsafe {
                        #imgui_fn (label.as_ptr());
                    }
                }
            }
        };
    }
    gen.into()
}
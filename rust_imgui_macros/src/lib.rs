extern crate proc_macro;
extern crate syn;

use proc_macro::{TokenStream};
use quote::{quote, format_ident, ToTokens};
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
    let mut contains_label = false;

    let mut fun_param = vec![];

    for member in members {
        match member.ident {
            Some(i) => {
                
                if i == Ident::new("value", syn::__private::Span::call_site()) {
                    contains_value = true;
                    continue;
                } else if i == Ident::new("callback", syn::__private::Span::call_site()) {
                    contains_callback = true;
                    continue;
                } else if i == Ident::new("label", syn::__private::Span::call_site()) {
                    contains_label = true;
                    continue;
                }
                fun_param.push(i.clone().into_token_stream());
            },
            None => unimplemented!()
        }
        
    }

    let imgui_fn = format_ident!("ImGui_{}", name);    
    
    let gen;
    if contains_callback && contains_value && contains_label {
        if fun_param.len() == 0 {
            gen = quote! {
                impl ImgGuiGlue for #name {
                    fn render(&self) {
                        let mut label = self.label.clone();
                        label.push('\0');
                        unsafe {
                            #imgui_fn (label.as_ptr(), &self.value);
                        }
                        if self.value as i32 > 0{ //TODO: if value is non bool the callback is called always and in case value = 0 never... 
                            self.call_callback();
                        }
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
                            #imgui_fn (label.as_ptr(), &self.value, #(self.#fun_param),*);
                        }
                        if self.value as i32 > 0 {
                            self.call_callback();
                        }
                    }
                }
            };
        }
    } else if contains_value && contains_label {
        if fun_param.len() == 0 {
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
                        unsafe { #imgui_fn (label.as_ptr(), &self.value, #(self.#fun_param),*) }
                    }
                }
            };
        }
    } else if contains_label{
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
    } else {
        //all struct members will be used as ImGui_xx() function Parameters
        gen = quote! {
            impl ImgGuiGlue for #name {
                fn render(&self) {
                    unsafe {
                        #imgui_fn (#(self.#fun_param),*);
                    }
                }
            }
        };
    }
    gen.into()
}

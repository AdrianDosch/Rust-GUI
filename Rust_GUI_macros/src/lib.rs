extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Data, Fields, Ident};

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

#[proc_macro_derive(ImGuiGlue)]
pub fn im_gui_glue_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_im_gui_glue(&ast)
}

fn impl_im_gui_glue(ast: &syn::DeriveInput) -> TokenStream {
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
    let mut ty = None;

    for member in members {
        match member.ident {
            Some(i) => {
                if i == Ident::new("value", syn::__private::Span::call_site()) {
                    contains_value = true;
                    ty = Some(member.ty);
                    continue;
                } else if i == Ident::new("callback", syn::__private::Span::call_site()) {
                    contains_callback = true;
                    continue;
                } else if i == Ident::new("label", syn::__private::Span::call_site()) {
                    contains_label = true;
                    continue;
                }
                fun_param.push(i.clone().into_token_stream());
            }
            None => unimplemented!(),
        }
    }

    let imgui_fn = format_ident!("ImGui_{}", name);

    let label_manipulation = quote! {
        let mut label = self.label.clone();
        if label.len() == 0 {
            label.push(' ');
        }
        label.push('\0');
    };

    let check_callback = quote! {
        if stringify!(#ty) == stringify!(bool) {
            if self.value as i32 == 1 {
                self.call_callback();
            }
        } else {
            if prev != self.value {
                self.call_callback();
            }
        }
    };

    let gen;
    if contains_callback && contains_value && contains_label {
        if fun_param.is_empty() {
            gen = quote! {
                impl ImGuiGlue for #name {
                    fn render(&self) {
                        let prev = self.value.clone();
                        #label_manipulation
                        unsafe {
                            #imgui_fn (label.as_ptr(), &self.value);
                        }

                        #check_callback
                    }
                }
            };
        } else {
            gen = quote! {
                impl ImGuiGlue for #name {
                    fn render(&self) {
                        let prev = self.value.clone();
                        #label_manipulation
                        unsafe {
                            #imgui_fn (label.as_ptr(), &self.value, #(self.#fun_param),*);
                        }

                        #check_callback
                    }
                }
            };
        }
    } else if contains_value && contains_label {
        if fun_param.is_empty() {
            gen = quote! {
                impl ImGuiGlue for #name {
                    fn render(&self) {
                        #label_manipulation
                        unsafe { #imgui_fn (label.as_ptr(), &self.value) }
                    }
                }
            };
        } else {
            gen = quote! {
                impl ImGuiGlue for #name {
                    fn render(&self) {
                        #label_manipulation
                        unsafe { #imgui_fn (label.as_ptr(), &self.value, #(self.#fun_param),*) }
                    }
                }
            };
        }
    } else if contains_label {
        gen = quote! {
            impl ImGuiGlue for #name {
                fn render(&self) {
                    #label_manipulation
                    unsafe {
                        #imgui_fn (label.as_ptr());
                    }
                }
            }
        };
    } else {
        //all struct members will be used as ImGui_xx() function Parameters
        gen = quote! {
            impl ImGuiGlue for #name {
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

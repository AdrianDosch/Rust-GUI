use quote::{self, ToTokens, TokenStreamExt};
use proc_macro2;

#[proc_macro]
///Params:
/// 
/// type: the name of the struct to implement the trait Update on
/// 
/// function call: the function call of the Dear ImGui library 
/// 
/// Example:
/// impl_Update!(SliderInt, ImGui_SliderInt(self.label.blocking_write().as_ptr(), &self.value.blocking_write(), *self.min.blocking_read(), *self.max.blocking_read()));
pub fn impl_Update(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut counter = 0;

    let mut ty = String::from("");
    let mut fun = String::from("");
    let mut param = GroupTokens{ val: None };
    let mut callback = IdentTokens {val: None};
    let mut callback_val = IdentTokens {val: None};

    for tt in proc_macro2::TokenStream::from(input).into_iter() {
        match tt {
            proc_macro2::TokenTree::Group(g) => param = GroupTokens{val: Some(g)},
            proc_macro2::TokenTree::Ident(i) => {
                counter += 1;
                if counter == 1 {
                    ty = i.to_string();
                } else if counter == 2 {
                    fun = i.to_string();
                } else if counter == 3 {
                    callback.val = Some(i);
                } else if counter == 4 {
                    callback_val.val = Some(i);
                }
            },
            proc_macro2::TokenTree::Punct(p) => {},
            proc_macro2::TokenTree::Literal(l) => panic!("unexpected literal"),
        }
    }

    let ty = quote::format_ident!("{}", ty);
    let fun = quote::format_ident!("{}", fun);
    // let param = quote::format_ident!("{}", param);

    let tokens = 
    if callback.val.is_none() { 
        quote::quote!{
            impl Update for #ty {
                fn update(&self) -> bool{
                    unsafe { #fun #param }
                    false
                }
            }
        }
    } else {
        quote::quote!{
            impl Update for #ty {
                fn update(&self) -> bool{
                    let cp = self.#callback_val.blocking_read().clone();
                    unsafe { #fun #param }
                    cp != *self.#callback_val.blocking_read()
                }

                fn call_callback(&self, gui: &Gui) {
                    (self.#callback.blocking_read())(gui);
                }
            }
        }
    };


    tokens.into()
}

struct IdentTokens{
    pub val: Option<proc_macro2::Ident>
}

impl ToTokens for IdentTokens {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(tt) = &self.val {
            let x = self.val.as_ref().unwrap();
            tokens.append(x.clone());
        } else {
            //do nothing
        }
    }
}

struct GroupTokens{
    pub val: Option<proc_macro2::Group>
}

impl ToTokens for GroupTokens {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(tt) = &self.val {
            let x = self.val.as_ref().unwrap();
            tokens.append(x.clone());
        } else {
            //do nothing
        }
    }
}

#[proc_macro]
pub fn what(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    for tt in input.into_iter() {
        match tt {
            proc_macro::TokenTree::Group(g) => eprintln!("Group {}", g),
            proc_macro::TokenTree::Ident(i) => eprintln!("Ident {}", i),
            proc_macro::TokenTree::Punct(p) => eprintln!("Punct {}", p),
            proc_macro::TokenTree::Literal(l) => eprintln!("Literal {}", l),
        }
    }
    

    return TokenStream::new();
}

#[proc_macro]
///Params:
/// 
/// type: the name of the struct to implement the trait Set on
/// 
/// ident: the name of the member variable of the struct Window which contains all widgets of the type specified
/// 
/// 
/// Example:
/// impl_Add!(SliderInt, slider_int);
pub fn impl_Add(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut counter = 0;

    let mut ty = String::from("");
    let mut window_widget_container = String::from("");

    for tt in proc_macro2::TokenStream::from(input).into_iter() {
        match tt {
            proc_macro2::TokenTree::Group(g) => panic!("unexpected group"),
            proc_macro2::TokenTree::Ident(i) => {
                counter += 1;
                if counter == 1 {
                    ty = i.to_string();
                } else if counter == 2 {
                    window_widget_container = i.to_string();
                }
            },
            proc_macro2::TokenTree::Punct(p) => {},
            proc_macro2::TokenTree::Literal(l) => panic!("unexpected literal"),
        }
    }

    let ty = quote::format_ident!("{}", ty);
    let window_widget_container = quote::format_ident!("{}", window_widget_container);

    let tokens = quote::quote!{
        impl Add<#ty> for Window {
            fn add(mut self, widget: #ty) -> Self {
                let widget = Arc::new(widget);
                self.widgets.push(to_update(&widget));
                self.#window_widget_container.push(widget);
                self
            }
        }
    };


    tokens.into()
}


#[proc_macro]
///Params:
/// 
/// type: the name of the struct to implement the trait Set on
/// 
/// type: the type of the value which will be set
/// 
/// ident: the name of the member variable which will be set
/// 
/// 
/// Example:
/// impl_Set!(SliderInt, i32, value);
pub fn impl_Set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut counter = 0;

    let mut ty = String::from("");
    let mut ty2 = String::from("");
    let mut val = String::from("");

    for tt in proc_macro2::TokenStream::from(input).into_iter() {
        match tt {
            proc_macro2::TokenTree::Group(g) => panic!("unexpected group"),
            proc_macro2::TokenTree::Ident(i) => {
                counter += 1;
                if counter == 1 {
                    ty = i.to_string();
                } else if counter == 2 {
                    ty2 = i.to_string();
                } else if counter == 3 {
                    val = i.to_string();
                }
            },
            proc_macro2::TokenTree::Punct(p) => {},
            proc_macro2::TokenTree::Literal(l) => panic!("unexpected literal"),
        }
    }

    let ty = quote::format_ident!("{}", ty);
    let ty2 = quote::format_ident!("{}", ty2);
    let val = quote::format_ident!("{}", val);

    let tokens = quote::quote!{
        impl Set<#ty2> for #ty {
            fn set(&self, value: #ty2) {
                *self.#val.blocking_write() = #val;
            }
        }
    };

    tokens.into()
}




































// use proc_macro::TokenStream;
// use quote::{format_ident, quote, ToTokens};
// use syn::{Data, Fields, Ident};

// #[proc_macro_derive(Callback)]
// pub fn callback_derive(input: TokenStream) -> TokenStream {
//     let ast = syn::parse(input).unwrap();
//     impl_callback(&ast)
// }

// fn impl_callback(ast: &syn::DeriveInput) -> TokenStream {
//     let name = &ast.ident;

//     let gen = quote! {
//         impl Callback for #name {
//             fn set_callback(&mut self, callback: impl Fn() + 'static){
//                 self.callback = Box::new(callback);
//             }

//             fn call_callback(&self) {
//                 (self.callback)();
//             }
//         }
//     };
//     gen.into()
// }

// #[proc_macro_derive(ImGuiGlue)]
// pub fn im_gui_glue_derive(input: TokenStream) -> TokenStream {
//     let ast = syn::parse(input).unwrap();
//     impl_im_gui_glue(&ast)
// }

// fn impl_im_gui_glue(ast: &syn::DeriveInput) -> TokenStream {
//     let name = &ast.ident;
//     let fields = &ast.data;

//     let fields = match fields {
//         Data::Struct(s) => &s.fields,
//         _ => unimplemented!(),
//     };

//     let named = match fields {
//         Fields::Named(named) => named,
//         _ => unimplemented!(),
//     };

//     let members = named.named.clone();
//     let mut contains_value = false;
//     let mut contains_callback = false;
//     let mut contains_label = false;

//     let mut fun_param = vec![];
//     let mut ty = None;

//     for member in members {
//         match member.ident {
//             Some(i) => {
//                 if i == Ident::new("value", syn::__private::Span::call_site()) {
//                     contains_value = true;
//                     ty = Some(member.ty);
//                     continue;
//                 } else if i == Ident::new("callback", syn::__private::Span::call_site()) {
//                     contains_callback = true;
//                     continue;
//                 } else if i == Ident::new("label", syn::__private::Span::call_site()) {
//                     contains_label = true;
//                     continue;
//                 }
//                 fun_param.push(i.clone().into_token_stream());
//             }
//             None => unimplemented!(),
//         }
//     }

//     let imgui_fn = format_ident!("ImGui_{}", name);

//     let label_manipulation = quote! {
//         let mut label = self.label.clone();
//         if label.len() == 0 {
//             label.push(' ');
//         }
//         label.push('\0');
//     };

//     let check_callback = quote! {
//         if stringify!(#ty) == stringify!(bool) {
//             if self.value as i32 == 1 {
//                 self.call_callback();
//             }
//         } else {
//             if prev != self.value {
//                 self.call_callback();
//             }
//         }
//     };

//     let gen;
//     if contains_callback && contains_value && contains_label {
//         if fun_param.is_empty() {
//             gen = quote! {
//                 impl ImGuiGlue for #name {
//                     fn render(&self) {
//                         let prev = self.value.clone();
//                         #label_manipulation
//                         unsafe {
//                             #imgui_fn (label.as_ptr(), &self.value);
//                         }

//                         #check_callback
//                     }
//                 }
//             };
//         } else {
//             gen = quote! {
//                 impl ImGuiGlue for #name {
//                     fn render(&self) {
//                         let prev = self.value.clone();
//                         #label_manipulation
//                         unsafe {
//                             #imgui_fn (label.as_ptr(), &self.value, #(self.#fun_param),*);
//                         }

//                         #check_callback
//                     }
//                 }
//             };
//         }
//     } else if contains_value && contains_label {
//         if fun_param.is_empty() {
//             gen = quote! {
//                 impl ImGuiGlue for #name {
//                     fn render(&self) {
//                         #label_manipulation
//                         unsafe { #imgui_fn (label.as_ptr(), &self.value) }
//                     }
//                 }
//             };
//         } else {
//             gen = quote! {
//                 impl ImGuiGlue for #name {
//                     fn render(&self) {
//                         #label_manipulation
//                         unsafe { #imgui_fn (label.as_ptr(), &self.value, #(self.#fun_param),*) }
//                     }
//                 }
//             };
//         }
//     } else if contains_label {
//         gen = quote! {
//             impl ImGuiGlue for #name {
//                 fn render(&self) {
//                     #label_manipulation
//                     unsafe {
//                         #imgui_fn (label.as_ptr());
//                     }
//                 }
//             }
//         };
//     } else {
//         //all struct members will be used as ImGui_xx() function Parameters
//         gen = quote! {
//             impl ImGuiGlue for #name {
//                 fn render(&self) {
//                     unsafe {
//                         #imgui_fn (#(self.#fun_param),*);
//                     }
//                 }
//             }
//         };
//     }
//     gen.into()
// }

use proc_macro::{TokenStream, TokenTree, Ident, Span};

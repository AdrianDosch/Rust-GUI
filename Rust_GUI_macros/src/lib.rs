use quote::{self, ToTokens, TokenStreamExt};
use proc_macro::TokenStream;
use proc_macro2;

#[proc_macro]
#[allow(non_snake_case)]
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
            proc_macro2::TokenTree::Punct(_) => {},
            proc_macro2::TokenTree::Literal(_) => panic!("unexpected literal"),
        }
    }

    let ty = quote::format_ident!("{}", ty);
    let fun = quote::format_ident!("{}", fun);
    // let param = quote::format_ident!("{}", param);

    let tokens = 
    if callback.val.is_none() { 
        quote::quote!{
            impl Update for #ty {
                fn update(&self, gui: &Gui) -> bool{
                    unsafe { #fun #param }
                    false
                }
                fn as_any(&self) -> &dyn Any{
                    self
                }
            }
        }
    } else {
        quote::quote!{
            impl Update for #ty {
                fn update(&self, gui: &Gui) -> bool{
                    let cp = self.#callback_val.blocking_read().clone();
                    unsafe { #fun #param }
                    cp != *self.#callback_val.blocking_read()
                }

                fn call_callback(&self, gui: &Gui) {
                    (self.#callback.blocking_read())(gui);
                }

                fn set_callback<T: 'static + Send + Sync + Fn(&Gui)>(mut self, callback: T) -> Self {
                    self.callback = Arc::new(RwLock::new(Box::new(callback)));
                    self
                }
                
                fn as_any(&self) -> &dyn Any{
                    self
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
            tokens.append(tt.clone());
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
            tokens.append(tt.clone());
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


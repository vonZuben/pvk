
use proc_macro2::TokenStream; // 1.0.9
use quote::{quote, ToTokens}; // 1.0.3

use crate::{ty::*, utils::StrAsCode};
use crate::utils;
use crate::global_data;

struct Setter<'a> {
    field: &'a vkxml::Field,
    container: &'a str,
}

impl ToTokens for Setter<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field = &self.field;
        let container = self.container;

        let f_name = utils::field_name_expected(field);
        let member_name = utils::case::camel_to_snake(f_name).as_code();
        let set_member_name = utils::case::camel_to_snake(&("set_".to_owned() + f_name)).as_code();

        let count_setter = field.size.as_ref()
                .map(|size| {
                    if f_name == "pCode" {
                        Some( quote!(self.code_size = #member_name.len() as usize * 4;) )
                    }
                    else if f_name == "pSampleMask" {
                        None
                    }
                    else {
                        match field.array.as_ref().expect("error: field with size but not an array") {
                            vkxml::ArrayType::Static => return None,
                            vkxml::ArrayType::Dynamic => {
                                let count_name = utils::case::camel_to_snake(size).as_code();
                                Some( quote!(self.#count_name = #member_name.len() as _;) )
                            }
                        }
                    }
                });

        let member_type = utils::Rtype::new(field, container)
                .public_lifetime("'public")
                .private_lifetime("'private")
                .ref_lifetime("'private")
                .context(utils::FieldContext::Member);

        quote!(
            pub fn #member_name(mut self, #member_name: #member_type) -> Self {
                #count_setter
                self.#member_name = #member_name.to_c();
                self
            }
            pub fn #set_member_name(&mut self, #member_name: #member_type) -> &mut Self {
                #count_setter
                self.#member_name = #member_name.to_c();
                self
            }
        ).to_tokens(tokens);
    }
}
// only supports normal structs (not unit or tuple structs)
#[derive(Default)]
pub struct Struct<'a> {
    name: &'a str,
    generics: Generics,
    fields: Vec<Field>,
    attributes: Vec<TokenStream>,
    public: bool,
    setters: Vec<Setter<'a>>,
}

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.as_code();
        let generics = &self.generics;
        let fields = &self.fields;
        let attributes = &self.attributes;
        let setters = &self.setters;

        let public = if self.public {
            Some(quote!(pub))
        }
        else {
            None
        };

        let struct_init_code = if setters.is_empty() {
            None
        }
        else {
            let init = if global_data::is_base(self.name) {
                quote! {
                    Self::init_s_type()
                }
            }
            else {
                quote! {
                    unsafe { MaybeUninit::uninit().assume_init() }
                }
            };

            let must_init_setters: Vec<_> = setters.iter()
                .filter(|setter| utils::must_init(self.name, setter.field)).collect();

            let field_names: Vec<_> = must_init_setters.iter()
                .map(|setter|{
                    let name = utils::case::camel_to_snake(utils::field_name_expected(setter.field)).as_code();
                    quote!(#name)
                })
                .collect();

            let field_types: Vec<_> = must_init_setters.iter()
                .map(|setter|{
                    let ty = utils::Rtype::new(setter.field, setter.container)
                        .public_lifetime("'public")
                        .private_lifetime("'private")
                        .ref_lifetime("'private")
                        .context(utils::FieldContext::Member);
                    quote!(#ty)
                })
                .collect();

            Some(quote!{
                impl #generics #name #generics {
                    pub fn uninit() -> Self {
                        #init
                    }
                    pub fn new(#(#field_names: #field_types),*) -> Self {
                        Self {
                            #(#field_names: #field_names.to_c(),)*
                            ..Self::uninit()
                        }
                    }
                    #(#setters)*
                }
            })
        };

        quote!(
            #(#attributes)*
            #public struct #name #generics {
                #(#fields,)*
            }
            #struct_init_code
        ).to_tokens(tokens);
    }
}

impl<'a> Struct<'a> {
    pub fn new(name: &'a impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref(),
            .. Self::default()
        }
    }
    pub fn lifetime_param(mut self, l: impl Into<Lifetime>) -> Self {
        self.generics.push_lifetime_param(l);
        self
    }
    #[allow(unused)]
    pub fn type_param(mut self, t: impl ToTokens) -> Self {
        self.generics.push_type_param(t);
        self
    }
    pub fn fields(mut self, f: impl IntoIterator<Item=Field>) -> Self {
        self.fields = f.into_iter().collect();
        self
    }
    pub fn attribute(mut self, a: TokenStream) -> Self {
        self.attributes.push(a);
        self
    }
    pub fn public(mut self) -> Self {
        self.public = true;
        self
    }
    pub fn setters(mut self, s: impl IntoIterator<Item=&'a vkxml::Field>) -> Self {
        self.setters = s.into_iter().map(|field| Setter{field, container: self.name}).collect();
        self
    }
}
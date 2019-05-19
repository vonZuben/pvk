
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;

pub fn handle_definitions(definitions: &Definitions) -> TokenStream {

    let q = definitions.elements.iter().map(|def| {

        match def {
            DefinitionsElement::Typedef(type_def) => {
                let actual_type = type_def.basetype.as_ident();
                let name = type_def.name.as_ident();
                quote!{
                    pub type #name = #actual_type;
                }
            },
            DefinitionsElement::Reference(_reference) => {
                //TODO For now we will not include this
                // and consider manually adding anything necessary later

                //println!("{:#?}", reference);
                //let name = reference.name.as_ident();
                //rvec.push(quote!{
                //    pub type #name = *const c_void;
                //});
                quote!()
            },
            DefinitionsElement::Bitmask(bitmask) => {
                // TODO do somthing with the enumref
                let actual_type = bitmask.basetype.as_code();
                let name = bitmask.name.as_code();
                quote!{
                    #[repr(transparent)]
                    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
                    pub struct #name(pub(crate) #actual_type);
                }
            },
            DefinitionsElement::Struct(stct) => {
                let name = stct.name.as_code();

                let params = stct.elements.iter().filter_map( |elem| match elem {
                    StructElement::Member(field) => Some(handle_field(field)),
                    StructElement::Notation(_) => None,
                });

                quote!{
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    pub struct #name {
                        #( #params ),*
                    }
                }
            },
            DefinitionsElement::Union(uni) => {
                let name = uni.name.as_ident();
                let params = uni.elements.iter().map(handle_field);

                quote!{
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    union #name {
                        #( #params ),*
                    }
                }

            },
            //DefinitionsElement::Define(def) => {
            //    dbg!(def);
            //    quote!()
            //}
            DefinitionsElement::Handle(handle) => {
                let name = handle.name.as_code();
                match handle.ty {
                    // based on the spec, i understand that dispatchable
                    // handles will be pointers, thus, they will be different
                    // sizes on 32bit and 64 bit computers
                    // but nondispatchable handles will always be 64 bits
                    HandleType::Dispatch => {
                        quote!(pub type #name = usize;)
                    },
                    HandleType::NoDispatch => {
                        quote!(pub type #name = u64;)
                    },
                }
            },
            // We will ignor this because the enum elements
            // are defined elsewhere, and rust dosn't need the
            // definitions like this
            //DefinitionsElement::Enumeration(enumeration) => {
            //    dbg!(enumeration);
            //    quote!()
            //},
            // TODO funtion pointers
            DefinitionsElement::FuncPtr(fptr) => {
                let name = fptr.name.as_code();
                let return_type = make_field_type(&fptr.return_type);
                let params = fptr.param.iter().map(handle_field);

                quote!{
                    #[allow(non_camel_case_types)]
                    pub type #name = extern "system" fn(
                        #( #params ),*
                        ) -> #return_type;
                }
            },
            _ => quote!(),
        }

    });

    quote!( #(#q)* )

}

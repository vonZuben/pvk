
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::ParseState;
//use crate::commands;

pub fn make_manager_name(name: &str) -> TokenStream {
    format!("{}Manager", name).as_code()
}

pub fn handle_definitions(definitions: &Definitions, _parse_state: &mut ParseState) -> TokenStream {

    let q = definitions.elements.iter().map(|def| {

        match def {
            DefinitionsElement::Typedef(type_def) => {
                let actual_type = type_def.basetype.as_code();
                let name = type_def.name.as_code();
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
                let name = uni.name.as_code();
                let params = uni.elements.iter().map(handle_field);

                quote!{
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    pub union #name {
                        #( #params ),*
                    }
                }

            },
            //DefinitionsElement::Define(def) => {
            //    dbg!(def);
            //    quote!()
            //}
            DefinitionsElement::Handle(handle) => {

                let handle_name = handle.name.as_code();

                match handle.ty {
                    // based on the spec, i understand that dispatchable
                    // handles will be pointers, thus, they will be different
                    // sizes on 32bit and 64 bit computers
                    // but nondispatchable handles will always be 64 bits
                    HandleType::Dispatch => {
                        // get list of functions where the first parameter is the the handle type
                        //let commands: Vec<_> = parse_state.command_list.iter()
                        //    .filter_map(|command_node| {
                        //        if command_node.data().param[0].basetype == handle.name {
                        //            Some(command_node.take())
                        //        }
                        //        else {
                        //            None
                        //        }
                        //    }).collect();

                        //let pfn_params = commands.iter()
                        //    .map(|command| {
                        //        let name = command.name.as_code();
                        //        let pfn_name = commands::make_pfn_loader_name(&command);

                        //        quote!( #name: #pfn_name )
                        //    });

                        //let pfn_names = commands.iter()
                        //    .map(|command| {
                        //        let name = command.name.as_code();

                        //        quote!( #name )
                        //    });

                        let manager_members = match handle.name.as_str() {
                            "VkInstance" => quote!( commands: InstanceCommands, ),
                            "VkDevice" => quote!( commands: DeviceCommands, ),
                            _ => {
                                let parent = handle.parent.as_ref().expect("error: expected parent for handle").as_code();
                                quote!( parent: #parent, )
                            }
                        };

                        // each dispatchable handle will have a manager type that will handle
                        // creation and destruciton automatically, and will provide convinience
                        // methods for their respective vulkan commands (i.e. where the respective
                        // handle is the first parameter)
                        //
                        // check commands.rs for method definitions
                        let custum_type_name = make_manager_name(handle.name.as_str());

                        // make the handle type
                        quote!{

                            pub type #handle_name = *const c_void; // object pointer???

                            pub struct #custum_type_name {
                                handle: #handle_name,
                                #manager_members
                                //#( #pfn_params ),*
                            }

                            //impl #name {

                            //    fn load_function_pointers(&mut self) {
                            //        #( self.#pfn_names.load(); )*
                            //    }
                            //}
                        }
                    },
                    HandleType::NoDispatch => {
                        quote!(pub type #handle_name = u64;) // uint64_t
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

pub fn generate_aliases_of_types<'a>(types: &'a vk_parse::Types) -> TokenStream {
    let aliases = types
        .children
        .iter()
        .filter_map(|child| match child {
            vk_parse::TypesChild::Type(ty) => Some((ty.name.as_ref()?, ty.alias.as_ref()?)),
            _ => None,
        })
        .filter_map(|(name, alias)| {
            if name.contains("FlagBits") { // be carful of this since as_code will convert FlagBits to Flags
                return None;
            }
            let name_ident = name.as_code();
            let alias_ident = alias.as_code();
            let tokens = quote! {
                pub type #name_ident = #alias_ident;
            };
            Some(tokens)
        });
    quote! {
        #(#aliases)*
    }
}

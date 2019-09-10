
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

                let builder_code;

                // gererate bulders and initializers for only non return types
                fn not_return(stct: &Struct) -> bool {
                    if stct.name.contains("BaseOutStructure") || stct.name.contains("BaseInStructure") {
                        false
                    }
                    else {
                        !stct.is_return
                    }
                }
                if not_return(&stct) {
                    //dbg!(&stct);
                    let member_setters = stct.elements.iter().filter_map( |elem| match elem {
                        StructElement::Member(field) => {

                            let field_name = field.name.as_ref().expect("error, field with no name");

                            let raw_name = field_name.as_code();
                            let setter_name = case::camel_to_snake(field_name).as_code();

                            let basetype = field.basetype.as_code();

                            let mut arg_type;
                            let mut val_setter;
                            let mut count_setter = quote!();

                            let arg_mut = match field.is_const {
                                true => quote!(),
                                false => quote!(mut),
                            };

                            let ptr_mut = match field.is_const {
                                true => quote!(as_ptr()),
                                false => quote!(as_mut_ptr()),
                            };

                            match &field.array {
                                Some(ArrayType::Dynamic) => {
                                    if field.basetype.as_str() == "char" {

                                        val_setter = quote!(self.inner.#raw_name = val.#ptr_mut;);

                                        match &field.reference {
                                            Some(ReferenceType::Pointer) => {

                                                arg_type = quote!(&'a #arg_mut ::std::ffi::CStr);

                                            }
                                            Some(ReferenceType::PointerToConstPointer) => {

                                                arg_type = quote!(&'a #arg_mut [*const c_char]);

                                                let count_name = field.size.as_ref()
                                                    .expect("char PointerToConstPointer with no size error")
                                                    .as_code();
                                                count_setter = quote!(self.inner.#count_name = val.len() as _;);

                                            }
                                            _ => panic!("unexpected refernce case for char field"),
                                        }

                                    }
                                    else if field.basetype.as_str() == "void" {
                                        match &field.reference {
                                            Some(ReferenceType::Pointer) => {

                                                arg_type = quote!(&'a #arg_mut [u8]);
                                                val_setter = quote!(self.inner.#raw_name = val.#ptr_mut as *const c_void;);

                                                let count_name = field.size.as_ref()
                                                    .expect("void Pointer with no size error")
                                                    .as_code();
                                                count_setter = quote!(self.inner.#count_name = val.len() as _;);

                                            }
                                            _ => panic!("unexpected refernce case for void field"),
                                        }
                                    }
                                    else { // any other possible type

                                        arg_type = quote!(&'a #arg_mut [#basetype]);
                                        val_setter = quote!(self.inner.#raw_name = val.#ptr_mut;);

                                        match &field.reference {
                                            Some(ReferenceType::Pointer) => {
                                                // the size of pCode and pSampleMask are provided
                                                // as c code or latext math. maybe provide some
                                                // more complex code later to parse properly?
                                                if field.name.as_ref().unwrap() == "pCode" {

                                                    // special case for setting count
                                                    count_setter = quote!(self.inner.codeSize = val.len() as usize * 4;);

                                                }
                                                else if field.name.as_ref().unwrap() == "pSampleMask" {
                                                    // do nothing for this case (i.e. no set count)
                                                }
                                                else if field.size.is_some() {

                                                    let count_name = field.size.as_ref()
                                                        .unwrap()
                                                        .as_code();
                                                    count_setter = quote!(self.inner.#count_name = val.len() as _;);

                                                }
                                                else {
                                                    panic!("error: no size for array type")
                                                }
                                            }
                                            _ => panic!("unhandled refernce case for 'any' field"),
                                        }
                                    }
                                }
                                Some(ArrayType::Static) => {
                                    let size = field.size.as_ref()
                                        .expect("error: static array with no size")
                                        .as_code();
                                    arg_type = quote!( [#basetype; #size] );
                                    val_setter = quote!(self.inner.#raw_name = val;);
                                }
                                None => {

                                    val_setter = quote!(self.inner.#raw_name = val;);

                                    match field.reference {
                                        Some(ReferenceType::Pointer) => {

                                            arg_type = quote!(&'a #arg_mut #basetype);

                                        }
                                        None => {

                                            arg_type = quote!(#basetype);

                                        }
                                        _ => panic!("error: unexpected case for none array type"),
                                    }
                                }
                            }

                            Some(quote!{
                                pub fn #setter_name(&mut self, val : #arg_type) -> &mut Self {
                                    #val_setter
                                    #count_setter
                                    self
                                }
                            })
                        }
                        StructElement::Notation(_) => None,
                    });

                    let builder_name = format!("{}Builder", stct.name).as_code();

                    builder_code = Some(quote!{
                        impl #name {
                            fn zeroed() -> Self {
                                unsafe { ::std::mem::zeroed() }
                            }
                            pub fn builder<'a>() -> #builder_name<'a> {
                                #builder_name {
                                    inner: Self::zeroed(),
                                    phantom: ::std::marker::PhantomData,
                                }
                            }
                        }
                        pub struct #builder_name<'a> {
                            inner: #name,
                            phantom: ::std::marker::PhantomData<&'a ()>,
                        }
                        impl<'a> ::std::ops::Deref for #builder_name<'a> {
                            type Target = #name;
                            fn deref(&self) -> &Self::Target {
                                &self.inner
                            }
                        }
                        impl<'a> #builder_name<'a> {
                            #( #member_setters )*
                        }
                    });
                }
                else {
                    builder_code = None;
                }

                quote!{
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    pub struct #name {
                        #( #params ),*
                    }
                    #( #builder_code )*
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

                        // handle managers will provide convinience usage of dipatchable handles
                        // define the members that each type should have
                        // the instance and device managers should hold function pointers
                        // the other managers should have references to their parent
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

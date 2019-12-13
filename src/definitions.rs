
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::ParseState;
//use crate::commands;

pub fn make_handle_owner_name(name: &str) -> TokenStream {
    format!("{}Owner", name).as_code()
}

pub fn handle_definitions<'a>(definitions: &'a Definitions, parse_state: &mut ParseState<'a>) -> TokenStream {

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
                    StructElement::Member(field) => Some(make_c_field(field, FieldContext::Member)),
                    StructElement::Notation(_) => None,
                });

                // gererate bulders and initializers for only non return types
                fn not_return(stct: &Struct) -> bool {
                    if stct.name.contains("BaseOutStructure") || stct.name.contains("BaseInStructure") {
                        false
                    }
                    else {
                        !stct.is_return
                    }
                }
                let builder_code = if not_return(&stct) {
                    let member_setters = stct.elements.iter().filter_map( |elem| match elem {
                        StructElement::Member(field) => {

                            let raw_name = field.name.as_ref().expect("error, field with no name");

                            let field_name = raw_name.as_code();
                            let setter_name = case::camel_to_snake(raw_name).as_code();

                            let setter_field = make_rust_field(field);
                            let val_setter = quote!(self.inner.#field_name = #field_name.into(););
                            let count_setter = field.size.as_ref()
                                .map(|size| {
                                    if raw_name == "pCode" {
                                        Some( quote!(self.inner.codeSize = #field_name.len() as usize * 4;) )
                                    }
                                    else if raw_name == "pSampleMask" {
                                        None
                                    }
                                    else {
                                        match field.array.as_ref().expect("error: field with size but not an array") {
                                            ArrayType::Static => return None,
                                            _ => {}
                                        }
                                        let count_name = size.as_code();
                                        Some( quote!(self.inner.#count_name = #field_name.len() as _;) )
                                    }
                                });

                            Some(quote!{
                                pub fn #setter_name(&mut self, #setter_field) -> &mut Self {
                                    #val_setter
                                    #count_setter
                                    self
                                }
                            })
                        }
                        StructElement::Notation(_) => None,
                    });

                    let builder_name = format!("{}Builder", stct.name).as_code();

                    Some(quote!{
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
                    })
                }
                else { // return only type
                    None
                };

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
                let params = uni.elements.iter().map(|field|make_c_field(field, FieldContext::Member));

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

                parse_state.is_handle.insert(handle.name.as_str(), ());

                let handle_name = handle.name.as_code();

                parse_state.handle_cache.push(&handle);

                let owner_name = make_handle_owner_name(handle.name.as_str());

                quote!{
                    #[derive(Debug)]
                    #[repr(transparent)]
                    struct #handle_name<'a, T: SyncType> {
                        handle: raw::#handle_name,
                        _parent_ref: ::std::marker::PhantomData<&'a #owner_name<'a>>,
                        _sync_type: SyncWrapper<T>,
                    }
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
                let return_type = make_c_type(&fptr.return_type, FieldContext::Member);
                let params = fptr.param.iter().map(|field|make_c_field(field, FieldContext::FunctionParam));

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

fn get_dispatchable_parent(handle: &Handle, handle_cache: &[&Handle]) -> Option<TokenStream> {
    handle.parent.as_ref()
        .and_then(|parent_name| {
            find_in_slice(handle_cache, |handle| handle.name.as_str() == parent_name.as_str())
                .and_then(|handle| match handle.ty {
                    HandleType::Dispatch => Some( handle.name.as_str().as_code() ),
                    HandleType::NoDispatch => get_dispatchable_parent(handle, handle_cache),
                })
        })
}

// TODO maybe remove this
//fn is_parent_dispatchable(handle: &Handle, handle_cache: &[&Handle]) -> bool {
//    handle.parent.as_ref()
//        .and_then(|parent_name| {
//            find_in_slice(handle_cache, |handle| handle.name.as_str() == parent_name.as_str())
//                .and_then(|handle| match handle.ty {
//                    HandleType::Dispatch => Some( make_handle_owner_name(handle.name.as_str()) ),
//                    HandleType::NoDispatch => get_dispatchable_parent_owner(handle, handle_cache),
//                })
//        })
//}

pub fn post_process_handles(parse_state: &ParseState) -> TokenStream {

    let owners = parse_state.handle_cache.iter().map(|handle| {

        let handle_name = handle.name.as_code();

        match handle.ty {
            HandleType::Dispatch => {
                // each dispatchable handle will have a owner type that will handle
                // creation and destruciton automatically, and will provide convinience
                // methods for their respective vulkan commands (i.e. where the respective
                // handle is the first parameter)
                //
                // check commands.rs for method definitions
                let owner_name = make_handle_owner_name(handle.name.as_str());

                // handle owners will provide convinience usage of dipatchable handles
                // define the members that each type should have
                // the instance and device owners should hold function pointers
                // the other owners should have references to their parent
                let owner_members = match handle.name.as_str() {
                    "VkInstance" => quote!{
                        commands: InstanceCommands,
                        feature_version: Box<dyn Feature>,
                        phantom: ::std::marker::PhantomData<&'a ()>,
                    },
                    "VkDevice" => quote!{
                        commands: DeviceCommands,
                        dispatch_parent: PhysicalDevice<'a>,
                    },
                    _ => {
                        let dispatch_parent = get_dispatchable_parent(&handle, parse_state.handle_cache.as_slice());
                        quote!{
                            dispatch_parent: #dispatch_parent<'a>,
                        }
                    }
                };

                let new_method = match handle.name.as_str() {
                    "VkInstance" => quote!{
                        fn new(handle: raw::Instance, commands: InstanceCommands,
                               feature_version: Box<dyn Feature>) -> #owner_name<'a> {
                            #owner_name {
                                handle,
                                commands,
                                feature_version,
                                phantom: ::std::marker::PhantomData,
                            }
                        }
                    },
                    "VkDevice" => quote!{
                        fn new(handle: raw::Device, commands: DeviceCommands,
                               dispatch_parent: PhysicalDevice<'a>) -> #owner_name<'a> {
                            #owner_name {
                                handle,
                                commands,
                                dispatch_parent,
                            }
                        }
                    },
                    _ => {
                        let dispatch_parent = get_dispatchable_parent(&handle, parse_state.handle_cache.as_slice());
                        quote!{
                            fn new(handle: #handle_name<'a>,
                                            dispatch_parent: #dispatch_parent<'a>) -> #owner_name<'a> {
                                #owner_name {
                                    handle,
                                    dispatch_parent,
                                }
                            }
                        }
                    }
                };

                // make the handle owner
                quote!{
                    pub struct #owner_name<'a> {
                        handle: raw::#handle_name,
                        #owner_members
                        //#( #pfn_params ),*
                    }
                    impl<'a> #owner_name<'a> {
                        #new_method
                    }
                }
            }
            HandleType::NoDispatch => {
                let owner_name = make_handle_owner_name(handle.name.as_str());

                let new_method;
                let dispatch_parent = if let Some(parent_name) = handle.parent.as_ref() {
                    // NOTE some non-dispatchable handle type can have multiple parents
                    // for now, we just take the first parent
                    let parent_name = parent_name.as_str().split(',')
                        .next()
                        .expect("there must be at least one elemet in the parent names");

                    // NOTE in order to make code generation easier (especially regarding method
                    // creation), we make the device a parent to the swapchain types (rather the
                    // actual parent which is the surface). This is because the surface owner is
                    // not easily available in the swapchain create methods
                    let dispatch_parent;
                    if handle.name.as_str() == "VkSwapchainKHR" {
                        dispatch_parent = quote!(Device);
                    }
                    else {
                        dispatch_parent = parent_name.as_code();
                    }

                    new_method = quote!{
                        fn new(handle: #handle_name<'a>, dispatch_parent: #dispatch_parent<'a>) -> #owner_name<'a> {
                                #owner_name {
                                    handle,
                                    dispatch_parent,
                                }
                            }
                    };

                    quote!{
                        dispatch_parent: #dispatch_parent<'a>,
                    }
                }
                // for handles with no parent, it is easier to make a method that
                // takes a parent parameter for consistency and just ignoring the param
                else {
                    new_method = quote!{
                        fn new<T>(handle: #handle_name<'a>, _parent: T) -> #owner_name<'a> {
                                #owner_name {
                                    handle,
                                    phantom: ::std::marker::PhantomData,
                                }
                            }
                    };
                    quote!( phantom: ::std::marker::PhantomData<&'a ()>, )
                };

                quote!{
                    pub struct #owner_name<'a> {
                        handle: raw::#handle_name,
                        #dispatch_parent
                    }
                    impl<'a> #owner_name<'a> {
                        #new_method
                    }
                }
            }
        }
    });

    let raw_handles = parse_state.handle_cache.iter()
        .map(|handle|
             {
                 let handle_name = handle.name.as_code();
                 let handle_type = match handle.ty {
                     HandleType::Dispatch => quote!( *const c_void ),
                     HandleType::NoDispatch => quote!( u64 ),
                 };
                 quote!( pub type #handle_name = #handle_type; )
             });

    let raw_module = quote!{
        mod raw {
            use std::os::raw::*;
            #( #raw_handles )*
        }
    };

    quote!{
        #( #owners )*
        #raw_module
    }
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

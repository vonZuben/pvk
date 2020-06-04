
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::ParseState;

use crate::global_data;

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
                    StructElement::Member(field) => Some(c_field(field, WithLifetime::Yes, FieldContext::Member)),
                    StructElement::Notation(_) => None,
                });

                let lifetime = global_data::lifetime(stct.name.as_str());

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

                            let setter_field = r_field(field, WithLifetime::Yes, FieldContext::Member);
                            let val_setter = quote!(self.#field_name = #field_name.into(););
                            let count_setter = field.size.as_ref()
                                .map(|size| {
                                    if raw_name == "pCode" {
                                        Some( quote!(self.codeSize = #field_name.len() as usize * 4;) )
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
                                        Some( quote!(self.#count_name = #field_name.len() as _;) )
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
                        impl #lifetime #name #lifetime {
                            fn zeroed() -> Self {
                                unsafe { ::std::mem::zeroed() }
                            }
                            //pub fn builder<'a>() -> #builder_name<'a> {
                            //    #builder_name {
                            //        inner: Self::zeroed(),
                            //        phantom: ::std::marker::PhantomData,
                            //    }
                            //}
                        }
                        //pub struct #builder_name<'handle> {
                        //    inner: #name #lifetime,
                        //    phantom: ::std::marker::PhantomData<&'handle ()>,
                        //}
                        //impl<'handle> ::std::ops::Deref for #builder_name<'handle> {
                        //    type Target = #name #lifetime;
                        //    fn deref(&self) -> &Self::Target {
                        //        &self.inner
                        //    }
                        //}
                        impl<'handle> #name<'handle> {
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
                    pub struct #name<'handle> {
                        #( #params, )*
                        _p: PhantomData<&'handle ()>
                    }
                    #( #builder_code )*
                }
            },
            DefinitionsElement::Union(uni) => {
                let name = uni.name.as_code();
                let params = uni.elements.iter().map(|field|c_field(field, WithLifetime::Yes, FieldContext::Member));

                let lifetime = global_data::lifetime(uni.name.as_str());

                quote!{
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    pub union #name #lifetime {
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

                parse_state.handle_cache.push(&handle);

                let owner_name = make_handle_owner_name(handle.name.as_str());

                quote!{
                    #[derive(Debug, Clone, Copy)]
                    #[repr(transparent)]
                    pub struct #handle_name<'owner> {
                        handle: raw::#handle_name,
                        // when a Handleowner provides the inner handle for use, the provided
                        // handle borrows the owner. This allows us to keep the owner borrowed
                        // which is helpful especiially when creating new handleOwners so that
                        // Memory does not outlive Device for example
                        _parent_ref: PhantomData<&'owner #owner_name<'owner>>,
                    }
                    impl<'a> From<&#owner_name<'a>> for #handle_name<'a> {
                        fn from(owner: &#owner_name<'a>) -> Self {
                            owner.handle
                        }
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
                let return_type = c_type(&fptr.return_type, WithLifetime::No, FieldContext::Member);
                let params = fptr.param.iter().map(|field|c_field(field, WithLifetime::No, FieldContext::FunctionParam));

                //let lifetime = global_data::lifetime(fptr.name.as_str());

                quote!{
                    #[allow(non_camel_case_types)]
                    //pub type #name #lifetime = extern "system" fn(
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
    match handle.name.as_str() { // exceptions for these handles
        "VkSwapchainKHR" => return Some( quote!(DeviceOwner) ),
        "VkDisplayModeKHR" => return Some( quote!(PhysicalDeviceOwner) ),
        _ => {}
    }
    handle.parent.as_ref()
        .and_then(|parent_name| {
            find_in_slice(handle_cache, |handle| handle.name.as_str() == parent_name.as_str())
                .and_then(|handle| match handle.ty {
                    HandleType::Dispatch => Some( make_handle_owner_name(handle.name.as_str()) ),
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

                let dispatch_parent = get_dispatchable_parent(&handle, parse_state.handle_cache.as_slice());

                // handle owners will provide convinience usage of dipatchable handles
                // define the members that each type should have
                // the instance and device owners should hold function pointers
                // the other owners should have references to their parent
                let owner_members = match handle.name.as_str() {
                    "VkInstance" => quote!{
                        commands: InstanceCommands,
                        feature_version: Box<dyn Feature>,
                        _parent: std::marker::PhantomData<&'parent ()>,
                    },
                    "VkDevice" => quote!{
                        commands: DeviceCommands,
                        dispatch_parent: &'parent PhysicalDeviceOwner<'parent>,
                    },
                    _ => {
                        quote!{
                            dispatch_parent: &'parent #dispatch_parent<'parent>,
                        }
                    }
                };

                let new_method = match handle.name.as_str() {
                    "VkInstance" => quote!{
                        fn new(handle: Instance<'static>, commands: InstanceCommands,
                               feature_version: Box<dyn Feature>) -> #owner_name {
                            #owner_name {
                                handle,
                                commands,
                                feature_version,
                                _parent: PhantomData,
                            }
                        }
                    },
                    "VkDevice" => quote!{
                        fn new<'parent>(handle: Device<'static>, commands: DeviceCommands,
                               dispatch_parent: &'parent PhysicalDeviceOwner) -> #owner_name<'parent> {
                            #owner_name {
                                handle,
                                commands,
                                dispatch_parent,
                            }
                        }
                    },
                    _ => {
                        quote!{
                            fn new<'parent>(handle: #handle_name<'static>,
                                            dispatch_parent: &'parent #dispatch_parent) -> #owner_name<'parent> {
                                #owner_name {
                                    handle,
                                    dispatch_parent,
                                }
                            }
                        }
                    }
                };

                let return_impl = match handle.name.as_str() {
                    "VkInstance" => None,
                    "VkDevice" => Some(quote!{
                        impl<'parent> Return<#owner_name<'parent>> for ((#handle_name<'static>), &'parent #dispatch_parent<'_>) {
                            fn ret(self) -> #owner_name<'parent> {
                                unimplemented!()
                                    //#owner_name::new(self.0, self.1);
                            }
                        }
                    }),
                    _ => Some(quote!{
                        impl<'parent> Return<#owner_name<'parent>> for ((#handle_name<'static>), &'parent #dispatch_parent<'_>) {
                            fn ret(self) -> #owner_name<'parent> {
                                #owner_name::new(self.0, self.1)
                            }
                        }
                        impl<'parent> Return<Vec<#owner_name<'parent>>> for
                                ((Vec<#handle_name<'static>>), &'parent #dispatch_parent<'_>) {
                            fn ret(self) -> Vec<#owner_name<'parent>> {
                                self.0.iter().copied().map(|handle| ((handle), self.1).ret()).collect()
                            }
                        }
                    })
                };

                // make the handle owner
                quote!{
                    // NOTE: a handle owner will hold a reference to it's dispatch parent, and a
                    // 'virtual' borrow of any direct parent (e.g. CommandBufferOwner holds a
                    // reference of the Device, and the CommandBufferOwner borrows the CommandPool
                    // due to the way that the allocate fn is defined
                    pub struct #owner_name<'parent> {
                        // the interpretation of this is that nothing is acutally borrowed, and nothing is 'static
                        handle: #handle_name<'static>,
                        #owner_members
                        //#( #pfn_params ),*
                    }
                    impl #owner_name<'_> {
                        #new_method
                    }
                    #return_impl
                }
            }
            HandleType::NoDispatch => {
                let owner_name = make_handle_owner_name(handle.name.as_str());

                let new_method;
                let return_impl;
                let dispatch_member = if let Some(parent_name) = handle.parent.as_ref() {
                    // NOTE some non-dispatchable handle type can have multiple parents
                    // for now, we just take the first parent
                    let parent_name = parent_name.as_str().split(',')
                        .next()
                        .expect("there must be at least one elemet in the parent names");

                    let dispatch_parent = get_dispatchable_parent(&handle, parse_state.handle_cache.as_slice())
                            .unwrap_or(quote!(DeviceOwner));

                    new_method = quote!{
                        fn new<'parent>(handle: #handle_name<'static>,
                               dispatch_parent: &'parent #dispatch_parent) -> #owner_name<'parent> {
                            #owner_name {
                                handle,
                                dispatch_parent,
                            }
                        }
                    };

                    return_impl = quote!{
                        impl<'parent> Return<#owner_name<'parent>> for ((#handle_name<'static>), &'parent #dispatch_parent<'_>) {
                            fn ret(self) -> #owner_name<'parent> {
                                #owner_name::new(self.0, self.1)
                            }
                        }
                        impl<'parent> Return<Vec<#owner_name<'parent>>> for
                                ((Vec<#handle_name<'static>>), &'parent #dispatch_parent<'_>) {
                            fn ret(self) -> Vec<#owner_name<'parent>> {
                                self.0.iter().copied().map(|handle| ((handle), self.1).ret()).collect()
                            }
                        }
                    };

                    quote!{
                        dispatch_parent: &'parent #dispatch_parent<'parent>,
                    }
                }
                // for handles with no parent, it is easier to make a method that
                // takes a parent parameter for consistency and just ignoring the param
                else {
                    new_method = quote!{
                        fn new<T>(handle: #handle_name<'static>, _parent: T) -> #owner_name {
                                #owner_name {
                                    handle,
                                    phantom: PhantomData,
                                }
                            }
                    };
                    return_impl = quote!{
                        impl<'parent, T> Return<#owner_name<'parent>> for ((#handle_name<'static>), T) where T: Copy {
                            fn ret(self) -> #owner_name<'parent> {
                                #owner_name::new(self.0, self.1)
                            }
                        }
                        impl<'parent, A: Copy, B: Copy> Return<(A, #owner_name<'parent>)> for ((A, #handle_name<'static>), B) {
                            fn ret(self) -> (A, #owner_name<'parent>) {
                                ((self.0).0, #owner_name::new((self.0).1, self.1))
                            }
                        }
                        impl<'parent, T> Return<Vec<#owner_name<'parent>>> for
                                ((Vec<#handle_name<'static>>), T) where T: Copy {
                            fn ret(self) -> Vec<#owner_name<'parent>> {
                                self.0.iter().copied().map(|handle| ((handle), self.1).ret()).collect()
                            }
                        }
                    };
                    quote!( phantom: ::std::marker::PhantomData<&'parent ()>, )
                };

                quote!{
                    pub struct #owner_name<'parent> {
                        handle: #handle_name<'static>,
                        #dispatch_member
                    }
                    impl #owner_name<'_> {
                        #new_method
                    }
                    #return_impl
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

//fn gnerate_handle_return_code(handle: &Handle)

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

            let alias_ident = if global_data::GLOBAL_DATA.get().expect("error: global_data not set")
                .needs_lifetime.get(alias.as_str()).is_some() {
                    quote!( #alias_ident<'a> )
                }
            else {
                quote!( #alias_ident )
            };

            let tokens = quote! {
                pub type #name_ident<'a> = #alias_ident;
            };
            Some(tokens)
        });
    quote! {
        #(#aliases)*
    }
}


use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::ParseState;
//use crate::commands;

pub fn make_manager_name(name: &str) -> TokenStream {
    format!("{}Manager", name).as_code()
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
                else { // return only type
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

                parse_state.handle_cache.push(&handle);

                match handle.ty {
                    // based on the spec, i understand that dispatchable
                    // handles will be pointers, thus, they will be different
                    // sizes on 32bit and 64 bit computers
                    // but nondispatchable handles will always be 64 bits
                    HandleType::Dispatch => {
                        // each dispatchable handle will have a manager type that will handle
                        // creation and destruciton automatically, and will provide convinience
                        // methods for their respective vulkan commands (i.e. where the respective
                        // handle is the first parameter)
                        //
                        // check commands.rs for method definitions
                        //let manager_name = make_manager_name(handle.name.as_str());

                        //// handle managers will provide convinience usage of dipatchable handles
                        //// define the members that each type should have
                        //// the instance and device managers should hold function pointers
                        //// the other managers should have references to their parent
                        //let manager_members = match handle.name.as_str() {
                        //    "VkInstance" => quote!{
                        //        commands: InstanceCommands,
                        //        phantom: ::std::marker::PhantomData<&'a ()>,
                        //    },
                        //    "VkDevice" => quote!{
                        //        commands: DeviceCommands,
                        //        phantom: ::std::marker::PhantomData<&'a ()>,
                        //    },
                        //    _ => {
                        //        let parent = handle.parent.as_ref().expect("error: expected parent for handle").as_str();
                        //        let parent_manager = make_manager_name(parent);
                        //        quote!{
                        //            parent: &'a #parent_manager<'a>,
                        //        }
                        //    }
                        //};

                        // make the handle type
                        quote!{
                            pub type #handle_name = *const c_void; // object pointer???

                            //pub struct #manager_name<'a> {
                            //    handle: #handle_name,
                            //    #manager_members
                            //    //#( #pfn_params ),*
                            //}
                        }
                    },
                    HandleType::NoDispatch => {
                        //let manager_name = make_manager_name(handle.name.as_str());

                        //let parent_manager = if let Some(parent_name) = handle.parent.as_ref() {
                        //    // NOTE some non-dispatchable handle type can have multiple parents
                        //    // for now, we just take the first parent
                        //    let parent_name = parent_name.as_str().split(',')
                        //        .next()
                        //        .expect("there must be at least one elemet in the parent names");

                        //    let parent_manager = make_manager_name(parent_name);
                        //    quote!{
                        //        parent: &'a #parent_manager<'a>,
                        //    }
                        //}
                        //else {
                        //    quote!( phantom: ::std::marker::PhantomData<&'a ()>, )
                        //};

                        quote!{
                            pub type #handle_name = u64; // uint64_t
                            //pub struct #manager_name<'a> {
                            //    handle: #handle_name,
                            //    #parent_manager
                            //}
                        }
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

fn get_dispatchable_parent_manager(handle: &Handle, handle_cache: &[&Handle]) -> Option<TokenStream> {
    handle.parent.as_ref()
        .and_then(|parent_name| {
            find_in_slice(handle_cache, |handle| handle.name.as_str() == parent_name.as_str())
                .and_then(|handle| match handle.ty {
                    HandleType::Dispatch => Some( make_manager_name(handle.name.as_str()) ),
                    HandleType::NoDispatch => get_dispatchable_parent_manager(handle, handle_cache),
                })
        })
}

pub fn post_process_handles(parse_state: &ParseState) -> TokenStream {

    let managers = parse_state.handle_cache.iter().map(|handle| {

        let handle_name = handle.name.as_code();

        match handle.ty {
            HandleType::Dispatch => {
                // each dispatchable handle will have a manager type that will handle
                // creation and destruciton automatically, and will provide convinience
                // methods for their respective vulkan commands (i.e. where the respective
                // handle is the first parameter)
                //
                // check commands.rs for method definitions
                let manager_name = make_manager_name(handle.name.as_str());

                // handle managers will provide convinience usage of dipatchable handles
                // define the members that each type should have
                // the instance and device managers should hold function pointers
                // the other managers should have references to their parent
                let manager_members = match handle.name.as_str() {
                    "VkInstance" => quote!{
                        commands: InstanceCommands,
                        feature_version: Box<dyn Feature>,
                        phantom: ::std::marker::PhantomData<&'a ()>,
                    },
                    "VkDevice" => quote!{
                        commands: DeviceCommands,
                        parent: &'a PhysicalDeviceManager<'a>,
                    },
                    _ => {
                        let parent_manager = get_dispatchable_parent_manager(&handle, parse_state.handle_cache.as_slice());
                        quote!{
                            parent: &'a #parent_manager<'a>,
                        }
                    }
                };

                let new_method = match handle.name.as_str() {
                    "VkInstance" => quote!{
                        fn new(handle: Instance, commands: InstanceCommands, feature_version: Box<dyn Feature>) -> #manager_name<'a> {
                            #manager_name {
                                handle,
                                commands,
                                feature_version,
                                phantom: ::std::marker::PhantomData,
                            }
                        }
                    },
                    "VkDevice" => quote!{
                        fn new(handle: Device, commands: DeviceCommands, parent: &'a PhysicalDeviceManager) -> #manager_name<'a> {
                            #manager_name {
                                handle,
                                commands,
                                parent,
                            }
                        }
                    },
                    _ => {
                        let parent_manager = get_dispatchable_parent_manager(&handle, parse_state.handle_cache.as_slice());
                        quote!{
                            fn new<'parent>(handle: #handle_name, parent: &'parent #parent_manager) -> #manager_name<'a>
                                where 'parent: 'a {
                                #manager_name {
                                    handle,
                                    parent,
                                }
                            }
                        }
                    }
                };

                // make the handle manager
                quote!{
                    pub struct #manager_name<'a> {
                        handle: #handle_name,
                        #manager_members
                        //#( #pfn_params ),*
                    }
                    impl<'a> #manager_name<'a> {
                        #new_method
                    }
                }
            }
            HandleType::NoDispatch => {
                let manager_name = make_manager_name(handle.name.as_str());

                let new_method;
                let parent_manager = if let Some(parent_name) = handle.parent.as_ref() {
                    // NOTE some non-dispatchable handle type can have multiple parents
                    // for now, we just take the first parent
                    let parent_name = parent_name.as_str().split(',')
                        .next()
                        .expect("there must be at least one elemet in the parent names");

                    // NOTE in order to make code generation easier (especially regarding method
                    // creation), we make the device a parent to the swapchain types (rather the
                    // actual parent which is the surface). This is because the surface manager is
                    // not easily available in the swapchain create methods
                    let parent_manager;
                    if handle.name.as_str() == "VkSwapchainKHR" {
                        parent_manager = quote!(DeviceManager);
                    }
                    else {
                        parent_manager = make_manager_name(parent_name);
                    }

                    new_method = quote!{
                        fn new<'parent>(handle: #handle_name, parent: &'parent #parent_manager) -> #manager_name<'a>
                            where 'parent: 'a {
                                #manager_name {
                                    handle,
                                    parent,
                                }
                            }
                    };

                    quote!{
                        parent: &'a #parent_manager<'a>,
                    }
                }
                else {
                    new_method = quote!{
                        fn new<'parent, T>(handle: #handle_name, _parent: &'parent T) -> #manager_name<'a>
                            where 'parent: 'a {
                                #manager_name {
                                    handle,
                                    phantom: ::std::marker::PhantomData,
                                }
                            }
                    };
                    quote!( phantom: ::std::marker::PhantomData<&'a ()>, )
                };

                quote!{
                    pub struct #manager_name<'a> {
                        handle: #handle_name,
                        #parent_manager
                    }
                    impl<'a> #manager_name<'a> {
                        #new_method
                    }
                }
            }
        }
    });

    quote!{
        #( #managers )*
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

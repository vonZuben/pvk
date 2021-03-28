
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;

use crate::stct;
use crate::ty;

use crate::utils;
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
                    vk_bitflags_wrapped!(#name);
                }
            },
            DefinitionsElement::Struct(stct) => {
                let name = stct.name.as_code();

                let public_lifetime = "'public";
                let private_lifetime = "'private";

                let not_stype_pnext = |field: &&vkxml::Field| {
                    let fname = utils::field_name_expected(field);
                    fname != "sType" && fname != "pNext"
                };

                let not_return = |stct: &Struct| {
                    if stct.name.contains("BaseOutStructure") || stct.name.contains("BaseInStructure") {
                        false
                    }
                    else {
                        !stct.is_return
                    }
                };

                let fields = stct.elements.iter().filter_map(variant!(StructElement::Member))
                    .map(|field| {
                        let ty = CType::new(field)
                            .public_lifetime(public_lifetime)
                            .private_lifetime(private_lifetime)
                            .context(FieldContext::Member)
                            .as_field();
                        if not_stype_pnext(&field) {
                            ty.public()
                        }
                        else {
                            ty
                        }
                    });

                let type_lifetime = global_data::type_lifetime(stct.name.as_str()).unwrap_or_default();

                let gen_struct = pipe!{ st = stct::Struct::new(&stct.name) =>
                    STAGE {
                        st.public()
                            .attribute(quote!(#[repr(C)]))
                            .attribute(quote!(#[derive(Copy, Clone)]))
                    }
                    STAGE {
                        st.fields(fields)
                    }
                    WHEN type_lifetime.public => {
                        st.lifetime_param(public_lifetime)
                    }
                    WHEN type_lifetime.private => {
                        st.lifetime_param(private_lifetime)
                    }
                    WHEN not_return(&stct) => {
                        st.setters(stct.elements.iter().filter_map(variant!(StructElement::Member)).filter(not_stype_pnext))
                    }
                };

                // gererate bulders and initializers for only non return types
                let builder_code = if not_return(&stct) {

                    let struct_field = |field| {
                        utils::Rtype::new(field, stct.name.as_str())
                        .public_lifetime(public_lifetime)
                        .private_lifetime(private_lifetime)
                        .ref_lifetime(private_lifetime)
                        .context(FieldContext::Member)
                        .as_field()
                    };

                    let must_init_members: Vec<_> = stct.elements.iter().filter_map(variant!(StructElement::Member))
                        .filter(|field| utils::must_init(&stct.name, field))
                        .filter(not_stype_pnext)
                        .map(struct_field)
                        .collect();

                    let optional_members: Vec<_> = stct.elements.iter().filter_map(variant!(StructElement::Member))
                        .filter(|field| utils::is_optional(&stct.name, field))
                        .filter(not_stype_pnext)
                        .map(struct_field)
                        .collect();

                    let param_rules = stct.elements.iter().filter_map(variant!(StructElement::Member))
                        .filter(not_stype_pnext)
                        .map(|field| {
                            let param = case::camel_to_snake(utils::field_name_expected(field)).as_code();

                            let raw_name = utils::field_name_expected(field);

                            let count_setter = field.size.as_ref()
                                .map(|size| {
                                    if raw_name == "pCode" {
                                        Some( quote!( { code_size , #param ; as usize * 4 } ) )
                                    }
                                    else if raw_name == "pSampleMask" {
                                        None
                                    }
                                    else {
                                        match field.array.as_ref().expect("error: field with size but not an array") {
                                            ArrayType::Static => return None,
                                            _ => {}
                                        }
                                        let count_name = case::camel_to_snake(size).as_code();
                                        Some( quote!( { #count_name , #param ; as _ } ) )
                                    }
                                });

                            if is_optional(&stct.name, field) {
                                quote!{
                                    ( @munch { #param: $val:expr $( , $( $rest:tt )* )? }
                                        -> { $( $optional:tt )* } ; { $( $nonoptional:tt )* } ; { $($count_setters:tt)* } ) => {

                                        $crate::#name!( @munch { $($($rest)*)* }
                                                     -> { $($optional)* #param:$val , } ; { $($nonoptional)* } ;
                                                         { $($count_setters)* #count_setter } )
                                    };
                                }
                            }
                            else { // must_init
                                quote!{
                                    ( @munch { #param: $val:expr $( , $( $rest:tt )* )? }
                                        -> { $( $optional:tt )* } ; { $( $nonoptional:tt )* } ; { $($count_setters:tt)* } ) => {

                                        $crate::#name!( @munch { $($($rest)*)* }
                                                     -> { $($optional)* } ; { $($nonoptional)* #param:$val , } ;
                                                         { $($count_setters)* #count_setter } )
                                    };
                                }
                            }
                        });

                    let must_init_move = stct.elements.iter().filter_map(variant!(StructElement::Member))
                        .filter(|field| utils::must_init(&stct.name, field))
                        .filter(not_stype_pnext)
                        .map(|field| {
                            let set_field = format!("set_{}", case::camel_to_snake(utils::field_name_expected(field))).as_code();
                            let field = case::camel_to_snake(utils::field_name_expected(field)).as_code();
                            quote!{
                                fin.#set_field(init.#field);
                            }
                        });

                    let optional_move = stct.elements.iter().filter_map(variant!(StructElement::Member))
                        .filter(|field| utils::is_optional(&stct.name, field))
                        .filter(not_stype_pnext)
                        .map(|field| {
                            let set_field = format!("set_{}", case::camel_to_snake(utils::field_name_expected(field))).as_code();
                            let field = case::camel_to_snake(utils::field_name_expected(field)).as_code();
                            quote!{
                                fin.#set_field(opt.#field);
                            }
                        });

                    Some(quote!{
                        #[macro_export]
                        macro_rules! #name {

                            ( @munch {} ->
                                    { $( $o_name:ident : $o_val:expr ),* $(,)? } ;
                                    { $( $nono_name:ident : $nono_val:expr ),* $(,)? } ;
                                    { $( $count_setters:tt )* }) => {
                                {
                                    use $crate::*;
                                    use std::marker::PhantomData;
                                    mod vk {
                                        use $crate::*;
                                        use std::marker::PhantomData;
                                        pub struct #name<'public, 'private> {
                                            #( pub #must_init_members , )*
                                            pub _p1: PhantomData<&'public ()>,
                                            pub _p2: PhantomData<&'private ()>,
                                        }
                                    }
                                    #[derive(Default)]
                                    struct Opt<'public, 'private> {
                                        #( #optional_members , )*
                                        _p1: PhantomData<&'public ()>,
                                        _p2: PhantomData<&'private ()>,
                                    }

                                    struct Combined<'public, 'private> {
                                        #( #must_init_members , )*
                                        #( #optional_members , )*
                                        _p1: PhantomData<&'public ()>,
                                        _p2: PhantomData<&'private ()>,
                                    }

                                    let init = vk::#name {
                                        $( $nono_name: $nono_val, )*
                                        _p1: PhantomData,
                                        _p2: PhantomData,
                                    };

                                    #[allow(unused_mut)]
                                    let mut opt = Opt::default();
                                    $( opt.$o_name = $o_val; )*

                                    let mut fin = #name::uninit();
                                    #(#must_init_move)*
                                    #(#optional_move)*

                                    fin
                                }
                            };

                            ( @munch { s_type $($restin:tt)* } $($rest:tt)* ) => {
                                compile_error!("s_type param is automatically set. Do not set manually");
                            };

                            ( @munch { p_next $($restin:tt)* } $($rest:tt)* ) => {
                                compile_error!("p_next param is automatically set. Do not set manually. Use extend() to add extension structs instead");
                            };

                            // match each param as optional or nonoptional
                            #( #param_rules )*

                            // add unrecognized parans to 'non-optional' params
                            // so that they will be detected as non existent members
                            ( @munch { $unrecognized:ident : $val:expr $( , $( $rest:tt )* )? } ->
                                { $( $optional:tt )* } ; { $( $nonoptional:tt )* } ; { $($count_setters:tt)* } ) => {
                                    $crate::#name!( @munch { $($($rest)*)* } ->
                                            { $($optional)* } ; { $($nonoptional)* $unrecognized:$val , } ;
                                            { $($count_setters)* } )
                            };

                            // entry point
                            // transform input into -> { input } -> { optional } ; { nonoptional }
                            //      ; { count_setters }
                            ( $( $( $name:ident : $val:expr ),+ $(,)? )? ) => {
                                $crate::#name!( @munch { $($( $name : $val , )+)? } -> {} ; {} ; {} )
                            };

                        }
                    })
                }
                else { // return only type
                    None
                };

                fn is_base(stct: &Struct) -> bool {
                    if stct.name.as_str() == "VkBaseOutStructure" || stct.name.as_str() == "VkBaseInStructure" {
                        return false;
                    }
                    for field in stct.elements.iter().filter_map(variant!(StructElement::Member)) {
                        if field.name.as_ref().map(String::as_str) == Some("pNext") {
                            return true;
                        }
                    }
                    false
                }

                fn is_ex(stct: &Struct) -> bool {
                    is_base(stct) && stct.extends.is_some()
                }

                let get_st = || {
                    for field in stct.elements.iter().filter_map(variant!(StructElement::Member)) {
                        if field.name.as_ref().map(String::as_str) == Some("sType") {
                            return utils::structure_type_name(field);
                        }
                    }
                    panic!("error: no sType in {}", stct.name);
                };

                let impl_base = if is_base(stct) {
                    let name = stct.name.as_code();
                    let st = get_st().as_code();
                    Some(
                        quote! {
                            unsafe impl<'public> Base<'public> for #name<'public, '_> {
                                const ST: StructureType = StructureType::#st;
                            }
                            impl<'public, 'private> StypeInit<'public> for #name<'public, 'private> {}
                            impl AddChain for #name<'static, '_> {}
                        }
                    )
                }
                else {
                    None
                };

                let ex_trait;
                let impl_struct;
                if global_data::is_extendable(&stct.name) {
                    let ex_trait_name = utils::ex_trait_name(&stct.name).as_code();
                    ex_trait = Some(
                        quote! {
                            pub trait #ex_trait_name<'public> : Base<'public> {}
                        }
                    );

                    let name = stct.name.as_code();
                    impl_struct = Some(
                        quote!{
                            impl<'public, 'private> #name <'public, 'private> {
                                fn extend<E: #ex_trait_name <'public> > (&mut self, e: &'private mut E) {
                                    self.p_next.push(e);
                                }
                            }
                        }
                    );
                }
                else {
                    ex_trait = None;
                    impl_struct = None;
                }

                let impl_ex_trait: Vec<_> = if is_ex(stct) {
                    let name = stct.name.as_code();
                    stct.extends.as_ref().unwrap().split(',').map(|extends| {
                        let ex_trait_name = utils::ex_trait_name(extends).as_code();
                        let extends = extends.as_code();
                        quote! {
                            impl<'public> #ex_trait_name<'public> for #name<'public, '_> {}
                            impl PnLink<#extends<'static, 'static>> for #name<'static, 'static> {}
                        }
                    }).collect()
                }
                else {
                    Vec::new()
                };

                fn filter_redundent(field: &&Field) -> bool {
                    if field.name.as_ref().map(String::as_str) == Some("pNext") || field.name.as_ref().map(String::as_str) == Some("sType"){
                        return false;
                    }
                    true
                }

                let name = stct.name.as_code();
                let name_str = stct.name.as_str();

                let field_names = stct.elements.iter()
                    .filter_map(variant!(StructElement::Member))
                    .filter(filter_redundent)
                    .map(|field|{
                        utils::formatted_field_name(field)
                    });
                let field_members = stct.elements.iter()
                    .filter_map(variant!(StructElement::Member))
                    .filter(filter_redundent)
                    .map(|field|{
                        utils::formatted_field_name(field).as_code()
                    });

                let mut generics = ty::Generics::default();
                if type_lifetime.public { generics.push_lifetime_param("'_") }
                if type_lifetime.private { generics.push_lifetime_param("'_") }

                // let p_next_fmt = if is_base(stct) {
                //     Some(
                //         quote! { // inside Debug::fmt(self, f)
                //             self.p_next.fmt(f)?;
                //         }
                //     )
                // }
                // else {
                //     None
                // };

                let impl_debug = quote! {
                    impl fmt::Debug for #name #generics {
                        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                            f.debug_struct(#name_str)
                                #( .field(#field_names, &self.#field_members) )*
                                .finish()
                            // #p_next_fmt
                            // Ok(())
                        }
                    }
                };

                quote!{
                    #gen_struct
                    #impl_struct
                    #impl_base
                    #ex_trait
                    #(#impl_ex_trait)*
                    #impl_debug
                    #builder_code
                }
            },
            DefinitionsElement::Union(uni) => {
                let name = uni.name.as_code();
                let params = uni.elements.iter().map(|field|c_field(field, WithLifetime::Yes("'handle"), FieldContext::Member));

                let possible_value = uni.elements.iter().map(|field| {
                    case::camel_to_snake(utils::field_name_expected(field)).as_code()
                });

                quote!{
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    pub union #name {
                        #( #params ),*
                    }

                    /// since we cannot know how to interpret the uniion when printing
                    /// we just preint out every possible interpretation
                    impl ::std::fmt::Debug for #name {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                            f.debug_struct(concat!(stringify!(#name), " (possible interpretations)"))
                                #( .field(stringify!(#possible_value), unsafe{ &self.#possible_value }) )*
                                .finish()
                        }
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

                let send_or_sync_impl = {
                    if global_data::is_handle_not_sync_and_send(handle.name.as_str()) {
                        None
                    }
                    else if global_data::is_handle_not_sync(handle.name.as_str()) {
                        Some( quote!{
                            unsafe impl Send for #handle_name<'_> {}
                        })
                    }
                    else { // handle should be send and sync by default
                        Some( quote!{
                            unsafe impl Send for #handle_name<'_> {}
                            unsafe impl Sync for #handle_name<'_> {}
                        })
                    }
                };

                quote!{
                    #[derive(Clone, Copy)]
                    #[repr(transparent)]
                    pub struct #handle_name<'owner> {
                        handle: raw::#handle_name,
                        // when a Handleowner provides the inner handle for use, the provided
                        // handle borrows the owner. This allows us to keep the owner borrowed
                        // which is helpful especiially when creating new handleOwners so that
                        // Memory does not outlive Device for example
                        _parent_ref: PhantomData<&'owner #owner_name<'owner>>,
                        // we will manually implemet Send and Sync for all handles which can be
                        // send or sync
                        _manual_send_sync: PhantomData<*const ()>,
                    }
                    impl Handle for #handle_name<'_> {}
                    #send_or_sync_impl
                    impl Default for #handle_name<'_> {
                        fn default() -> Self {
                            // should be fine as long as VK_NULL_HANDLE == 0
                            // TODO maybe just use VK_NULL_HANDLE
                            unsafe { std::mem::zeroed() }
                        }
                    }
                    impl ::std::fmt::Debug for #handle_name<'_> {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                            write!(f, concat!(stringify!(#handle_name), "({:?})"), self.handle)
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
                let fptr_name_str = &fptr.name;
                let fptr_name = (fptr.name.to_string() + "_fpointer").as_code();
                let return_type = c_type(&fptr.return_type, WithLifetime::No, FieldContext::Member) // TODO: should this be FieldContext::FunctionParam?
                                                .is_return_type(true);
                let params = fptr.param.iter().map(|field|c_field(field, WithLifetime::No, FieldContext::FunctionParam));

                // we wrap the funtion pointer in a transparent struct so we can implement traits for it such as Debug
                let name_wrapper = fptr.name.as_code();

                quote!{
                    #[allow(non_camel_case_types)]
                    //pub type #name #lifetime = extern "system" fn(
                    pub type #fptr_name = extern "system" fn(
                        #( #params ),*
                        ) -> #return_type;

                    #[repr(transparent)]
                    #[derive(Clone, Copy)]
                    pub struct #name_wrapper(Option<#fptr_name>);

                    impl #name_wrapper {
                        fn take(self) -> Option<#fptr_name> {
                            self.0
                        }
                    }

                    // simply print the name of the function pointer
                    // there is'nt much else hepful to print
                    impl ::std::fmt::Debug for #name_wrapper {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                            write!(f, #fptr_name_str)
                        }
                    }
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
                        dispatch_parent: &'parent InstanceOwner<'parent>,
                    },
                    _ => {
                        quote!{
                            dispatch_parent: &'parent #dispatch_parent<'parent>,
                        }
                    }
                };

                let implements = match handle.name.as_str() {
                    "VkInstance" => quote!{
                        impl<'parent, Own> CreateOwner<'parent> for #owner_name<'parent, Own> {
                            type Handle = #handle_name<'static>;
                            type DispatchParent = ();
                            fn new(handle: Self::Handle, _dispatch_parent: &'parent Self::DispatchParent) -> Self {
                                #owner_name {
                                    handle,
                                    commands: InstanceCommands::new(),
                                    feature_version: Box::new(VERSION_1_0),
                                    _parent: PhantomData,
                                    _is_owned: PhantomData,
                                }
                            }
                            fn disassemble(self) -> (Self::Handle, &'parent Self::DispatchParent) {
                                let ret = (self.handle, &());
                                ::std::mem::forget(self);
                                ret
                            }
                        }
                    },
                    "VkDevice" => quote!{
                        impl<'parent, Own> CreateOwner<'parent> for #owner_name<'parent, Own> {
                            type Handle = #handle_name<'static>;
                            type DispatchParent = InstanceOwner<'parent>;
                            fn new(handle: Self::Handle, dispatch_parent: &'parent Self::DispatchParent) -> Self {
                                #owner_name {
                                    handle,
                                    commands: DeviceCommands::new(),
                                    dispatch_parent,
                                    _is_owned: PhantomData,
                                }
                            }
                            fn disassemble(self) -> (Self::Handle, &'parent Self::DispatchParent) {
                                let ret = (self.handle, self.dispatch_parent);
                                ::std::mem::forget(self);
                                ret
                            }
                        }
                    },
                    _ => {
                        quote!{
                            impl<'parent, Own> CreateOwner<'parent> for #owner_name<'parent, Own> {
                                type Handle = #handle_name<'static>;
                                type DispatchParent = #dispatch_parent<'parent>;
                                fn new(handle: Self::Handle, dispatch_parent: &'parent Self::DispatchParent) -> Self {
                                    #owner_name {
                                        handle,
                                        dispatch_parent,
                                        _is_owned: PhantomData,
                                    }
                                }
                                fn disassemble(self) -> (Self::Handle, &'parent Self::DispatchParent) {
                                    let ret = (self.handle, self.dispatch_parent);
                                    ::std::mem::forget(self);
                                    ret
                                }
                            }
                        }
                    }
                };

                let return_impl = match handle.name.as_str() {
                    "VkInstance" | "VkDevice" => None,
                    _ => Some(quote!{
                        impl<'parent, Own> Return<#owner_name<'parent, Own>> for ((#handle_name<'static>), &'parent #dispatch_parent<'_>) {
                            fn ret(self) -> #owner_name<'parent, Own> {
                                #owner_name::new(self.0, self.1)
                            }
                        }
                        impl<'parent, Own> Return<Vec<#owner_name<'parent, Own>>> for
                                ((Vec<#handle_name<'static>>), &'parent #dispatch_parent<'_>) {
                            fn ret(self) -> Vec<#owner_name<'parent, Own>> {
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
                    #[repr(C)]
                    pub struct #owner_name<'parent, Own=Borrowed> where Own: 'static {
                        // the interpretation of this is that nothing is acutally borrowed, and nothing is 'static
                        handle: #handle_name<'static>,
                        #owner_members
                        _is_owned: PhantomData<Own>,
                        //#( #pfn_params ),*
                    }
                    #implements
                    impl<'owner> HandleOwner<'owner> for #owner_name<'_> {
                        type Handle = #handle_name<'owner>;
                        fn handle(&'owner self) -> #handle_name<'owner> {
                            self.handle
                        }
                    }
                    impl<Own: ::std::fmt::Debug + Default> ::std::fmt::Debug for #owner_name<'_, Own> {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                            write!(f, concat!(stringify!(#owner_name), "({:?})({:?})"), self.handle.handle, Own::default())
                        }
                    }
                    // Notes: There transumtes should be completely safe, especiialy since we use C repr
                    impl<'parent> ::std::ops::Deref for #owner_name<'parent, Owned> {
                        type Target = #owner_name<'parent, Borrowed>;
                        fn deref(&self) -> &Self::Target {
                            unsafe { ::std::mem::transmute::<&#owner_name<'parent, Owned>, &#owner_name<'parent, Borrowed>>(self) }
                        }
                    }
                    impl<'parent> ::std::ops::Deref for #owner_name<'parent, ManuallyManaged> {
                        type Target = #owner_name<'parent, Borrowed>;
                        fn deref(&self) -> &Self::Target {
                            unsafe { ::std::mem::transmute::<&#owner_name<'parent, ManuallyManaged>, &#owner_name<'parent, Borrowed>>(self) }
                        }
                    }
                    impl<Own> ConvertToC<#handle_name<'static>> for #owner_name<'_, Own> {
                        fn to_c(self) -> #handle_name<'static> {
                            self.disassemble().0
                        }
                    }
                    #return_impl
                }
            }
            HandleType::NoDispatch => {
                let owner_name = make_handle_owner_name(handle.name.as_str());

                let implements;
                let return_impl;
                let dispatch_member = if let Some(parent_name) = handle.parent.as_ref() {
                    // NOTE some non-dispatchable handle type can have multiple parents
                    // for now, we just take the first parent
                    let _ = parent_name.as_str().split(',')
                        .next()
                        .expect("there must be at least one elemet in the parent names");

                    let dispatch_parent = get_dispatchable_parent(&handle, parse_state.handle_cache.as_slice())
                            .unwrap_or(quote!(DeviceOwner));

                    implements = quote!{
                        impl<'parent, Own> CreateOwner<'parent> for #owner_name<'parent, Own> {
                            type Handle = #handle_name<'static>;
                            type DispatchParent = #dispatch_parent<'parent>;
                            fn new(handle: Self::Handle, dispatch_parent: &'parent Self::DispatchParent) -> Self {
                                #owner_name {
                                    handle,
                                    dispatch_parent,
                                    _is_owned: PhantomData,
                                }
                            }
                            fn disassemble(self) -> (Self::Handle, &'parent Self::DispatchParent) {
                                let ret = (self.handle, self.dispatch_parent);
                                ::std::mem::forget(self);
                                ret
                            }
                        }
                    };

                    return_impl = quote!{
                        impl<'parent, Own> Return<#owner_name<'parent, Own>> for ((#handle_name<'static>), &'parent #dispatch_parent<'_>) {
                            fn ret(self) -> #owner_name<'parent, Own> {
                                #owner_name::new(self.0, self.1)
                            }
                        }
                        impl<'parent, Own> Return<Vec<#owner_name<'parent, Own>>> for
                                ((Vec<#handle_name<'static>>), &'parent #dispatch_parent<'_>) {
                            fn ret(self) -> Vec<#owner_name<'parent, Own>> {
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
                    implements = quote!{
                        impl<'parent, Own> CreateOwner<'parent> for #owner_name<'parent, Own> {
                            type Handle = #handle_name<'static>;
                            type DispatchParent = PhysicalDeviceOwner<'parent>;
                            fn new(handle: Self::Handle, dispatch_parent: &'parent Self::DispatchParent) -> Self {
                                #owner_name {
                                    handle,
                                    // phantom: PhantomData,
                                    dispatch_parent,
                                    _is_owned: PhantomData,
                                }
                            }
                            fn disassemble(self) -> (Self::Handle, &'parent Self::DispatchParent) {
                                let ret = (self.handle, self.dispatch_parent);
                                ::std::mem::forget(self);
                                ret
                            }
                        }
                    };
                    return_impl = quote!{
                        impl<'parent, Own> Return<#owner_name<'parent, Own>> for ((#handle_name<'static>), &'parent PhysicalDeviceOwner<'_>) {
                            fn ret(self) -> #owner_name<'parent, Own> {
                                #owner_name::new(self.0, self.1)
                            }
                        }
                        impl<'parent, Own, A: Copy> Return<(A, #owner_name<'parent, Own>)> for ((A, #handle_name<'static>), &'parent PhysicalDeviceOwner<'_>) {
                            fn ret(self) -> (A, #owner_name<'parent, Own>) {
                                ((self.0).0, #owner_name::new((self.0).1, self.1))
                            }
                        }
                        impl<'parent, Own> Return<Vec<#owner_name<'parent, Own>>> for
                                ((Vec<#handle_name<'static>>), &'parent PhysicalDeviceOwner<'_>) {
                            fn ret(self) -> Vec<#owner_name<'parent, Own>> {
                                self.0.iter().copied().map(|handle| ((handle), self.1).ret()).collect()
                            }
                        }
                    };
                    quote!( dispatch_parent: &'parent PhysicalDeviceOwner<'parent>, )
                    // quote!( phantom: ::std::marker::PhantomData<&'parent ()>, )
                };

                quote!{
                    #[repr(C)]
                    pub struct #owner_name<'parent, Own=Borrowed> where Own: 'static {
                        handle: #handle_name<'static>,
                        #dispatch_member
                        _is_owned: PhantomData<Own>,
                    }
                    #implements
                    impl<'owner> HandleOwner<'owner> for #owner_name<'_> {
                        type Handle = #handle_name<'owner>;
                        fn handle(&'owner self) -> #handle_name<'owner> {
                            self.handle
                        }
                    }
                    impl<Own: ::std::fmt::Debug + Default> std::fmt::Debug for #owner_name<'_, Own> {
                        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(f, concat!(stringify!(#owner_name), "({:?})({:?})"), self.handle.handle, Own::default())
                        }
                    }
                    impl<'parent> ::std::ops::Deref for #owner_name<'parent, Owned> {
                        type Target = #owner_name<'parent, Borrowed>;
                        fn deref(&self) -> &Self::Target {
                            unsafe { ::std::mem::transmute::<&#owner_name<'parent, Owned>, &#owner_name<'parent, Borrowed>>(self) }
                        }
                    }
                    impl<'parent> ::std::ops::Deref for #owner_name<'parent, ManuallyManaged> {
                        type Target = #owner_name<'parent, Borrowed>;
                        fn deref(&self) -> &Self::Target {
                            unsafe { ::std::mem::transmute::<&#owner_name<'parent, ManuallyManaged>, &#owner_name<'parent, Borrowed>>(self) }
                        }
                    }
                    impl<Own> ConvertToC<#handle_name<'static>> for #owner_name<'_, Own> {
                        fn to_c(self) -> #handle_name<'static> {
                            self.disassemble().0
                        }
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

            let mut generics = ty::Generics::default();

            let type_lifetime = global_data::type_lifetime(alias.as_str()).unwrap_or_default();

            if type_lifetime.public {
                generics.push_lifetime_param("'public")
            }
            if type_lifetime.private {
                generics.push_lifetime_param("'private")
            }

            let tokens = quote! {
                pub type #name_ident #generics = #alias_ident #generics;
            };
            Some(tokens)
        });
    quote! {
        #(#aliases)*
    }
}

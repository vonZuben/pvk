
use quote::{quote, ToTokens};

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::{global_data, utils::*};
use crate::utils;
use crate::ty;

use std::collections::HashMap;

use crate::definitions;

#[derive(Default)]
pub struct Commands2<'a> {
    function_pointers: Vec<definitions::FunctionPointer<'a>>,
}

// impl<'a, I: IntoIterator<Item=definitions::FunctionPointer<'a>>> From<I> for Commands2<'a> {
//     fn from(i: I) -> Self {
//         Self {
//             function_pointers: i.into_iter().collect(),
//         }
//     }
// }

impl<'a> Commands2<'a> {
    pub fn push(&mut self, function_pointer: definitions::FunctionPointer<'a>) {
        self.function_pointers.push(function_pointer);
    }
}

impl ToTokens for Commands2<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let function_pointers = &self.function_pointers;
        let commands = self.function_pointers.iter().map(|fptr|fptr.name);
        quote!(
            #(#function_pointers)*
            macro_rules! use_command_function_pointer_names {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#commands),* );
                }
            }
        ).to_tokens(tokens);
    }
}

pub fn make_pfn_name(cmd_name: &str) -> TokenStream {
    format!("PFN_{}", cmd_name).as_code()
}

pub fn make_pfn_loader_name(cmd_name: &str) -> TokenStream {
    format!("PFN_Loader_{}", cmd_name).as_code()
}

//#[derive(Debug)]
//struct CommandParts<'a> {
//    verb: &'a str,
//}
//
//fn get_command_parts(cmd: &Command) -> CommandParts {
//    let second_capital = cmd.name[3..].find(char::is_uppercase).expect("can't find delimiter to command verb") + 3;
//    let verb = &cmd.name[2..second_capital];
//    CommandParts {
//        verb
//    }
//}

#[derive(Debug, Clone, Copy)]
pub enum CommandCategory {
    Device,
    Instance,
    Static,
    Entry,
    DoNotGenerate,
}

pub fn command_category(cmd: &Command) -> CommandCategory {
    match cmd.name.as_str() {
        "vkGetInstanceProcAddr" | "vkGetDeviceProcAddr" => CommandCategory::Static,
        "vkEnumerateInstanceVersion" => CommandCategory::DoNotGenerate, // this function is manually created in lib.rs in order to support VK 1.0
        _ =>
            match cmd.param[0].basetype.as_str() {
                "VkDevice" | "VkCommandBuffer" | "VkQueue" => CommandCategory::Device,
                "VkInstance" | "VkPhysicalDevice" => CommandCategory::Instance,
                _ => CommandCategory::Entry,
            }
    }
}

// fn field_first_indirection_optional(field: &vkxml::Field) -> bool {
//     if field.basetype.as_str() == "char" {
//         return false;
//     }
//     field.optional.as_ref().map(|option_type| {
//         // vkxml optional is a comma separated list representing if each level of
//         // indirection is optional
//         //
//         // we only care about the first level of indirection
//         let first_indirection_optional = option_type.split(',')
//             .next().expect(format!("error: empty optional? -> {:?}", field.name).as_str());

//         match first_indirection_optional {
//             "true" => true,
//             "false" => false,
//             _ => panic!(format!("error: optional type not true or false -> {:?}", field.name)),
//         }
//     }).unwrap_or(false)
// }

fn field_name(field: &vkxml::Field) -> &str {
    field.name.as_ref()
        .expect("error: field with no name").as_str()
}

pub fn handle_commands<'a>(commands: &'a Commands) -> TokenStream {

    macro_rules! filter_varients {
        ( $( $varient:tt )* ) => {
            |cmd| {
                match command_category(&cmd) {
                    $( $varient )* => true,
                    _ => false,
                }
            }
        }
    }

    let instance_commands = commands.elements.iter().filter(filter_varients!(CommandCategory::Instance));
    let device_commands = commands.elements.iter().filter(filter_varients!(CommandCategory::Device));
    let non_static_commands = commands.elements.iter().filter(
        filter_varients!(CommandCategory::Instance | CommandCategory::Device | CommandCategory::Entry));
    let static_commands = commands.elements.iter().filter(filter_varients!(CommandCategory::Static));

    // parameters are made the same for instance and device commands
    // but they need to be separated
    fn make_param(cmd: &Command) -> TokenStream {
        let name = cmd.name.as_code();
        let pfn_loader_name = make_pfn_loader_name(cmd.name.as_str());
        quote!( #name : #pfn_loader_name )
    }
    let instance_command_params = instance_commands.clone().map(make_param);
    let device_command_params = device_commands.clone().map(make_param);

    fn make_cmd_inits(cmd: &Command) -> TokenStream {
        let name = cmd.name.as_code();
        let pfn_loader_name = make_pfn_loader_name(cmd.name.as_str());
        quote!( #name : #pfn_loader_name::new() )
    }
    let instance_cmd_inits = instance_commands.clone().map(make_cmd_inits);
    let device_cmd_inits = device_commands.clone().map(make_cmd_inits);

    // make definitions for instance and device (non-static) commands
    let non_static_command_definitions = non_static_commands.map(|cmd| {
        let name = cmd.name.as_code();
        let pfn_name = make_pfn_name(cmd.name.as_str());
        let pfn_loader_name = make_pfn_loader_name(cmd.name.as_str());
        let raw_name = &cmd.name;

        let return_type = c_type(&cmd.return_type, WithLifetime::No, FieldContext::Member)
                                    .is_return_type(true);
        let params: Vec<_> = cmd.param.iter().map(|field|c_field(field, WithLifetime::No, FieldContext::FunctionParam)).collect();
        //let params2 = params1.clone(); // because params is needed twice and quote will consume params1

        // create owner methods
        let owner_method = make_owner_method(&cmd);

        let entry_loader = if matches!(command_category(cmd), CommandCategory::Entry) {
            Some(
                quote!{
                    struct #name;
                    impl #name {
                        fn call() -> #pfn_name {
                            use std::sync::Once;
                            static LOAD: Once = Once::new();
                            static mut PFN: MaybeUninit<#pfn_loader_name> = MaybeUninit::uninit();
                            unsafe {
                                LOAD.call_once(||{
                                    let loader = |raw_cmd_name: &CStr| unsafe { GetInstanceProcAddr(Default::default(), raw_cmd_name.to_c()) };
                                    let mut pfn = #pfn_loader_name::new();
                                    pfn.load(loader);
                                    PFN.as_mut_ptr().write(pfn)
                                });
                                PFN.as_ptr().read().call()
                            }
                        }
                    }
                }
            )
        }
        else {
            None
        };

        quote!{
            #[allow(non_camel_case_types)]
            pub type #pfn_name = extern "system" fn(
                #( #params ),*
            ) -> #return_type;

            struct #pfn_loader_name(#pfn_name);
            impl #pfn_loader_name {
                fn new() -> Self {
                    extern "system" fn default_function( #( #params ),* ) -> #return_type {
                        panic!(concat!(#raw_name, " is not loaded. Make sure the correct feature/extension is enabled"))
                    }
                    Self(default_function)
                }
                // this function is unsafe since the caller (in general) must ensure that the command loader is
                // not aliased
                // in practice, this only needs to be considered when creating a DeviceOwner,
                // wherein the InstanceOwner must not be aliased since some device extensions might load
                // some instance commands in the InstanceOwner which is shared
                // Further note, the only real concern is if a "torn" value can be observed
                // i.e. if a cpu platform writes or reads a function pointer non-atomically, then
                // it may be possible to use a function pointer which is only partially written
                // causing massive UB
                // but if you are confident that your platform writes/reads function pointers
                // atomically, then there is no real issue here, and synchronization should be safe
                // to ignore
                fn load<F>(&mut self, mut f: F) where F: FnMut(&::std::ffi::CStr) -> PFN_vkVoidFunction {
                    let cname = ::std::ffi::CString::new(#raw_name).unwrap();
                    let function_pointer = f(&cname).take();
                    if let Some(fptr) = function_pointer {
                        self.0 = unsafe { ::std::mem::transmute(fptr) };
                    }
                    else{
                        panic!(concat!("error: couldn't load ", #raw_name));
                    }
                }
                fn call(&self) -> #pfn_name {
                    self.0
                }
            }
            //impl std::fmt::Debug for #pfn_loader_name {
            //    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            //        write!(f, "Loader for: {}", stringify!(#pfn_name))
            //    }
            //}
            #entry_loader
            #owner_method
        }
    });

    let static_command_definitions = static_commands.map(|cmd| {
        let name = cmd.name.as_code();
        let return_type = c_type(&cmd.return_type, WithLifetime::No, FieldContext::Member)
                                    .is_return_type(true);
        let params1 = cmd.param.iter().map(|field|c_field(field, WithLifetime::No, FieldContext::FunctionParam));
        let params2 = params1.clone();

        let pfn_name = make_pfn_name(cmd.name.as_str());

        let raw_name = &cmd.name;

        quote!{
            pub type #pfn_name = extern "system" fn(
                #( #params1 ),*
            ) -> #return_type;

            // #[link(name = "vulkan")]
            #[cfg_attr(target_os = "linux", link(name = "vulkan"))]
            #[cfg_attr(target_os = "windows", link(name = "vulkan-1"))]
            extern "system" {
                #[link_name = #raw_name]
                fn #name( #( #params2 ),* ) -> #return_type;
            }
        }
    });

    quote!{

        pub struct InstanceCommands {
            #( #instance_command_params ),*
        }
        impl InstanceCommands {
            fn new() -> Self {
                Self {
                    #( #instance_cmd_inits, )*
                }
            }
        }
        impl ::std::fmt::Debug for InstanceCommands {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "InstanceCommands")
            }
        }

        pub struct DeviceCommands {
            #( #device_command_params ),*
        }
        impl DeviceCommands {
            fn new() -> Self {
                Self {
                    #( #device_cmd_inits, )*
                }
            }
        }
        impl ::std::fmt::Debug for DeviceCommands {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "DeviceCommands")
            }
        }

        #( #non_static_command_definitions )*

        #( #static_command_definitions )*

    }

}

fn is_return_param(field: &&vkxml::Field, catagories: &HashMap<&str, FieldCatagory>) -> bool {
    // exception for *mut c_void
    let exception;
    if field.basetype.as_str() == "void" && matches!(field.reference, Some(ReferenceType::Pointer)) {
        // the exception does not take place if the *mut c_void has a corresponding mutable size param
        if let Some(size) = field.size.as_ref() {
            exception = catagories
                            .get(size.as_str())
                            .map(|cat| !matches!(cat, FieldCatagory::Return))
                            .unwrap_or(true);
        }
        else {
            exception = true;
        }
    }
    else {
        exception = false;
    }

    field.reference.is_some() && field.is_const == false && !exception
}

// this is for automatically generating methods which provide a more ideal rust interface for calling
// vulkan commands
fn make_owner_method(cmd: &Command) -> TokenStream {

    // skip vkCreateDevice since it needs special handling
    if &cmd.name == "vkCreateDevice" || &cmd.name == "vkCreateInstance" {
        return quote!();
    }

    let category_map = catagorize_fields(&cmd);
    if category_map.is_err() {
        // error generating category_map and automatic method create cannot be done
        eprintln!("error making category_map for command {}: {}", cmd.name, category_map.unwrap_err());
        return quote!();
    }

    let category_map = category_map.unwrap();

    let name = cmd.name.as_code();
    let owner_name = utils::make_handle_owner_name(cmd.param[0].basetype.as_str());

    let method_name_raw = case::camel_to_snake(cmd.name.as_str());
    let method_name = method_name_raw.as_code();

    let cmd_cat = command_category(cmd);
    let skip_first = if matches!(cmd_cat, CommandCategory::Entry) {
        0
    }
    else {
        1
    };

    let method_caller;
    match cmd.param[0].basetype.as_str() {
        "VkInstance" | "VkDevice" => {
            method_caller = quote!( self.commands.#name.call() );
        }
        _ => {
            match cmd_cat {
                CommandCategory::Entry => {
                    method_caller = quote!( #name::call() );
                }
                _ => {
                    method_caller = quote!( self.dispatch_parent.commands.#name.call() );
                }
            }
        }
    }

    // check method verb
    //
    // NOTE all vulkan commands have a verb as the first word of the command
    // the verb indicates what type of command it is, and can be used to help determine how best to
    // make a rust safe method to call the vulkan command safely
    let method_verb = method_name_raw.split('_').skip(1).next().expect("error: method name without verb");

    // for each method, the first parameter should be the dispatchable handle
    let first_inner_param = if matches!(cmd_cat, CommandCategory::Entry){
        None
    }
    else {
        Some( quote!( self.handle , ) )
    };

    match method_verb {
        "destroy" => {
            // NOTE we are making the allocator always None for now
            // thus, there is no support for using custom allocators for now

            // destroy methods are a special case where we implement the drop method for the
            // appropriate handle owner
            let owner_name;
            let method_params;
            let method_caller;
            match cmd.name.as_str() {
                "vkDestroyInstance" | "vkDestroyDevice" => {
                    // for instancce and device, we are destroying the dispatchalbe type
                    let type_to_destroy = utils::make_handle_owner_name(cmd.param[0].basetype.as_str());
                    owner_name = quote!( #type_to_destroy );
                    method_params = quote!( self.handle, None.to_c() );
                    method_caller = quote!( self.commands.#name.call() );
                }
                _ => {
                    // for everything else, the second parameter should be the type we are
                    // destroying
                    let type_to_destroy = utils::make_handle_owner_name(cmd.param[1].basetype.as_str());
                    owner_name = quote!( #type_to_destroy );
                    method_params = quote!( self.dispatch_parent.handle, self.handle, None.to_c() );
                    method_caller = quote!( self.dispatch_parent.commands.#name.call() );
                }
            }

            quote!{
                impl<Own> Drop for #owner_name<'_, Own> {
                    fn drop(&mut self) {
                        use ::std::any::TypeId;
                        if TypeId::of::<Own>() == TypeId::of::<Owned>() {
                            #method_caller(#method_params);
                        }
                    }
                }
            }
        }
        // "free" => {

        //   eprint!("{} (", cmd.name.as_str());
        //   for param in cmd.param.iter() {
        //       eprint!("{}, ", param.name.as_ref().unwrap().as_str());
        //   }
        //   eprintln!(") -> {:?}", cmd.return_type.basetype);

        //   //quote!()
        //     //dbg!(&cmd);
        //     //let category_map = catagorize_fields(&cmd);
        //     //for category in category_map.iter() {
        //     //    dbg!(category);
        //     //}
        //     quote!()
        // }
        //"queue" => {
        //    dbg!(cmd.name.as_str());
        //    quote!()
        //}
        //"reset" => {
        //    dbg!(cmd.name.as_str());
        //    quote!()
        //}
        //"enumerate" | "get" | "create" | "set" | "acquire" | "bind" => {
        _ => {
            //for category in category_map.iter() {
            //    dbg!(category);
            //}

            let lifetime_defs;
            let impl_lifetime;
            // let call_lifetime;
            let self_modifier;
            let mut with_lifetime;

            let mut fn_generics = ty::Generics::default();

            let safty = match cmd_cat {
                CommandCategory::Entry => None,
                _ => Some(quote!(unsafe)),
            };

            match method_verb {
                "create" | "allocate" | "enumerate" | "get" => {
                    lifetime_defs = quote!();
                    impl_lifetime = quote!('_);
                    // call_lifetime = quote!('handle);
                    fn_generics.push_lifetime_param("'handle");
                    self_modifier = quote!('handle);
                    with_lifetime = WithLifetime::Yes("'handle");
                }
                "cmd" => {
                    lifetime_defs = quote!('resource);
                    impl_lifetime = quote!('resource);
                    // call_lifetime = quote!();
                    self_modifier = quote!(mut);
                    with_lifetime = WithLifetime::Yes("'resource");
                }
                "free" => {
                    lifetime_defs = quote!();
                    impl_lifetime = quote!('_);
                    // call_lifetime = quote!('a);
                    fn_generics.push_lifetime_param("'a");
                    self_modifier = quote!();
                    with_lifetime = WithLifetime::Yes("'a");
                }
                _ => {
                    lifetime_defs = quote!();
                    impl_lifetime = quote!('_);
                    // call_lifetime = quote!();
                    self_modifier = quote!();
                    with_lifetime = WithLifetime::No
                }
            }

            if matches!(cmd_cat, CommandCategory::Entry) {
                with_lifetime = WithLifetime::Yes("'static");
            }

            let mut count = 0;
            let mut new_generic_name = || {
                let name = format!("C{}", count);
                count += 1;
                name
            };

            let mut generic_names = HashMap::new();

            // assuming there is only ever one return pn parameter, we create only one generic type to handle it
            for field in  cmd.param.iter() {
                if category_map.get(field_name(field)).unwrap().is_pn() {
                    let bt = field.basetype.as_code();
                    let generic_name = new_generic_name();
                    let generic_name_code = generic_name.as_code();
                    generic_names.insert(field_name(field), generic_name);
                    fn_generics.push_type_param(quote!(#generic_name_code: PnChain<#bt<'static, 'static>>));
                }
            }

            // determine if method should call the vulkan command twice in order to query a size
            // field category
            let can_query_size = category_map
                .values()
                .find(|cat| cat.is_size_mut())
                .is_some();

            let fields_outer = cmd.param.iter().skip(skip_first)
                .filter(|field| category_map.get(field_name(field)).unwrap().is_normal())
                .map(|field| {
                    Rtype::new(field, cmd.name.as_str())
                        .public_lifetime(with_lifetime)
                        .command_verb(method_verb)
                        .as_field()
                });

            // when a count/size variable affects multiple input arrays, set the size once
            // based on the first input array, and debug_assert that the other input arrays are the
            // same size
            // The HashMap is just incase a function has multiple different count/size variables
            let mut size_set = HashMap::new();
            let size_vars = cmd.param.iter()
                .filter_map(|field| {
                    let field_name_raw = field_name(field);
                    let field_name = case::camel_to_snake(field_name_raw).as_code();
                    match category_map.get(field_name_raw).unwrap() {
                        FieldCatagory::SizeMut => {
                            let basetype = field.basetype.as_code();
                            Some(quote!( let #field_name: &mut #basetype = &mut 0; ))
                        }
                        FieldCatagory::Size => { // there should be a following NormalSized parameter that will set the size value
                            let basetype = field.basetype.as_code();
                            size_set.insert(field_name_raw, false);
                            Some(quote!( let #field_name : #basetype; ))
                        }
                        FieldCatagory::NormalSized => {
                            let size_raw = field.size.as_ref().expect("error: NormalSized with no size").as_str();
                            let size = case::camel_to_snake(size_raw).as_code();
                            match field.array.as_ref().unwrap() {
                                vkxml::ArrayType::Static => None,
                                vkxml::ArrayType::Dynamic =>
                                {
                                    let is_size_set = size_set.get(size_raw).expect(format!("error: NormalSized with no size {}", cmd.name).as_str());
                                    if !is_size_set {
                                        size_set.entry(size_raw).and_modify(|x| *x=true);
                                        Some(quote!( #size = #field_name.len() as _; ))
                                    } else {
                                        let a_type = Rtype::new(field, cmd.name.as_str())
                                            .allow_optional(false);
                                        Some(quote!{
                                            let o: Option<#a_type> = (#field_name).into();
                                            o.map(|field|debug_assert!(field.len() == #size as _));
                                        })
                                    }
                                }
                            }
                        }
                        _ => None,
                    }
                });

            let size_query = if can_query_size {
                let fields_inner = cmd.param.iter().skip(skip_first)
                    .map( |field| {
                        let name_raw = field_name(&field);
                        let field_name = case::camel_to_snake(name_raw).as_code();
                        if category_map.get(name_raw).unwrap().is_return_sized() {
                            quote!( None.to_c() )
                        }
                        else {
                            quote!( #field_name.to_c() )
                        }
                    });
                if cmd.return_type.basetype == "VkResult" {
                    Some( quote!{
                        let vk_result = #method_caller( #first_inner_param #( #fields_inner ),* );
                        // for commands where we can querry size, I assume that the result can only
                        // be success (0) or an error (negitive number)
                        if vk_result.is_err() {
                            return vk_result.err(); // return the error code
                        }
                    })
                }
                else {
                    assert_eq!(cmd.return_type.basetype, "void");
                    Some( quote!{
                        #method_caller( #first_inner_param #( #fields_inner ),* );
                    })
                }
            }
            else {
                None
            };

            let return_vars = cmd.param.iter()
                .filter_map(|field| {
                    let f_name_raw = field_name(field);
                    let f_name = case::camel_to_snake(f_name_raw).as_code();
                    match category_map.get(f_name_raw).unwrap() {
                        FieldCatagory::Return => {
                            Some(quote!( let mut #f_name = MaybeUninit::uninit(); ))
                        }
                        FieldCatagory::ReturnSized => {
                            let size = field.size.as_ref().unwrap().replace("::", ".");
                            let size = case::camel_to_snake(size.as_str()).as_code();
                            Some(quote!( let mut #f_name = Vec::with_capacity(#size.value() as _); ))
                        }
                        FieldCatagory::ReturnPn => {
                            let bt = field.basetype.as_code();
                            let generic_name = generic_names.get(field_name(field)).unwrap().as_code();
                            Some(quote!(
                                let mut #f_name: PnTuple<#bt<'static, 'static>, #generic_name> = PnTuple::new();
                                #f_name.link_list();
                            ))
                        }
                        FieldCatagory::ReturnPnSized => {
                            let size = field.size.as_ref().unwrap().replace("::", ".");
                            let size = case::camel_to_snake(size.as_str()).as_code();
                            let bt = field.basetype.as_code();
                            let generic_name = generic_names.get(field_name(field)).unwrap().as_code();
                            Some(quote!(
                                let mut #f_name: Vec<_> = (0..#size.value()).into_iter().map(|_|#bt::init_s_type()).collect();
                                let mut pn_chains: Vec<_> = (0..#size.value()).into_iter().map(|_|#generic_name::new_chain()).collect();
                                for (head, chain) in #f_name.iter_mut().zip(pn_chains.iter_mut()) {
                                    chain.link_chain();
                                    head.add_chain(chain);
                                }
                            ))
                        }
                        _ => None,
                    }
                });

            let main_call = {
                let fields_inner = cmd.param
                    .iter()
                    .skip(skip_first)
                    .map( |field| {
                        let name_raw = field_name(&field);
                        let field_name = case::camel_to_snake(name_raw).as_code();
                        if category_map.get(name_raw).unwrap().is_return() {
                            quote!( (&mut #field_name).to_c() )
                        }
                        else {
                            quote!( #field_name.to_c() )
                        }
                    });
                if cmd.return_type.basetype == "VkResult" {
                    quote!{
                        let vk_result = #method_caller( #first_inner_param #( #fields_inner ),* );
                        if vk_result.is_err() {
                            return vk_result.err();
                        }
                    }
                }
                // if the return_type is not VkResult and is not void, then we assume that this
                // function only returns this return_type
                else if cmd.return_type.basetype != "void" {
                    quote!{
                        return #method_caller( #first_inner_param #( #fields_inner ),* );
                    }
                }
                else {
                    quote!{
                        #method_caller( #first_inner_param #( #fields_inner ),* );
                    }
                }
            };

            let prep_return_vars = cmd.param.iter()
                .filter_map(|field| {
                    let field_name_raw = field_name(field);
                    let field_name = case::camel_to_snake(field_name_raw).as_code();
                    match category_map.get(field_name_raw).unwrap() {
                        FieldCatagory::Return => {
                            Some(quote!( let #field_name = unsafe { #field_name.assume_init() }; ))
                        }
                        FieldCatagory::ReturnSized => {
                            let size = field.size.as_ref().unwrap().replace("::", ".");
                            let size = case::camel_to_snake(size.as_str()).as_code();
                            Some(quote!( unsafe { #field_name.set_len(#size.value() as _) }; ))
                        }
                        FieldCatagory::ReturnPnSized => {
                            Some(quote!(
                                let #field_name: Vec<_> = #field_name.drain(..).zip(pn_chains.drain(..))
                                    .map(|(head, chain)|PnTuple::from_parts(head, chain))
                                    .collect();
                            ))
                        }
                        _ => None,
                    }
                });

            let return_count = category_map
                .values()
                .filter(|cat|cat.is_return())
                .count();

            let return_code;
            let return_type;
            if return_count == 0 {
                if cmd.return_type.basetype == "VkResult" {
                    return_type = Some( quote!(VkResult<()>) );
                    return_code = Some( quote!(vk_result.success(())) )
                }
                else {
                    return_type = Some( quote!(()) );
                    return_code = None;
                }
            }
            else {
                let return_field_types = cmd.param.iter().filter_map(|field| {
                    let field_cat = category_map.get(field_name(field)).unwrap();
                    if field_cat.is_return() {
                        // Some(utils::r_return_type(field, with_lifetime).command_verb(method_verb))
                        Some(
                            pipe!{ ret = RreturnType::new(&field) =>
                                STAGE {
                                    ret.command_verb(method_verb)
                                }
                                WHEN field_cat.is_pn() => {
                                    ret.public_lifetime("'static")
                                        .private_lifetime("'static")
                                        .pn_tuple(generic_names.get(field_name(field)).unwrap())
                                }
                                WHEN !field_cat.is_pn() => {
                                    ret.public_lifetime(with_lifetime)
                                }
                            }
                        )
                    }
                    else {
                        None
                    }
                });
                return_type = Some(
                        quote!{
                            ( #( #return_field_types ),* )
                        }
                    );

                let return_vars = cmd.param.iter()
                    .filter_map(|field| {
                        let field_name_raw = field_name(field);
                        let field_name = case::camel_to_snake(field_name_raw).as_code();
                        if category_map.get(field_name_raw).unwrap().is_return() {
                            Some(quote!( #field_name ))
                        }
                        else {
                            None
                        }
                    });
                let return_tuple = if matches!(cmd_cat, CommandCategory::Entry) {
                    quote!{ (ret, &()) }
                }
                else {
                    quote!{ (ret, self) }
                };
                return_code = if cmd.return_type.basetype == "VkResult" {
                    Some( quote!{
                        let ret = ( #( #return_vars ),* ); //(A, B, ...);
                        vk_result.success(#return_tuple.ret())
                    } )
                }
                else {
                    Some( quote!{
                        let ret = ( #( #return_vars ),* ); //(A, B, ...);
                        #return_tuple.ret()
                    } )
                };
            }

            let result = if cmd.return_type.basetype == "VkResult" && return_count > 0 {
                Some( quote!(VkResult<#return_type>) )
            }
            // this branch should only ever be hit for commands which have non prameter return types
            // (i.e. the return type is not written to some user provided pointer)
            else if cmd.return_type.basetype != "void" && cmd.return_type.basetype != "VkResult" {
                let result = utils::r_return_type(&cmd.return_type, with_lifetime);
                Some( quote!(#result) )
            }
            else {
                return_type
            };

            let this = if matches!(cmd_cat, CommandCategory::Entry) {
                None
            }
            else {
                Some( quote!( & #self_modifier self, ) )
            };

            let method = quote!{
                pub #safty fn #method_name #fn_generics ( #this #( #fields_outer ),* ) -> #result {
                    #( #size_vars )*
                    #size_query
                    #( #return_vars )*
                    #main_call
                    #( #prep_return_vars )*
                    #return_code
                }
            };

            if matches!(command_category(cmd), CommandCategory::Entry) {
                method
            }
            else {
                quote!{
                    //#multi_return_type
                    impl<#lifetime_defs> #owner_name<#impl_lifetime> {
                        #method
                    }
                }
            }
        }
    }
}

type CategoryMap<'a> = HashMap<&'a str, FieldCatagory>;

#[derive(Debug)]
enum FieldCatagory {
    Normal,
    NormalSized,
    Return,
    ReturnSized,
    ReturnPn,
    ReturnPnSized,
    Size,
    SizeMut,
}

impl FieldCatagory {
    fn is_return(&self) -> bool {
        use FieldCatagory::*;
        match self {
            Return | ReturnPn | ReturnSized | ReturnPnSized => true,
            _ => false,
        }
    }
    fn is_return_sized(&self) -> bool {
        use FieldCatagory::*;
        match self {
            ReturnSized | ReturnPnSized => true,
            _ => false,
        }
    }
    fn is_normal(&self) -> bool {
        use FieldCatagory::*;
        match self {
            Normal | NormalSized => true,
            _ => false,
        }
    }
    fn is_pn(&self) -> bool {
        use FieldCatagory::*;
        match self {
            ReturnPn | ReturnPnSized => true,
            _ => false,
        }
    }
    // fn is_size(&self) -> bool {
    //     use FieldCatagory::*;
    //     match self {
    //         Size | SizeMut => true,
    //         _ => false,
    //     }
    // }
    fn is_size_mut(&self) -> bool {
        use FieldCatagory::*;
        match self {
            SizeMut => true,
            _ => false,
        }
    }
}

fn catagorize_fields(cmd: &Command) -> Result<CategoryMap, &'static str> {
    let mut catagories = HashMap::new();
    for field in cmd.param.iter() {

        let name = field.name.as_ref().unwrap().as_str();

        let cmd_return_type = cmd.return_type.basetype.as_str();

        let extendable = global_data::is_base(&field.basetype);

        // NOTE this has the following assumptions
        // 1) if a command includes mutable pointers, it is assumed that those parameters are for
        //    returning data, and the auto generated api doesn't require the user to provide such
        //    parameters
        // 2) an exception to the above is if a command returns something other than VkResult or void
        //    then we assume that any mutable inputs must still be provided by the user and are not
        //    just for returning data (e.g. vkGetPhysicalDeviceWaylandPresentationSupportKHR still
        //    requires the user to provide a &mut to the 'display' parameter)
        // 3) another exception we will make is for functions that take a *mut c_void. These will
        //    require the user to provide a buffer of the appropriate size manually.
        if is_return_param(&field, &catagories) && ( cmd_return_type == "VkResult" || cmd_return_type == "void" ) {
            if field.size.is_some() && extendable {
                catagories.insert(name, FieldCatagory::ReturnPnSized);
            }
            else if extendable {
                catagories.insert(name, FieldCatagory::ReturnPn);
            }
            else if field.size.is_some() {
                catagories.insert(name, FieldCatagory::ReturnSized);
            }
            else {
                catagories.insert(name, FieldCatagory::Return);
            }
        }
        else {
            if field.size.is_some() {
                catagories.insert(name, FieldCatagory::NormalSized);
            }
            else {
                catagories.insert(name, FieldCatagory::Normal);
            }
        }

        // if a param has a size, then we should find the corresponding field already in the map
        // NOTE this is because we assume that all size fields come before the field which is sized
        // in vulkan
        if let Some(size) = field.size.as_ref() {
            let size_name = size.as_str();
            if let Some(category) = catagories.get_mut(size_name) {
                //.expect(format!("error: size field is not already in hashmap {}", size_name).as_str());
                // change an existing field category to Size
                match category {
                    FieldCatagory::Normal => *category = FieldCatagory::Size,
                    FieldCatagory::Return => *category = FieldCatagory::SizeMut,
                    _ => {}, // no need to update if already updated, and size can,t be NormalSized
                }
            }
            else {

                // do nothing for this case, the size parameter should be available as a static size
                // or a member in an info stuct
                eprintln!("special sized field: {}", size_name);
                //return Err("method has dynamically sized return, but no size field");
            }
        }

    }
    Ok(catagories)
}

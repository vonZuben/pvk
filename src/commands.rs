
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;

use std::collections::HashMap;

pub fn make_pfn_name(cmd_name: &str) -> TokenStream {
    format!("PFN_{}", cmd_name).as_code()
}

pub fn make_pfn_loader_name(cmd_name: &str) -> TokenStream {
    format!("PFN_Loader_{}", cmd_name).as_code()
}

pub fn make_macro_name_instance(cmd_name: &str) -> TokenStream {
    format!("get_instance_cmd_{}", cmd_name).as_code()
}

pub fn make_macro_name_device(cmd_name: &str) -> TokenStream {
    format!("get_device_cmd_{}", cmd_name).as_code()
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

#[derive(Debug)]
pub enum CommandCategory {
    Device,
    Instance,
    Static,
}

pub fn command_category(cmd: &Command) -> CommandCategory {
    match cmd.name.as_str() {
        "vkGetInstanceProcAddr" | "vkGetDeviceProcAddr" => CommandCategory::Static,
        _ =>
            match cmd.param[0].basetype.as_str() {
                "VkDevice" | "VkCommandBuffer" | "VkQueue" => CommandCategory::Device,
                "VkInstance" | "VkPhysicalDevice" => CommandCategory::Instance,
                _ => CommandCategory::Static,
            }
    }
}

fn field_name(field: &Field) -> &str {
    field.name.as_ref()
        .expect("error: field with no name").as_str()
}

pub fn handle_commands<'a>(commands: &'a Commands, parse_state: &mut crate::ParseState<'a>) -> TokenStream {

    macro_rules! filter_varients {
        ( $( $varient:tt )* ) => {
            |cmd| {
                match command_category(&cmd) {
                    $( $varient )* => true,
                    _ => false,
                }
            }
        }
    };

    for cmd in commands.elements.iter() {
        parse_state.command_type_cache.insert(cmd.name.as_str(), command_category(&cmd));
    }

    let instance_commands = commands.elements.iter().filter(filter_varients!(CommandCategory::Instance));
    let device_commands = commands.elements.iter().filter(filter_varients!(CommandCategory::Device));
    let instance_and_device_commands = commands.elements.iter().filter(
        filter_varients!(CommandCategory::Instance | CommandCategory::Device));
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
    let non_static_command_definitions = instance_and_device_commands.map(|cmd| {
        let name = cmd.name.as_code();
        let pfn_name = make_pfn_name(cmd.name.as_str());
        let pfn_loader_name = make_pfn_loader_name(cmd.name.as_str());
        let raw_name = &cmd.name;

        let return_type = make_field_type(&cmd.return_type);
        let params1 = cmd.param.iter().map(handle_field);
        let params2 = params1.clone(); // because params is needed twice and quote will consume params1

        // create manager methods
        let manager_name = crate::definitions
            ::make_manager_name(cmd.param[0].basetype.as_str());

        let manager_method = {


            //for p in cmd.param.iter() {
                //eprintln!("const: {}; size: {:?}, ref_type: {:?}; array_type: {:?}",
                //          p.is_const,
                //          p.size.as_ref().map(|_|""),
                //          p.reference,
                //          p.array);
                //eprintln!("name: {}; type: {}; const: {}; size: {:?}, ref_type: {:?}; array_type: {:?}",
                //          p.name.as_ref().unwrap(),
                //          p.basetype,
                //          p.is_const,
                //          p.size.as_ref().map(|_|""),
                //          p.reference,
                //          p.array);
            //}


            // keep track of params that are counts for array params
            let mut count_cache = HashMap::new();
            for p in cmd.param.iter() {
                // also assert that these size members are not used
                assert!(p.c_size.is_none());
                assert!(p.size_enumref.is_none());
                if p.size.is_some() {
                    count_cache.insert(p.size.as_ref().unwrap().as_str(),
                                       field_name(&p));
                }
            }

            // -----------------------------------
            // generate user facing parameters for methods

                dbg!(cmd.name.as_str());

            let outer_params = cmd.param[1..].iter()
                .filter(|field| count_cache.get(field_name(&field)).is_none())
                .map(|field| {

                let field_name_raw = field_name(&field);
                let field_name = field_name_raw.as_code();

                let ref_type;
                let basetype;

                if field.optional.is_some() {
                    eprintln!("field: {}; opsional: {}", field.name.as_ref().unwrap().as_str(), field.optional.as_ref().unwrap());
                }

                match field.reference {
                    Some(ReferenceType::Pointer) => {
                        if field.is_const {
                            ref_type = quote!( & );
                        }
                        else {
                            ref_type = quote!( &mut );
                        }
                        // NOTE assumption: if reference is pointer, then array is dynamic or
                        // none
                        match field.array {
                            Some(ArrayType::Dynamic) => {
                                basetype = match field.basetype.as_str() {
                                        "char" => quote!( ::std::ffi::CStr ),
                                        "void" => quote!( [u8] ),
                                        _ => {
                                            let bt = field.basetype.as_code();
                                            quote!( [#bt] )
                                        }
                                };
                            }
                            None => {
                                basetype = match field.basetype.as_str() {
                                        "char" => quote!( ::std::ffi:CStr ),
                                        "void" => {
                                            eprintln!("{}", format!("error: unexpected pointer to void: {} -> {}",
                                                                 cmd.name.as_str(), field_name_raw));
                                            quote!( c_void )
                                        }
                                        _ => {
                                            let bt = field.basetype.as_code();
                                            quote!( #bt )
                                        }
                                };
                            }
                            _ => panic!(format!("error: only expecting dynamic array or not array: {} -> {}",
                                                cmd.name.as_str(), field_name_raw)),
                        }
                    }
                    Some(ReferenceType::PointerToPointer) => {
                        // ref_type can only be mut in this case
                        ref_type = quote!( &mut &mut );
                        basetype = match field.basetype.as_str() {
                            "char" => panic!(format!("error: unexpected pointer pointer to char: {} -> {}",
                                                     cmd.name.as_str(), field_name_raw)),
                            "void" => {
                                eprintln!("{}", format!("error: unexpected pointer pointer to void: {} -> {}",
                                                     cmd.name.as_str(), field_name_raw));
                                quote!( c_void )
                            }
                            _ => {
                                let bt = field.basetype.as_code();
                                quote!( #bt )
                            }
                        };
                    }
                    Some(ReferenceType::PointerToConstPointer) => {
                        match field.array {
                            Some(ArrayType::Dynamic) => {
                                if field.is_const {
                                    ref_type = quote!( && );
                                } else {
                                    ref_type = quote!( &mut& );
                                }
                                let bt = field.basetype.as_code();
                                basetype = quote!( #bt );
                            }
                            _ => panic!(format!("error: only expecting dynamic array type: {} -> {}",
                                                cmd.name.as_str(), field_name_raw)),
                        }
                    }
                    None => { // not a pointer
                        // could still be a static array
                        match field.array {
                            Some(ArrayType::Static) => {
                                ref_type = quote!();
                                let bt = field.basetype.as_code();
                                let size = field.size.as_ref().expect("error: static array without size").as_code();
                                basetype = quote!( [#bt;#size] );
                            }
                            None => {
                                ref_type = quote!();
                                let bt = field.basetype.as_code();
                                basetype = quote!( #bt );
                            }
                            _ => panic!(format!("error: unexpected array type for non-pointer field: {} -> {}",
                                                cmd.name.as_str(), field_name_raw)),
                        }
                    }
                }

                quote!( #field_name : #ref_type #basetype )
            });

            // -----------------------------------
            // determine how to pass rust method params to raw vulkan function

            // All non-static commands will have a first param which match a manager name which has
            // a handle of the corresponding type.
            // Put the handle as the first parameter.
            let first_inner_param = Some( quote!( self.handle ) );

            let other_inner_params = cmd.param[1..].iter().map(|field| {
                let field_name_raw = field_name(&field);

                let field_name = field_name_raw.as_code();

                let ptr_mut = match field.is_const {
                    true => quote!( as_ptr() ),
                    false => quote!( as_mut_ptr() ),
                };

                if let Some(array_param_raw) = count_cache.get(&field_name_raw) {
                    let array_param = array_param_raw.as_code();
                    quote!( #array_param.len() as _ )
                }
                else {

                    match field.reference {
                        Some(ReferenceType::Pointer) => {
                            // NOTE assumption: if reference is pointer, then array is dynamic or
                            // none
                            match field.array {
                                Some(ArrayType::Dynamic) => {
                                    quote!( #field_name.#ptr_mut as _ )
                                }
                                None => {
                                    quote!( #field_name )
                                }
                                _ => panic!(format!("error: only expecting dynamic array or not array: {} -> {}",
                                                    cmd.name.as_str(), field_name_raw)),
                            }
                        }
                        Some(ReferenceType::PointerToPointer) => {
                            quote!( #field_name as *mut &mut _ as *mut *mut _ )
                        }
                        Some(ReferenceType::PointerToConstPointer) => {
                            match field.array {
                                Some(ArrayType::Dynamic) => quote!( #field_name as *const & _ as *const *const _ ),
                                _ => panic!(format!("error: only expecting dynamic array type: {} -> {}",
                                                    cmd.name.as_str(), field_name_raw)),
                            }
                        }
                        None => { // not a pointer
                            // could still be a static array
                            match field.array {
                                Some(ArrayType::Static) => {
                                    quote!( #field_name )
                                }
                                None => {
                                    quote!( #field_name )
                                }
                                _ => panic!(format!("error: unexpected array type for non-pointer field: {} -> {}",
                                                    cmd.name.as_str(), field_name_raw)),
                            }
                        }
                    }
                }
            });

            let method_name = case::camel_to_snake(cmd.name.as_str()).as_code();

            let method_caller = match cmd.param[0].basetype.as_str() {
                "VkInstance" | "VkDevice" => quote!( self.commands.#name.0 ),
                _ => {
                    quote!( self.parent.commands.#name.0 )
                }
            };

            quote!{
                impl<'a> #manager_name<'a> {
                    fn #method_name(&self, #( #outer_params ),* ) {
                        #method_caller( #first_inner_param, #( #other_inner_params ),* );
                    }
                }
            }

        };

        // this is for generating the code to load funtion pointers based on the feature
        // these generate the code for only the respective device/instance commands
        //
        // they are called from the feature generated code to load a funtion pointer
        // for a command container's member
        //
        // e.g. InstanceCommand ($cmd_container) will have a member (#name)
        // when called from the feature loading code
        let instance_macro_name = make_macro_name_instance(&cmd.name);
        let is_instance_cmd = match command_category(&cmd) {
            CommandCategory::Instance => quote!( $cmd_container.#name.load(
                    |raw_cmd_name| {
                        unsafe { GetInstanceProcAddr(*$instance, raw_cmd_name.as_ptr()) }
                    })
                ),
            _ => quote!(),
        };
        let device_macro_name = make_macro_name_device(&cmd.name);
        let is_device_cmd = match command_category(&cmd) {
            CommandCategory::Device => quote!( $cmd_container.#name.load(
                    |raw_cmd_name| {
                        unsafe { GetDeviceProcAddr(*$device, raw_cmd_name.as_ptr()) }
                    })
                ),
            _ => quote!(),
        };

        quote!{
            #[allow(non_camel_case_types)]
            pub type #pfn_name = extern "system" fn(
                #( #params1 ),*
            ) -> #return_type;

            struct #pfn_loader_name (#pfn_name);
            impl #pfn_loader_name {
                fn new() -> Self {
                    extern "system" fn default_function ( #( #params2 ),* ) -> #return_type {
                        panic!(concat!(#raw_name, " is not loaded. Make sure the correct feature/extension is enabled"))
                    }
                    Self(default_function)
                }
                fn load<F>(&mut self, mut f: F) where F: FnMut(&::std::ffi::CStr) -> PFN_vkVoidFunction {
                    let cname = ::std::ffi::CString::new(#raw_name).unwrap();
                    let function_pointer = unsafe { ::std::mem::transmute::<_, *const ::std::ffi::c_void>( f(&cname) ) };
                    if function_pointer.is_null(){
                        panic!(concat!("error: couldn't load ", #raw_name));
                    }
                    else{
                        self.0 = unsafe { ::std::mem::transmute(function_pointer) };
                    }
                }
            }
            // add manager method
            //impl manager_name {
            //    //fn #method_name( #( #method_params ),* ) #
            //}
            #manager_method

            // define macro that can be used by the feature loader
            //
            // the feature loader only know what commands to load
            // but not what commands a device and instance commands
            //
            // these macros can be called on every command, but will only
            // actually produce code for the correct situation (instance/device)
            macro_rules! #instance_macro_name{
                ( $instance:ident, $cmd_container:ident ) => { #is_instance_cmd }
            }
            macro_rules! #device_macro_name {
                ( $device:ident, $cmd_container:ident ) => { #is_device_cmd }
            }
        }
    });

    let static_command_definitions = static_commands.map(|cmd| {
        let name = cmd.name.as_code();
        let return_type = make_field_type(&cmd.return_type);
        let params1 = cmd.param.iter().map(handle_field);
        let params2 = params1.clone();

        let pfn_name = make_pfn_name(cmd.name.as_str());

        let raw_name = &cmd.name;

        let instance_macro_name = make_macro_name_instance(&cmd.name);
        let device_macro_name = make_macro_name_device(&cmd.name);

        quote!{
            pub type #pfn_name = extern "system" fn(
                #( #params1 ),*
            ) -> #return_type;

            macro_rules! #instance_macro_name {
                ( $instance:ident, $cmd_container:ident ) => { }
            }
            macro_rules! #device_macro_name {
                ( $device:ident, $cmd_container:ident ) => { }
            }

            #[link(name = "vulkan")]
            extern "system" {
                #[link_name = #raw_name]
                fn #name ( #( #params2 ),* ) -> #return_type;
            }
        }
    });

    quote!{

        enum CommandCategory {
            Device,
            Instance,
            Static,
        }

        struct InstanceCommands {
            #( #instance_command_params ),*
        }
        impl InstanceCommands {
            fn new() -> Self {
                Self {
                    #( #instance_cmd_inits, )*
                }
            }
        }

        struct DeviceCommands {
            #( #device_command_params ),*
        }
        impl DeviceCommands {
            fn new() -> Self {
                Self {
                    #( #device_cmd_inits, )*
                }
            }
        }

        #( #non_static_command_definitions )*

        #( #static_command_definitions )*

    }

}

//fn get_command_parts(cmd: &Command) -> VkCommandParts {
//
//    let name = &cmd.name;
//
//    let prefix = match name {
//       x if x.starts_with("VkQueue") =>
//
//    (Some("efe"), Some("efef"), Some("fef"))
//
//}


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

fn field_first_indirection_optional(field: &Field) -> bool {
    if field.basetype.as_str() == "char" {
        return false;
    }
    field.optional.as_ref().map(|option_type| {
        // vkxml optional is a comma separated list representing if each level of
        // indirection is optional
        //
        // we only care about the first level of indirection
        let first_indirection_optional = option_type.split(',')
            .next().expect(format!("error: empty optional? -> {:?}", field.name).as_str());

        match first_indirection_optional {
            "true" => true,
            "false" => false,
            _ => panic!(format!("error: optional type not true or false -> {:?}", field.name)),
        }
    }).unwrap_or(false)
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

        let manager_method = make_manager_method(&cmd);

        let manager_method_old = {

            //let _ = make_manager_method(&cmd);

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
            let mut count_cache: HashMap<&str, &str> = HashMap::new();
            // NOTE commenting out this so that all parameters a generated normally
            //for p in cmd.param.iter() {
            //    // also assert that these size members are not used
            //    assert!(p.c_size.is_none());
            //    assert!(p.size_enumref.is_none());
            //    if p.size.is_some() {
            //        count_cache.insert(p.size.as_ref().unwrap().as_str(),
            //                           field_name(&p));
            //    }
            //}

            // -----------------------------------
            // generate user facing parameters for methods

                //dbg!(cmd.name.as_str());

            let outer_params = cmd.param[1..].iter()
                .filter(|field| count_cache.get(field_name(&field)).is_none())
                .map(|field| {

                let field_name_raw = field_name(&field);
                let field_name = field_name_raw.as_code();

                let ref_type;
                let basetype;

                //if field.optional.is_some() && field.reference.is_none() {
                //    eprintln!("field: {}; opsional: {}", field.name.as_ref().unwrap().as_str(), field.optional.as_ref().unwrap());
                //}

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
                                            //eprintln!("{}", format!("error: unexpected pointer to void: {} -> {}",
                                            //                     cmd.name.as_str(), field_name_raw));
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
                                //eprintln!("{}", format!("error: unexpected pointer pointer to void: {} -> {}",
                                //                     cmd.name.as_str(), field_name_raw));
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

                let optional_maybe = match field_first_indirection_optional(&field) && field.reference.is_some()
                    && field.array.is_none() {
                        true => quote!( Option<#ref_type #basetype> ),
                        false => quote!( #ref_type #basetype ),
                };

                quote!( #field_name : #optional_maybe )
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
                                    if field_first_indirection_optional(&field) {
                                        quote!( #field_name.#ptr_mut as _ )
                                    }
                                    else {
                                        quote!( #field_name as _ )
                                    }
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
                    pub fn #method_name(&self, #( #outer_params ),* ) {
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

fn is_return_param(field: &&Field) -> bool {
    field.reference.is_some() && field.is_const == false
}
fn not_return_param(field: &&Field) -> bool {
    !is_return_param(field)
}

// this is for automatically generating methods which provide a more ideal rust interface for calling
// vulkan commands
fn make_manager_method(cmd: &Command) -> TokenStream {

    let category_map = catagorize_fields(&cmd);
    if category_map.is_err() {
        // error generating category_map and automatic method create cannot be done
        eprintln!("error making category_map for command {}: {}", cmd.name, category_map.unwrap_err());
        return quote!();
    }

    let category_map = category_map.unwrap();

    // NOTE if method has a void return parameter, then we will not generate a method for it
    for field in cmd.param.iter() {
        let field_name = field_name(field);
        match category_map.get(field_name).unwrap() {
            FieldCatagory::ReturnSized | FieldCatagory::Return => {
                if field.basetype.as_str() == "void" {
                    eprintln!("cmd {} with void return type: {}", cmd.name, field_name);
                    return quote!();
                }
            }
            _ => {},
        }
    }

    let name = cmd.name.as_code();
    let manager_name = crate::definitions
        ::make_manager_name(cmd.param[0].basetype.as_str());

    let method_name_raw = case::camel_to_snake(cmd.name.as_str());
    let method_name = method_name_raw.as_code();

    let method_caller = match cmd.param[0].basetype.as_str() {
        "VkInstance" | "VkDevice" => quote!( self.commands.#name.0 ),
        _ => {
            quote!( self.parent.commands.#name.0 )
        }
    };

    // check method verb
    //
    // NOTE all vulkan commands have a verb as the first word of the command
    // the verb indicates what type of command it is, and can be used to help determine how best to
    // make a rust safe method to call the vulkan command safely
    let method_verb = method_name_raw.split('_').skip(1).next().expect("error: method name without verb");

    // for each method, the first parameter should be the dispatchable handle
    let first_inner_param = Some( quote!( self.handle ) );

    match method_verb {
        "create" => {
            //eprint!("{} (", cmd.name.as_str());
            //for param in cmd.param.iter() {
            //    eprint!("{}, ", param.name.as_ref().unwrap().as_str());
            //}
            //eprintln!(") -> {:?}", cmd.return_type.basetype);
            quote!()
        }
        //"get" => {
        //    //dbg!(&cmd);
        //    //let category_map = catagorize_fields(&cmd);
        //    //for category in category_map.iter() {
        //    //    dbg!(category);
        //    //}
        //    quote!()
        //}
        "enumerate" | "get" => {
            //for category in category_map.iter() {
            //    dbg!(category);
            //}

            // determine if method should call the vulkan command twice in order to query a size
            // field category
            let can_query_size = category_map
                .values()
                .find(|category| match category { FieldCatagory::SizeMut => true, _ => false } )
                .is_some();

            // -------------Can query size------------------
            if can_query_size {

                // fn locals for return fields
                let locals = cmd.param.iter()
                    .filter(is_return_param)
                    .map(|field| {
                        let field_name_raw = field_name(field);
                        let field_name = field_name_raw.as_code();
                        match category_map.get(field_name_raw).unwrap() {
                            FieldCatagory::ReturnSized => {
                                quote!( let mut #field_name = Vec::new(); )
                            }
                            FieldCatagory::Return => {
                                quote!( let mut #field_name = unsafe { ::std::mem::MaybeUninit::uninit() }; )
                            }
                            FieldCatagory::SizeMut => {
                                let basetype = field.basetype.as_code();
                                quote!( let #field_name: &mut #basetype = &mut 0; )
                            }
                            _ => unreachable!(),
                        }
                    });

                let update_locals1 = cmd.param.iter()
                    .filter(is_return_param)
                    .map(|field| {
                        let field_name_raw = field_name(field);
                        let field_name = field_name_raw.as_code();
                        match category_map.get(field_name_raw).unwrap() {
                            FieldCatagory::ReturnSized => {
                                let size = field.size.as_ref().unwrap().as_code();
                                Some ( quote!( #field_name.reserve(*#size as usize); ) )
                            }
                            _ => None,
                        }
                    })
                    .filter(Option::is_some);

                let update_locals2 = cmd.param.iter()
                    .filter(is_return_param)
                    .map(|field| {
                        let field_name_raw = field_name(field);
                        let field_name = field_name_raw.as_code();
                        match category_map.get(field_name_raw).unwrap() {
                            FieldCatagory::ReturnSized => {
                                let size = field.size.as_ref().unwrap().as_code();
                                Some ( quote!( unsafe { #field_name.set_len(*#size as usize) }; ) )
                            }
                            _ => None,
                        }
                    })
                    .filter(Option::is_some);

                let ret_count = category_map.iter()
                    .filter(|(_field_name, category)|
                            match category { FieldCatagory::Return | FieldCatagory::ReturnSized => true, _ => false })
                    .count();

                // if there is only one return field, then just return it directly
                // else, we create a struct with named parameters that correspond to each return
                // field
                let return_code;
                let return_type;
                if ret_count == 1 {
                    let ret_field = cmd.param.iter()
                        .find(|field| {
                            //let field_name_raw = field_name(&field);
                            let category = category_map.get(field_name(&field)).unwrap();
                            match category {
                                FieldCatagory::Return | FieldCatagory::ReturnSized => true,
                                _ => false,
                            }
                        })
                        .unwrap();
                    //let (field_name, _category) = category_map.iter()
                    //    .find(|(_field_name, category)| match category { FieldCatagory::Return => true, _ => false })
                    //    .unwrap(); // already know that there is one

                        // NOTE this only handles Vec returns,
                    let field_name = field_name(ret_field).as_code();
                    return_code = quote!( #field_name );

                    return_type = make_return_type(&ret_field);
                }
                else {
                    assert!(ret_count > 1);
                    return_code = quote!();
                    return_type = quote!();
                }

                let fields_outer = cmd.param[1..].iter()
                    .filter(|field| match category_map.get(field_name(field)).unwrap() {
                        FieldCatagory::Normal | FieldCatagory::NormalSized => true,
                        _ => false,
                    })
                    .map(make_outer_param);

                let fields_inner1 = cmd.param[1..].iter()
                    .map( |field| {
                        let name_raw = field_name(&field);
                        let _field_name = name_raw.as_code();
                        match category_map.get(name_raw).unwrap() {
                            FieldCatagory::ReturnSized => quote!( ::std::ptr::null_mut() ),
                            _ => make_inner_param(field),
                        }
                    });
                let fields_inner2 = cmd.param[1..].iter()
                    .map( |field| {
                        let name_raw = field_name(&field);
                        let field_name = name_raw.as_code();
                        match category_map.get(name_raw).unwrap() {
                            FieldCatagory::ReturnSized => quote!( #field_name.as_mut_ptr() ),
                            _ => make_inner_param(field),
                        }
                    });

                quote!{
                    impl<'a> #manager_name<'a> {
                        pub fn #method_name(&self, #( #fields_outer )* ) -> #return_type {
                            #( #locals )*
                            #method_caller( #first_inner_param, #( #fields_inner1 ),* );
                            #( #update_locals1 )*
                            #method_caller( #first_inner_param, #( #fields_inner2 ),* );
                            #( #update_locals2 )*
                            #return_code
                        }
                    }
                }
            }
            // -------------Cannot query size------------------
            else {
                // fn locals for return fields
                let locals = cmd.param.iter()
                    //.filter(is_return_param)
                    .map(|field| {
                        let field_name_raw = field_name(field);
                        let field_name = field_name_raw.as_code();
                        match category_map.get(field_name_raw).unwrap() {
                            FieldCatagory::ReturnSized => {
                                match field.reference.as_ref().expect("error: return type is not pointer 1") {
                                    ReferenceType::Pointer => {
                                        let size = field.size.as_ref().expect("error: ReturnSized with no size");
                                        let size = size.replace("::", ".").as_code();
                                        // NOTE HERE -- fix this!!!!! is this ok now??
                                        quote!( let mut #field_name = Vec::with_capacity(#size as usize); )
                                        //quote!( let #size = #field_name.len(); )
                                    }
                                    _ => {
                                        dbg!(cmd.name.as_str());
                                        dbg!(field_name_raw);
                                        panic!("error: unhandled pointer type for ReturnSized");
                                    }
                                }
                            }
                            FieldCatagory::Return => {
                                match field.reference.as_ref().expect("error: return type not pointer 2") {
                                    ReferenceType::Pointer => {
                                        //quote!( let mut #field_name = Vec::with_capacity(#size as usize); )
                                        let basetype = field.basetype.as_code();
                                        quote!( let mut #field_name = ::std::mem::MaybeUninit::<#basetype>::uninit(); )
                                    }
                                    ReferenceType::PointerToPointer => {
                                        let basetype = field.basetype.as_code();
                                        quote!( let mut #field_name = ::std::mem::MaybeUninit::<*mut #basetype>::uninit(); )
                                    }
                                    _ => {
                                        dbg!(cmd.name.as_str());
                                        dbg!(field_name_raw);
                                        panic!("error: unhandled pointer type for Return");
                                    }
                                }
                            }
                            FieldCatagory::NormalSized => {
                                let size = field.size.as_ref().expect("error: ReturnSized with no size").as_code();
                                quote!( let #size = #field_name.len() as _; )
                            }
                            _ => quote!(),
                        }
                    });

                let update_locals = cmd.param.iter()
                    .filter(is_return_param)
                    .map(|field| {
                        let field_name_raw = field_name(field);
                        let field_name = field_name_raw.as_code();
                        match category_map.get(field_name_raw).unwrap() {
                            FieldCatagory::ReturnSized => {
                                //let size = field.size.as_ref().expect("error: ReturnSized with no size 2").as_code();
                                let size = field.size.as_ref().expect("error: ReturnSized with no size");
                                let size = size.replace("::", ".").as_code();
                                Some ( quote!( unsafe { #field_name.set_len(#size as usize) }; ) )
                            }
                            FieldCatagory::Return => {
                                //let basetype = field.basetype.as_code();
                                Some( quote!( let #field_name = unsafe { #field_name.assume_init() }; ) )
                            }
                            _ => None,
                        }
                    })
                    .filter(Option::is_some);

                let ret_count = category_map.iter()
                    .filter(|(_field_name, category)|
                            match category { FieldCatagory::Return | FieldCatagory::ReturnSized => true, _ => false })
                    .count();

                // if there is only one return field, then just return it directly
                // else, we create a struct with named parameters that correspond to each return
                // field
                let return_code;
                let return_type;
                let custom_return_type;
                if ret_count == 1 {
                    let ret_field = cmd.param.iter()
                        .find(|field| {
                            //dbg!(cmd.name.as_str());
                            let field_name_raw = field_name(&field);
                            let category = category_map.get(field_name_raw).expect("error: field not in category map");
                            match category {
                                FieldCatagory::Return | FieldCatagory::ReturnSized => true,
                                _ => false,
                            }
                        })
                        .expect("error: can't find ret field");

                    let field_name = field_name(ret_field).as_code();
                    return_code = quote!( #field_name );

                    //match ret_field.reference {
                    //    Some(ReferenceType::PointerToPointer) => {
                    //        dbg!(ret_field.name.as_ref().unwrap().as_str());
                    //        dbg!(cmd.name.as_str());
                    //    }
                    //    _ => {}
                    //}
                    return_type = make_return_type(&ret_field);

                    custom_return_type = None;
                }
                else if ret_count == 0 {
                    eprintln!("0 returns???: {}", cmd.name);
                    return_code = quote!();
                    return_type = quote!(());
                    custom_return_type = None;
                }
                else {
                    assert!(ret_count > 1);
                    let custom_return_type_name = format!("{}Ret", cmd.name.as_str()).as_code();

                    let return_fields_iter = cmd.param.iter().filter(|field| {
                        match category_map.get(field_name(field)).unwrap() {
                            FieldCatagory::Return | FieldCatagory::ReturnSized => true,
                            _ => false,
                        }
                    });

                    return_type = quote!( #custom_return_type_name );

                    let return_fields = return_fields_iter.clone()
                        .map(|field| {
                            let name = field_name(field).as_code();
                            quote!( #name )
                        });
                    return_code = quote!{
                        #custom_return_type_name {
                            #( #return_fields, )*
                        }
                    };

                    let custom_return_type_fields = return_fields_iter.clone()
                        .map(|field| {
                            let name = field_name(field).as_code();
                            let basetype = make_return_type(field);
                            quote!( #name: #basetype )
                        });
                    custom_return_type = Some(quote!{
                        pub struct #custom_return_type_name {
                            #( #custom_return_type_fields, )*
                        }
                    });
                }

                let fields_outer = cmd.param[1..].iter()
                    .filter(|field|
                            match category_map.get(field_name(field)).expect("error: field not categotrized") {
                                FieldCatagory::Normal | FieldCatagory::NormalSized => true,
                                _ => false,
                    })
                    .map(make_outer_param);

                let fields_inner = cmd.param[1..].iter()
                    .map(|field| {
                        match category_map.get(field_name(field)).expect("error: field not categotrized") {
                            FieldCatagory::Return => {
                                let field_name = field_name(field).as_code();
                                quote!( #field_name.as_mut_ptr() )
                            }
                            _ => make_inner_param(field),
                        }
                    });

                quote!{
                    #custom_return_type
                    impl<'a> #manager_name<'a> {
                        pub fn #method_name(&self, #( #fields_outer )* ) -> #return_type {
                            #( #locals )*
                            #method_caller( #first_inner_param, #( #fields_inner ),* );
                            #( #update_locals )*
                            #return_code
                        }
                    }
                }
            }
        }
        _ => {
            quote!()
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
    Size,
    SizeMut,
}

fn catagorize_fields(cmd: &Command) -> Result<CategoryMap, &'static str> {
    let mut catagories = HashMap::new();
    for field in cmd.param.iter() {

        let name = field.name.as_ref().unwrap().as_str();

        if is_return_param(&field) {
            if field.size.is_some() {
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

fn make_return_type(field: &Field) -> TokenStream {
    match field.reference {
        Some(ReferenceType::Pointer) => {
            let basetype = field.basetype.as_code();
            if field.size.is_some() {
                quote!( Vec<#basetype> )
            }
            else {
                quote!( #basetype )
            }
        }
        Some(ReferenceType::PointerToPointer) => {
            let basetype = field.basetype.as_code();
            quote!( *mut #basetype )
        }
        Some(ReferenceType::PointerToConstPointer) => {
            panic!("error: PointerToConstPointer in return type")
        }
        None => {
            let basetype = field.basetype.as_code();
            quote!( #basetype )
        }
    }
}

fn make_outer_param(field: &Field) -> TokenStream {

    let field_name_raw = field_name(&field);
    let field_name = field_name_raw.as_code();

    let ref_type;
    let basetype;

    //if field.optional.is_some() && field.reference.is_none() {
    //    eprintln!("field: {}; opsional: {}", field.name.as_ref().unwrap().as_str(), field.optional.as_ref().unwrap());
    //}

    //if let Some(replace) = replace.get(field.name.as_ref().expect("error: outer fields should have a name").as_str()) {
    //    return quote!( #replace );
    //}

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
                            //eprintln!("{}", format!("error: unexpected pointer to void: {} -> {}",
                            //                     cmd.name.as_str(), field_name_raw));
                            quote!( c_void )
                        }
                        _ => {
                            let bt = field.basetype.as_code();
                            quote!( #bt )
                        }
                    };
                }
                _ => panic!(format!("error: only expecting dynamic array or not array: {}",
                                    field_name_raw)),
            }
        }
        Some(ReferenceType::PointerToPointer) => {
            // ref_type can only be mut in this case
            ref_type = quote!( &mut &mut );
            basetype = match field.basetype.as_str() {
                "char" => panic!(format!("error: unexpected pointer pointer to char: {}",
                                         field_name_raw)),
                "void" => {
                    //eprintln!("{}", format!("error: unexpected pointer pointer to void: {} -> {}",
                    //                     cmd.name.as_str(), field_name_raw));
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
                _ => panic!(format!("error: only expecting dynamic array type: {}",
                                    field_name_raw)),
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
                _ => panic!(format!("error: unexpected array type for non-pointer field: {}",
                                    field_name_raw)),
            }
        }
    }

    let optional_maybe = match field_first_indirection_optional(&field) && field.reference.is_some()
        && field.array.is_none() {
            true => quote!( Option<#ref_type #basetype> ),
            false => quote!( #ref_type #basetype ),
        };

    quote!( #field_name : #optional_maybe, )
}

fn make_inner_param(field: &Field) -> TokenStream {
    let field_name_raw = field_name(&field);

    let field_name = field_name_raw.as_code();

    let ptr_mut = match field.is_const {
        true => quote!( as_ptr() ),
        false => quote!( as_mut_ptr() ),
    };

    //if let Some(replace) = replace.get(field.name.as_ref().expect("error: outer fields should have a name").as_str()) {
    //    return quote!( #replace );
    //}

    //if let Some(array_param_raw) = count_cache.get(&field_name_raw) {
    //    let array_param = array_param_raw.as_code();
    //    quote!( #array_param.len() as _ )
    //}
    //else {

        match field.reference {
            Some(ReferenceType::Pointer) => {
                // NOTE assumption: if reference is pointer, then array is dynamic or
                // none
                match field.array {
                    Some(ArrayType::Dynamic) => {
                        quote!( #field_name.#ptr_mut as _ )
                    }
                    None => {
                        if field_first_indirection_optional(&field) {
                            quote!( #field_name.#ptr_mut as _ )
                        }
                        else {
                            quote!( #field_name )
                        }
                    }
                    _ => panic!(format!("error: only expecting dynamic array or not array: {}",
                                        field_name_raw)),
                }
            }
            Some(ReferenceType::PointerToPointer) => {
                quote!( #field_name as *mut &mut _ as *mut *mut _ )
            }
            Some(ReferenceType::PointerToConstPointer) => {
                match field.array {
                    Some(ArrayType::Dynamic) => quote!( #field_name as *const & _ as *const *const _ ),
                    _ => panic!(format!("error: only expecting dynamic array type: {}",
                                        field_name_raw)),
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
                    _ => panic!(format!("error: unexpected array type for non-pointer field: {}",
                                        field_name_raw)),
                }
            }
        }
    //}
}

//fn make_method_outer_params() {
//}

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

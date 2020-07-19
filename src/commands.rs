
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::utils;
use crate::global_data;
use crate::definitions;

use std::collections::HashMap;

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

fn field_first_indirection_optional(field: &vkxml::Field) -> bool {
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

fn field_name(field: &vkxml::Field) -> &str {
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

        let return_type = c_type(&cmd.return_type, WithLifetime::No, FieldContext::Member);
        let params1 = cmd.param.iter().map(|field|c_field(field, WithLifetime::No, FieldContext::FunctionParam));
        let params2 = params1.clone(); // because params is needed twice and quote will consume params1

        // create owner methods
        let owner_name = utils::make_handle_owner_name(cmd.param[0].basetype.as_str());

        let owner_method = make_owner_method(&cmd, parse_state);

        quote!{
            #[allow(non_camel_case_types)]
            pub type #pfn_name = extern "system" fn(
                #( #params1 ),*
            ) -> #return_type;

            struct #pfn_loader_name(#pfn_name);
            impl #pfn_loader_name {
                fn new() -> Self {
                    extern "system" fn default_function( #( #params2 ),* ) -> #return_type {
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
            #owner_method
        }
    });

    let static_command_definitions = static_commands.map(|cmd| {
        let name = cmd.name.as_code();
        let return_type = c_type(&cmd.return_type, WithLifetime::No, FieldContext::Member);
        let params1 = cmd.param.iter().map(|field|c_field(field, WithLifetime::No, FieldContext::FunctionParam));
        let params2 = params1.clone();

        let pfn_name = make_pfn_name(cmd.name.as_str());

        let raw_name = &cmd.name;

        quote!{
            pub type #pfn_name = extern "system" fn(
                #( #params1 ),*
            ) -> #return_type;

            #[link(name = "vulkan")]
            extern "system" {
                #[link_name = #raw_name]
                fn #name( #( #params2 ),* ) -> #return_type;
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

fn is_return_param(field: &&vkxml::Field) -> bool {
    field.reference.is_some() && field.is_const == false
}
fn not_return_param(field: &&vkxml::Field) -> bool {
    !is_return_param(field)
}

// this is for automatically generating methods which provide a more ideal rust interface for calling
// vulkan commands
fn make_owner_method(cmd: &Command, parse_state: &crate::ParseState) -> TokenStream {

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
    let owner_name = utils::make_handle_owner_name(cmd.param[0].basetype.as_str());

    let method_name_raw = case::camel_to_snake(cmd.name.as_str());
    let method_name = method_name_raw.as_code();

    let method_caller = match cmd.param[0].basetype.as_str() {
        "VkInstance" | "VkDevice" => quote!( self.commands.#name.0 ),
        _ => {
            quote!( self.dispatch_parent.commands.#name.0 )
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
                    method_params = quote!( self.handle, None.into() );
                    method_caller = quote!( self.commands.#name.0 );
                }
                _ => {
                    // for everything else, the second parameter should be the type we are
                    // destroying
                    let type_to_destroy = utils::make_handle_owner_name(cmd.param[1].basetype.as_str());
                    owner_name = quote!( #type_to_destroy );
                    method_params = quote!( self.dispatch_parent.handle, self.handle, None.into() );
                    method_caller = quote!( self.dispatch_parent.commands.#name.0 );
                }
            }

            quote!{
                impl Drop for #owner_name<'_> {
                    fn drop(&mut self) {
                        #method_caller(#method_params);
                    }
                }
            }
        }
        "free" => {

          eprint!("{} (", cmd.name.as_str());
          for param in cmd.param.iter() {
              eprint!("{}, ", param.name.as_ref().unwrap().as_str());
          }
          eprintln!(") -> {:?}", cmd.return_type.basetype);

          //quote!()
            //dbg!(&cmd);
            //let category_map = catagorize_fields(&cmd);
            //for category in category_map.iter() {
            //    dbg!(category);
            //}
            quote!()
        }
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

            let impl_lifetime;
            let call_lifetime;
            let self_lifetime;
            let with_lifetime;

            match method_verb {
                "create" => {
                    impl_lifetime = quote!('_);
                    call_lifetime = quote!('parent);
                    self_lifetime = quote!('parent);
                    with_lifetime = WithLifetime::Yes("'parent");
                }
                _ => {
                    impl_lifetime = quote!('_);
                    call_lifetime = quote!();
                    self_lifetime = quote!();
                    with_lifetime = WithLifetime::No
                }
            }

            // determine if method should call the vulkan command twice in order to query a size
            // field category
            let can_query_size = category_map
                .values()
                .find(|category| match category { FieldCatagory::SizeMut => true, _ => false } )
                .is_some();

            let fields_outer = cmd.param.iter().skip(1)
                .filter(|field| match category_map.get(field_name(field)).unwrap() {
                    FieldCatagory::Normal | FieldCatagory::NormalSized => true,
                    _ => false,
                })
            .map(|field|r_field(field, with_lifetime, FieldContext::FunctionParam, cmd.name.as_str()));

            // when a count/size variable affects multiple input arrays, set the size once
            // based on the first input array, and debug_assert that the other input arrays are the
            // same size
            // The HashMap is just incase a function has multiple different count/size variables
            let mut size_set = HashMap::new();
            let size_vars = cmd.param.iter()
                .filter_map(|field| {
                    let field_name_raw = field_name(field);
                    let field_name = field_name_raw.as_code();
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
                            let size = size_raw.as_code();
                            match field.array.as_ref().unwrap() {
                                vkxml::ArrayType::Static => None,
                                vkxml::ArrayType::Dynamic =>
                                {
                                    let is_size_set = size_set.get(size_raw).expect("error: NormalSized with no size");
                                    if !is_size_set {
                                        size_set.entry(size_raw).and_modify(|x| *x=true);
                                        Some(quote!( #size = #field_name.len() as _; ))
                                    } else {
                                        Some(quote!( debug_assert!(#field_name.len() == #size as _); ))
                                    }
                                }
                            }
                        }
                        _ => None,
                    }
                });

            let size_query = if can_query_size {
                let fields_inner = cmd.param.iter().skip(1)
                    .map( |field| {
                        let name_raw = field_name(&field);
                        let field_name = name_raw.as_code();
                        match category_map.get(name_raw).unwrap() {
                            FieldCatagory::ReturnSized => quote!( None.into() ),
                            _ => quote!( #field_name.into() ),
                        }
                    });
                Some( quote!{
                    #method_caller( #first_inner_param, #( #fields_inner ),* );
                })
            }
            else {
                None
            };

            let return_vars = cmd.param.iter()
                .filter_map(|field| {
                    let field_name_raw = field_name(field);
                    let field_name = field_name_raw.as_code();
                    match category_map.get(field_name_raw).unwrap() {
                        FieldCatagory::Return => {
                            Some(quote!( let mut #field_name = MaybeUninit::uninit(); ))
                        }
                        FieldCatagory::ReturnSized => {
                            let size = field.size.as_ref().unwrap().replace("::", ".").as_code();
                            Some(quote!( let mut #field_name = Vec::with_capacity(#size.value() as _); ))
                        }
                        _ => None,
                    }
                });

            let main_call = {
                let fields_inner = cmd.param
                    .iter()
                    .skip(1)
                    .map( |field| {
                        let name_raw = field_name(&field);
                        let field_name = name_raw.as_code();
                        match category_map.get(name_raw).unwrap() {
                            FieldCatagory::ReturnSized | FieldCatagory::Return => quote!( (&mut #field_name).into() ),
                            _ => quote!( #field_name.into() ),
                        }
                    });
                quote!{
                    #method_caller( #first_inner_param, #( #fields_inner ),* );
                }
            };

            let prep_return_vars = cmd.param.iter()
                .filter_map(|field| {
                    let field_name_raw = field_name(field);
                    let field_name = field_name_raw.as_code();
                    match category_map.get(field_name_raw).unwrap() {
                        FieldCatagory::Return => {
                            Some(quote!( let #field_name = unsafe { #field_name.assume_init() }; ))
                        }
                        FieldCatagory::ReturnSized => {
                            let size = field.size.as_ref().unwrap().replace("::", ".").as_code();
                            Some(quote!( unsafe { #field_name.set_len(#size.value() as _) }; ))
                        }
                        _ => None,
                    }
                });

            let return_count = category_map.iter()
                .filter(|(_field_name, category)|
                        match category { FieldCatagory::Return | FieldCatagory::ReturnSized => true, _ => false })
                .count();

            let return_code;
            let return_type;
            if return_count == 0 {
                return_type = Some( quote!(()) );
                return_code = None;
            }
            else {
                let return_field_types = cmd.param.iter().filter_map(|field| {
                    match category_map.get(field_name(field)).unwrap() {
                        FieldCatagory::Return | FieldCatagory::ReturnSized => Some(utils::r_return_type(field, with_lifetime)),
                        _ => None,
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
                        let field_name = field_name_raw.as_code();
                        match category_map.get(field_name_raw).unwrap() {
                            FieldCatagory::Return | FieldCatagory::ReturnSized => {
                                Some(quote!( #field_name ))
                            }
                            _ => None,
                        }
                    });
                return_code = Some( quote!{
                    let ret = ( #( #return_vars ),* ); //(A, B, ...);
                    (ret, self).ret()
                } );
            }

            quote!{
                //#multi_return_type
                impl #owner_name<#impl_lifetime> {
                    pub fn #method_name<#call_lifetime>(& #self_lifetime self, #( #fields_outer ),* ) -> #return_type {
                        #( #size_vars )*
                        #size_query
                        #( #return_vars )*
                        #main_call
                        #( #prep_return_vars )*
                        #return_code
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


use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;

pub fn make_pfn_name(cmd: &Command) -> TokenStream {
    format!("PFN_{}", cmd.name).as_code()
}

pub fn make_pfn_loader_name(cmd: &Command) -> TokenStream {
    format!("PFN_Loader_{}", cmd.name).as_code()
}

pub fn make_macro_name_instance(cmd_name: &str) -> TokenStream {
    format!("get_instance_cmd_{}", cmd_name).as_code()
}

pub fn make_macro_name_device(cmd_name: &str) -> TokenStream {
    format!("get_device_cmd_{}", cmd_name).as_code()
}

#[derive(Debug)]
struct CommandParts<'a> {
    verb: &'a str,
}

fn get_command_parts(cmd: &Command) -> CommandParts {
    let second_capital = cmd.name[3..].find(char::is_uppercase).expect("can't find delimiter to command verb") + 3;
    let verb = &cmd.name[2..second_capital];
    CommandParts {
        verb
    }
}

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

pub fn handle_commands(commands: &Commands) -> TokenStream {

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
        let pfn_loader_name = make_pfn_loader_name(&cmd);
        quote!( #name : #pfn_loader_name )
    }
    let instance_command_params = instance_commands.map(make_param);
    let device_command_params = device_commands.map(make_param);

    // make definitions for instance and device (non-static) commands
    let non_static_command_definitions = instance_and_device_commands.map(|cmd| {
        let name = cmd.name.as_code();
        let pfn_name = make_pfn_name(&cmd);
        let pfn_loader_name = make_pfn_loader_name(&cmd);
        let raw_name = &cmd.name;

        let return_type = make_field_type(&cmd.return_type);
        let params1 = cmd.param.iter().map(handle_field);
        let params2 = params1.clone(); // because params is needed twice and quote will consume params1

        //let load_function = match command_category(&cmd) {
        //    CommandCategory::Instance => quote!( GetInstanceProcAddr ),
        //    CommandCategory::Device => quote!( GetDeviceProcAddr ),
        //    _ => quote!(),
        //};

        //let command_category = match command_category(&cmd) {
        //    CommandCategory::Instance => quote!(CommandCategory::Instance),
        //    CommandCategory::Device => quote!(CommandCategory::Device),
        //    _ => panic!("unexpected command category for command definition"),
        //    //CommandCategory::Static => quote!(CommandCategory::Static),
        //};

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
            CommandCategory::Device => quote!( $cmd_container.#name.load() ),
            _ => quote!(),
        };

        //dbg!(get_command_parts(&cmd));

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
                //const fn command_category() -> CommandCategory {
                //    #command_category
                //}
            }
            macro_rules! #instance_macro_name{
                ( $instance:ident, $cmd_container:ident ) => { #is_instance_cmd }
            }
            macro_rules! #device_macro_name {
                ( $instance:ident, $cmd_container:ident ) => { #is_device_cmd }
            }
        }
    });

    let static_command_definitions = static_commands.map(|cmd| {
        let name = cmd.name.as_code();
        let return_type = make_field_type(&cmd.return_type);
        let params1 = cmd.param.iter().map(handle_field);
        let params2 = params1.clone();

        let pfn_name = make_pfn_name(&cmd);
        let pfn_loader_name = make_pfn_loader_name(&cmd);

        let raw_name = &cmd.name;

        let instance_macro_name = make_macro_name_instance(&cmd.name);
        let device_macro_name = make_macro_name_device(&cmd.name);

        quote!{
            pub type #pfn_name = extern "system" fn(
                #( #params1 ),*
            ) -> #return_type;

            // this is provided for type uniformity
            // this makes it easier to deal with feature/extention loading
            //struct #pfn_loader_name;
            //impl #pfn_loader_name {
            //    fn load<F>(&mut self, mut f: F) where F: FnMut(&::std::ffi::CStr) -> *const c_void {
            //        panic!("shouldn't actually try loading static functions");
            //    }
            //    //const fn command_category() -> CommandCategory {
            //    //    CommandCategory::Static
            //    //}
            //}

            macro_rules! #instance_macro_name {
                ( $instance:ident, $cmd_container:ident ) => { }
            }
            macro_rules! #device_macro_name {
                ( $instance:ident, $cmd_container:ident ) => { }
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

        struct DeviceCommands {
            #( #device_command_params ),*
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

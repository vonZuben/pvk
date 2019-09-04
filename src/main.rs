#![recursion_limit = "250"]

use quote::quote;

// just for coverting the xml file into a vkxml registry
extern crate vk_parse;

#[macro_use]
mod utils;
mod constants;
mod definitions;
mod enumerations;
mod commands;
mod features;

//mod take_list;

use std::path::Path;

use vkxml::*;
use proc_macro2::{TokenStream};

use definitions::*;
use enumerations::*;
use constants::*;
use commands::*;
use features::*;

// keep certain mutable state while parsing the registry
pub struct ParseState<'a> {
    //command_list: take_list::TakeList<&'a vkxml::Command>,
    previous_feature_instance: Option<TokenStream>,
    previous_feature_device: Option<TokenStream>,
    phantom: ::std::marker::PhantomData<&'a ()>,
}

pub fn vkxml_registry_token_stream(reg_elem: &vkxml::RegistryElement, parse_state: &mut ParseState) -> TokenStream {
    match reg_elem {
        RegistryElement::Definitions(definition) => {
            handle_definitions(definition, parse_state)
        },
        RegistryElement::Constants(cnts) => {
            handle_constants(cnts)
        },
        RegistryElement::Enums(enums) => {
            handle_enumerations(enums)
        }
        RegistryElement::Commands(cmds) => {
            handle_commands(cmds)
        }
        RegistryElement::Features(features) => {
            handle_features(features, parse_state)
        }
        //RegistryElement::Extensions(extensions) => {
        //    dbg!(extensions);
        //    quote!()
        //}
        _ => quote!(),
    }
}

#[allow(unused)]
fn debug(registry: &Registry) {
    // collect all of the commands for use in defining dispatchable handles
    //let mut command_list: take_list::TakeList<_> = registry.elements.iter()
    let command_list: Vec<_> = registry.elements.iter()
        .filter_map(|elem| match elem {
                RegistryElement::Commands(cmds) => Some(cmds.elements.iter()),
                _ => None,
            })
        .flatten()
        .collect();

    for cn in command_list.iter().filter(|node| { node.name.contains("Cmd") }) {
        println!();
        print!("{} : ", &cn.name);
        for param in &cn.param {
            print!("{}, ", &param.name.as_ref().unwrap());
            //dbg!(&param);
        }
        print!(" -> {}", &cn.return_type.name.as_ref().map(|name| name.as_str() ).unwrap_or(&"NO RETURN"));
    }
}

fn main() {
    // this it the easier to parse registry
    let registry = vk_parse::parse_file_as_vkxml(Path::new("vk.xml"));

    // this registry is closer to the xml formate, but it sucks to parse
    // but it does include the aliases
    let registry2 = vk_parse::parse_file(Path::new("vk.xml"));

    //debug(&registry);

    // TODO remove this later if it dosnt get used
    let mut parse_state = ParseState {
        //command_list,
        previous_feature_instance: None,
        previous_feature_device: None,
        phantom: ::std::marker::PhantomData,
    };

    //println!("{:#?}", registry2);

    let tokens = registry.elements.iter().map(|relem| vkxml_registry_token_stream(relem, &mut parse_state));

    let aliases = registry2
        .0
        .iter()
        .filter_map(|item| match item {
            vk_parse::RegistryChild::Types(ref ty) => {
                Some(generate_aliases_of_types(ty))
            }
            _ => None,
        });

    let allow_vulkan_name_formats = quote!{
        #![allow(non_camel_case_types)]
        #![allow(non_snake_case)]
        #![allow(non_upper_case_globals)]
        #![allow(unused)]
    };

    let initial_test_code = quote!{
        use std::os::raw::*;
        macro_rules! vk_make_version {
            ($major:expr, $minor:expr, $patch:expr) => {
                (($major as u32) << 22) | (($minor as u32) << 12) | $patch as u32
            };
        }
        fn main(){
            let mut inst: Instance = ::std::ptr::null();

            let app_name = ::std::ffi::CString::new("Hello World!").unwrap();
            let engine_name = ::std::ffi::CString::new("Hello Engine!").unwrap();

            let mut app_info = ApplicationInfo::builder();
            app_info
                .s_type(StructureType::APPLICATION_INFO)
                .p_application_name(app_name.as_c_str())
                .application_version(vk_make_version!(1, 0, 0))
                .p_engine_name(engine_name.as_c_str())
                .engine_version(vk_make_version!(1, 0, 0))
                .api_version(vk_make_version!(1, 1, 1));

            let mut create_info = InstanceCreateInfo::builder();
            create_info
                .s_type(StructureType::INSTANCE_CREATE_INFO)
                .p_application_info(&app_info);

            let res = unsafe {
                CreateInstance(
                    &*create_info as *const InstanceCreateInfo,
                    ::std::ptr::null(),
                    &mut inst as *mut Instance,
                    )
            };
            println!("{}", res);
            let mut instance_commands = InstanceCommands::new();
            let ver = VERSION_1_1;
            ver.load_instance_commands(&inst, &mut instance_commands);

            let mut phd: PhysicalDevice = std::ptr::null();
            let mut phd_count: u32 = 0;
            instance_commands.EnumeratePhysicalDevices.0(inst, &mut phd_count as *mut u32, std::ptr::null_mut());
            println!("num physical devices: {}", phd_count);

            // test Flags printing
            let flags: QueueFlags = unsafe { std::mem::transmute(0x5) };
            println!("{}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::SHADER_STAGE_ALL;
            println!("{}", flags);
            let flags: ShaderStageFlags = unsafe { std::mem::transmute(0x5) };
            println!("{}", flags);

            // test 1_1 feature command ?
            //let mut phd: PhysicalDevice = std::ptr::null();
            //let mut phd_count: u32 = 0;
            //instance_commands.EnumeratePhysicalDevices.0(inst, &mut phd_count as *mut u32, std::ptr::null_mut());
            //println!("{}", phd_count);
        }
    };

    let platform_specific_types = utils::platform_specific_types();

    // code used internally by the generated vk.rs
    let util_code = quote!{
        // used for printing flagbits
        // find and return the lowest bit in the input
        // then remove the lowest bit from the input
        fn take_lowest_bit(input: &mut i32) -> Option<i32> {
            let lowest_bit = *input & (*input).wrapping_neg();
            *input = *input ^ lowest_bit;
            if lowest_bit == 0 {
                None
            }
            else {
                Some(lowest_bit)
            }
        }
    };

    let q = quote!{
        #allow_vulkan_name_formats
        #initial_test_code
        #util_code
        #platform_specific_types
        #(#tokens)*
        #(#aliases)*
    };

    //for node in parse_state.command_list.iter() {
    //    dbg!(&node.data().name);
    //    dbg!(commands::command_category(node.data()));
    //}

    println!("{}", q);

}


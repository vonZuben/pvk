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
mod extensions;

//mod take_list;

use std::path::Path;
use std::collections::HashMap;

use vkxml::*;
use proc_macro2::{TokenStream};

use definitions::*;
use enumerations::*;
use constants::*;
use commands::*;
use features::*;
use extensions::*;

// keep certain mutable state while parsing the registry
pub struct ParseState<'a> {
    //command_list: take_list::TakeList<&'a vkxml::Command>,
    previous_feature_instance: Option<TokenStream>,
    previous_feature_device: Option<TokenStream>,

    // register command types when processing commands
    // then use in features and extension code
    // NOTE assumes that commands are always parsed first
    command_type_cache: HashMap<&'a str, commands::CommandCategory>,

    enum_constants_name_cache: HashMap<&'a str, ()>,

    // <enum_name, Vec<varients>>
    enum_cache: HashMap<&'a str, Vec<&'a str>>,

    command_alias_cache: HashMap<&'a str, &'a str>,

    handle_cache: Vec<&'a vkxml::Handle>,

    phantom: ::std::marker::PhantomData<&'a ()>,
}

pub fn vkxml_registry_token_stream<'a>(reg_elem: &'a vkxml::RegistryElement, parse_state: &mut ParseState<'a>) -> TokenStream {
    match reg_elem {
        RegistryElement::Definitions(definition) => {
            handle_definitions(definition, parse_state)
        },
        RegistryElement::Constants(cnts) => {
            handle_constants(cnts)
        },
        RegistryElement::Enums(enums) => {
            handle_enumerations(enums, parse_state)
        }
        RegistryElement::Commands(cmds) => {
            handle_commands(cmds, parse_state)
        }
        RegistryElement::Features(features) => {
            handle_features(features, parse_state)
        }
        RegistryElement::Extensions(extensions) => {
            handle_extensions(extensions, parse_state)
        }
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

    // this registry is closer to the xml format, but it sucks to parse
    // but it does include the aliases
    let registry2 = vk_parse::parse_file(Path::new("vk.xml"));

    let cmd_alias_iter = registry2.0.iter()
        .filter_map(
            |elem| match elem
            { vk_parse::RegistryChild::Commands(cmd) => Some(cmd), _ => None } )
        .map(|commands|commands.children.iter())
        .flatten()
        .filter_map(
            |command| match command
            { vk_parse::Command::Alias { name, alias } => Some((name.as_str(), alias.as_str())), _ => None } );

    //debug(&registry);

    // TODO remove this later if it dosnt get used
    let mut parse_state = ParseState {
        //command_list,
        previous_feature_instance: None,
        previous_feature_device: None,

        command_type_cache: HashMap::new(),

        enum_constants_name_cache: HashMap::new(),

        enum_cache: HashMap::new(),

        command_alias_cache: HashMap::new(),

        handle_cache: Vec::new(),

        phantom: ::std::marker::PhantomData,
    };

    for alias_tuple in cmd_alias_iter {
        // insert a mapping for     alias -> cmd
        // and for                  cmd -> alias
        assert!(parse_state.command_alias_cache.insert(alias_tuple.0, alias_tuple.1).is_none(), true);
        assert!(parse_state.command_alias_cache.insert(alias_tuple.1, alias_tuple.0).is_none(), true);
    }

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
            println!("instance creation: {}", res);
            let mut instance_commands = InstanceCommands::new();
            let ver = VERSION_1_1;
            ver.load_instance_commands(&inst, &mut instance_commands);

            let instance = InstanceManager::new(inst, instance_commands, Box::new(ver));

            //let mut phd: PhysicalDevice = std::ptr::null();
            //let mut phd_count: u32 = 0;
            //instance_commands.EnumeratePhysicalDevices.0(inst, &mut phd_count as *mut u32, std::ptr::null_mut());
            let pd = instance.enumerate_physical_devices();
            println!("num physical devices: {}", pd.len());

            // test Flags printing
            let flags: QueueFlags = QueueFlags::GRAPHICS | QueueFlags::COMPUTE;
            println!("{}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::ALL;
            println!("{}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::VERTEX
                | ShaderStageFlags::FRAGMENT | ShaderStageFlags::TESSELLATION_CONTROL;
            println!("{}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::ALL_GRAPHICS;
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
        //struct OnesIter(u32);
        //impl Iterator for OnesIter {
        //    type Item = u32;
        //    fn next(&mut self) -> Option<Self::Item> {
        //        let lowest_bit = self.0 & (self.0 as i32).wrapping_neg() as u32;
        //        if lowest_bit == 0 {
        //            None
        //        }
        //        else {
        //            self.0 ^= lowest_bit;
        //            Some(lowest_bit)
        //        }
        //    }
        //}
        trait Feature {
            fn load_instance_commands(&self, instance: &Instance, inst_cmds: &mut InstanceCommands);
            fn load_device_commands(&self, device: &Device, dev_cmds: &mut DeviceCommands);
        }
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
        macro_rules! vk_bitflags_wrapped {
            ($name: ident) => {

                impl Default for $name{
                    fn default() -> $name {
                        $name(0)
                    }
                }
                impl ::std::fmt::Debug for $name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        write!(f, "{}({:b})", stringify!($name), self.0)
                    }
                }

                impl $name {
                    #[inline]
                    pub fn empty() -> $name {
                        $name(0)
                    }

                    // TODO fix $all
                    //#[inline]
                    //pub fn all() -> $name {
                    //    $name($all)
                    //}

                    #[inline]
                    pub fn from_raw(x: Flags) -> Self { $name(x) }

                    #[inline]
                    pub fn as_raw(self) -> Flags { self.0 }

                    #[inline]
                    pub fn is_empty(self) -> bool {
                        self == $name::empty()
                    }

                    //#[inline]
                    //pub fn is_all(self) -> bool {
                    //    self & $name::all() == $name::all()
                    //}

                    //#[inline]
                    //pub fn intersects(self, other: $name) -> bool {
                    //    self & other != $name::empty()
                    //}

                    /// Returns whether `other` is a subset of `self`
                    #[inline]
                    pub fn contains(self, other: $name) -> bool {
                        self & other == other
                    }
                }

                impl ::std::ops::BitOr for $name {
                    type Output = $name;

                    #[inline]
                    fn bitor(self, rhs: $name) -> $name {
                        $name (self.0 | rhs.0 )
                    }
                }

                impl ::std::ops::BitOrAssign for $name {
                    #[inline]
                    fn bitor_assign(&mut self, rhs: $name) {
                        *self = *self | rhs
                    }
                }

                impl ::std::ops::BitAnd for $name {
                    type Output = $name;

                    #[inline]
                    fn bitand(self, rhs: $name) -> $name {
                        $name (self.0 & rhs.0)
                    }
                }

                impl ::std::ops::BitAndAssign for $name {
                    #[inline]
                    fn bitand_assign(&mut self, rhs: $name) {
                        *self = *self & rhs
                    }
                }

                impl ::std::ops::BitXor for $name {
                    type Output = $name;

                    #[inline]
                    fn bitxor(self, rhs: $name) -> $name {
                        $name (self.0 ^ rhs.0 )
                    }
                }

                impl ::std::ops::BitXorAssign for $name {
                    #[inline]
                    fn bitxor_assign(&mut self, rhs: $name) {
                        *self = *self ^ rhs
                    }
                }

                //impl ::std::ops::Sub for $name {
                //    type Output = $name;

                //    #[inline]
                //    fn sub(self, rhs: $name) -> $name {
                //        self & !rhs
                //    }
                //}

                //impl ::std::ops::SubAssign for $name {
                //    #[inline]
                //    fn sub_assign(&mut self, rhs: $name) {
                //        *self = *self - rhs
                //    }
                //}

                //impl ::std::ops::Not for $name {
                //    type Output = $name;

                //    #[inline]
                //    fn not(self) -> $name {
                //        self ^ $name::all()
                //    }
                //}
            }
        }

        // it is safe to transmute Option<&t> to raw pointer
        // because the null pointer optimization guarantees that
        // Option<&T> is ABI compatible with raw C pointers for FFI
        // (see the rust nomicon)
        pub trait AsRawPtr<T> {
            fn as_ptr(&self) -> *const T;
        }
        impl<'a, T> AsRawPtr<T> for Option<&'a T> {
            fn as_ptr(&self) -> *const T {
                //unsafe { ::std::mem::transmute::<Option<&T>, *const T>(*self) }
                unsafe { *(self as *const Option<&T> as *const *const T) }
            }
        }
        // NOTE note sure if this should be included
        //impl<'a, T> AsRawPtr<T> for Option<&'a mut T> {
        //    fn as_ptr(&self) -> *const T {
        //        // this is safe because it is safe to take shared reference from a mutable
        //        // reference
        //        //
        //        // however, the mutable reference shouldn't be used while the raw pointer still
        //        // exists
        //        unsafe { ::std::mem::transmute::<Option<&mut T>, *const T>(*self) }
        //    }
        //}

        pub trait AsRawMutPtr<T> {
            fn as_mut_ptr(&self) -> *mut T;
        }
        impl<'a, T> AsRawMutPtr<T> for Option<&'a mut T> {
            fn as_mut_ptr(&self) -> *mut T {
                //unsafe { ::std::mem::transmute::<Option<&mut T>, *mut T>(*self) }
                unsafe { *(self as *const Option<&mut T> as *const *mut T) }
            }
        }
    };

    let mut q = quote!{
        #allow_vulkan_name_formats
        #initial_test_code
        #util_code
        #platform_specific_types
        #(#tokens)*
        #(#aliases)*
    };

    let post_process_handles = post_process_handles(&parse_state);
    let post_process_enum_display_code = make_enumeration_display_code(&parse_state);

    q.extend(post_process_handles);
    q.extend(post_process_enum_display_code);

    //for node in parse_state.command_list.iter() {
    //    dbg!(&node.data().name);
    //    dbg!(commands::command_category(node.data()));
    //}

    println!("{}", q);

}


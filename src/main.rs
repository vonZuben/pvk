#![recursion_limit = "1000"]

use quote::quote;

// just for coverting the xml file into a vkxml registry
extern crate vk_parse;

use once_cell::sync::OnceCell;

#[macro_use]
mod utils;
mod global_data;
mod constants;
mod definitions;
mod enumerations;
mod commands;
mod features;
mod extensions;
mod ty;

//mod take_list;

use utils::StrAsCode;

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

    enum_constants_name_cache: HashMap<&'a str, ()>,

    command_alias_cache: HashMap<&'a str, &'a str>,

    handle_cache: Vec<&'a vkxml::Handle>,

    struct_with_sync_member: HashMap<&'a str, &'a str>,

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
            handle_enumerations(enums)
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

#[allow(unused)]
fn print_command_verbs(registry: &Registry) {
    let command_list: Vec<_> = registry.elements.iter()
        .filter_map(|elem| match elem {
                RegistryElement::Commands(cmds) => Some(cmds.elements.iter()),
                _ => None,
            })
        .flatten()
        .collect();

    for cn in command_list.iter() {
        println!();

        if cn.param.iter().find(|param| param.sync.is_some()).is_some() {
            print!("SOME SYNC ");
        }
        else {
            print!("NO SYNC ");
        }

        print!("{} : ", &cn.name);
        for param in &cn.param {
            print!("{}", param.name.as_ref().unwrap().as_str());
            if param.sync.is_some() {
                print!(" (sync: {})", param.sync.as_ref().unwrap());
            }
            print!(", ");
        }
        print!(" -> {}", &cn.return_type.name.as_ref().map(|name| name.as_str() ).unwrap_or(&"NO RETURN"));
    }
}

fn main() {
    // this it the easier to parse registry
    let registry = vk_parse::parse_file_as_vkxml(Path::new("vk.xml")).expect("failed to parse and convert vk.xml");

    global_data::REGISTRY.set(registry);

    let registry = global_data::REGISTRY.get().unwrap();

    // this registry is closer to the xml format, but it sucks to parse
    // but it does include the aliases
    let (registry2, _) = vk_parse::parse_file(Path::new("vk.xml")).expect("failed to parse vk.xml");

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
    //print_command_verbs(&registry);
    //return;

    // TODO remove this later if it dosnt get used
    let mut parse_state = ParseState {
        //command_list,
        previous_feature_instance: None,
        previous_feature_device: None,

        enum_constants_name_cache: HashMap::new(),

        command_alias_cache: HashMap::new(),

        handle_cache: Vec::new(),

        struct_with_sync_member: HashMap::new(),

        phantom: ::std::marker::PhantomData,
    };

    global_data::generate(&registry, &registry2);

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

        //trace_macros!(true);
    };

    let initial_test_code = quote!{
        // macro_rules! vk_make_version {
        //     ($major:expr, $minor:expr, $patch:expr) => {
        //         (($major as u32) << 22) | (($minor as u32) << 12) | $patch as u32
        //     };
        // }

        /// VkVersion(major, minor, patch)
        /// generates u32 version number based on
        /// Vulkan specification
        #[derive(Default, Copy, Clone, Debug)]
        pub struct VkVersion(pub u32, pub u32, pub u32);

        impl VkVersion {
            fn make(self) -> u32 {
                let major = self.0;
                let minor = self.1;
                let patch = self.2;
                vk_make_version(major, minor, patch)
            }
        }

        fn vk_make_version(major: u32, minor: u32, patch: u32) -> u32 {
            major << 22 | minor << 12 | patch
        }

        fn main(){

            let mut instance = CreateInstance::new()
                .app_name("heyo")
                .create()
                .unwrap();

            //let mut phd: PhysicalDevice = std::ptr::null();
            //let mut phd_count: u32 = 0;
            //instance_commands.EnumeratePhysicalDevices.0(inst, &mut phd_count as *mut u32, std::ptr::null_mut());
            let pd = instance.enumerate_physical_devices().unwrap();
            //println!("{:?}", inst);
            println!("{:?}", instance);
            for pd in &pd {
                println!("{:?}", pd);
                println!("{:?}", pd.get_physical_device_properties());
            }
            println!("num physical devices: {}", pd.len());

            // test Flags printing
            let flags: QueueFlags = QueueFlags::GRAPHICS | QueueFlags::COMPUTE;
            println!("{}", flags);
            println!("{:?}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::ALL;
            println!("{}", flags);
            println!("{:?}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::VERTEX
                | ShaderStageFlags::FRAGMENT | ShaderStageFlags::TESSELLATION_CONTROL;
            println!("{}", flags);
            println!("{:?}", flags);
            let flags: ShaderStageFlags = ShaderStageFlags::ALL_GRAPHICS;
            println!("{}", flags);
            println!("{:?}", flags);

            let u = ClearColorValue { int_32: [1, 1, 1, 1]};
            let i = ClearValue { color: u };
            println!("{:?}", i);

            let mb = handle_slice![instance, instance];

            for i in &mb {
                println!("{:?}", i);
            }

            let mb = mut_handle_slice![instance];

            for i in &mb {
                println!("{:?}", i);
            }

            //test 1_1 feature command ?
            //let mut phd: MaybeUninit<PhysicalDevice> = MaybeUninit::uninit();
            //let mut phd_count: u32 = 0;
            //instance_commands.EnumeratePhysicalDevices.0(inst, (&mut phd_count).into(), None.into());
            //println!("{}", phd_count);
        }
    };

    let platform_specific_types = utils::platform_specific_types();

    let result_members = global_data::result_members();

    let result_ok: Vec<_> = result_members.iter().filter_map(|member| {
        if !member.is_err {
            let name = member.name.as_code();
            Some( quote!(#name) )
        }
        else {
            None
        }
    }).collect();
    let result_ok = result_ok.as_slice();

    let result_err: Vec<_> = result_members.iter().filter_map(|member| {
        if member.is_err {
            let name = member.name.as_code();
            Some( quote!(#name) )
        }
        else {
            None
        }
    }).collect();
    let result_err = result_err.as_slice();

    let ext_c_names = global_data::extension_maps().iter()
        .map(|(c_name, _name)| {
            // dbg!(c_name.as_bytes());
            let c_name = c_name.as_code();
            quote!( #c_name )
        });

    let ext_loader_names = global_data::extension_maps().iter()
        .map(|(_c_name, name)| {
            let name = utils::extension_loader_name(name).as_code();
            quote!( #name )
        });


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

        #[derive(Default)]
        pub struct CreateInstance<'a> {
            app_name: CString,
            app_version: VkVersion,
            engine_name: CString,
            engine_version: VkVersion,
            // Vulkan spec: The patch version number specified in apiVersion is ignored when creating an instance object.
            // Only the major and minor versions of the instance must match those requested in apiVersion
            // therefore, I will only accept the custom Feature typs that I generate
            api_version: Option<Box<dyn Feature>>,
            enabled_layers: Vec<&'a dyn VkLayerName>,
            enabled_extensions: Vec<&'a dyn VkExtensionLoader>,
        }

        impl<'a> CreateInstance<'a> {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn app_name(mut self, app_name: &str) -> Self {
                self.app_name = CString::new(app_name).expect("str should not have internal null, and thus CString::new should never fail");
                self
            }

            pub fn app_version(mut self, app_version: VkVersion) -> Self {
                self.app_version = app_version;
                self
            }

            pub fn engine_name(mut self, engine_name: &str) -> Self {
                self.engine_name = CString::new(engine_name).expect("str should not have internal null, and thus CString::new should never fail");
                self
            }

            pub fn engine_version(mut self, engine_version: VkVersion) -> Self {
                self.engine_version = engine_version;
                self
            }

            pub fn api_version(mut self, api_version: impl Into<FeatureWrapper>) -> Self {
                self.api_version = Some(api_version.into().0);
                self
            }

            pub fn enabled_layers<L>(mut self, enabled_layers: impl IntoIterator<Item = L>) -> Self
                where L: Into<LayerWrapper<'a>>
            {
                self.enabled_layers = enabled_layers.into_iter()
                    .map( |l| l.into().0 )
                    .collect();
                self
            }

            pub fn enabled_extensions<E>(mut self, enabled_extensions: impl IntoIterator<Item = E>) -> Self
                where E: Into<ExtLoaderWrapper> + 'a
            {
                self.enabled_extensions = enabled_extensions.into_iter()
                    .map( |e| e.into().0 )
                    .collect();
                self
            }

            pub fn create(&self) -> VkResult<InstanceOwner<'static>> {
                let app_name: MyStr = (&self.app_name).into();
                let engine_name: MyStr = (&self.engine_name).into();

                let api_version: Box<dyn Feature> = self.api_version
                    .as_ref()
                    .map(|boxed_ver| boxed_ver.clone())
                    .unwrap_or(Box::new(VERSION_1_0));

                let app_info = ApplicationInfo! {
                    p_application_name: app_name,
                    application_version: self.app_version.make(),
                    p_engine_name: engine_name,
                    engine_version: self.engine_version.make(),
                    api_version: api_version.version(),
                };

                let enabled_layers = ArrayArray(self.enabled_layers.iter()
                                                .map( |layer| layer.layer_name().into() ).collect());
                let enabled_extensions = ArrayArray(self.enabled_extensions.iter()
                                                .map( |extension| extension.extension_name().into() ).collect());

                let create_info = InstanceCreateInfo!{
                    p_application_info: &app_info,
                    pp_enabled_layer_names: &enabled_layers,
                    pp_enabled_extension_names: &enabled_extensions,
                };

                let mut inst: MaybeUninit<Instance> = MaybeUninit::uninit();

                let vk_result = unsafe {
                    CreateInstance(
                        (&create_info).to_c(),
                        None.to_c(),
                        (&mut inst).to_c(),
                        )
                };

                if vk_result.is_err() {
                    vk_result.err()
                }
                else {
                    let inst = unsafe { inst.assume_init() };

                    let mut instance_commands = InstanceCommands::new();
                    api_version.load_instance_commands(inst, &mut instance_commands);
                    for extension in &self.enabled_extensions {
                        extension.load_instance_commands(inst, &mut instance_commands);
                    }

                    let instance = InstanceOwner::new(inst, instance_commands, api_version);

                    vk_result.success(instance)
                }
            }
        }

        #[derive(Debug)]
        pub enum VkResult<T> {
            //#( #result_members_code ),*
            #( pub #result_ok(T) , )*
            #( pub #result_err , )*
        }

        impl<T> VkResult<T> {
            pub fn unwrap(self) -> T {
                match self {
                    #( Self::#result_ok(t) => t, )*
                    #( Self::#result_err => panic!(concat!("failed to unwrap VkResult::", stringify!(#result_err))), )*
                }
            }
        }

        impl crate::VkResultRaw {
            // takes a possible value to wrap
            fn success<T>(self, t: T) -> VkResult<T> {
                match self {
                    //#( #success_members , )*
                    #( VkResultRaw::#result_ok => VkResult::#result_ok(t) , )*
                    //Some( quote!(#name => VkResult::#name(t)) )
                    x => panic!("vk.rs error. Can't handle result code {:?} (not success)", x),
                }
            }
            // if err then we don't want to return anything
            fn err<T>(self) -> VkResult<T> {
                match self {
                    //#( #err_members , )*
                    #( VkResultRaw::#result_err => VkResult::#result_err , )*
                    x => panic!("vk.rs error. Can't handle result code {:?} (not an err)", x),
                }
            }

            fn is_err(self) -> bool {
                match self {
                    #( VkResultRaw::#result_err => true , )*
                    _ => false ,
                }
            }
        }

        pub struct FeatureWrapper(Box<dyn Feature>);

        trait Feature : FeatureCore + Send + Sync + 'static {}
        impl<T> Feature for T where T: FeatureCore + Send + Sync + 'static {}

        trait FeatureCore {
            fn load_instance_commands(&self, instance: Instance, inst_cmds: &mut InstanceCommands);
            fn load_device_commands(&self, device: Device, dev_cmds: &mut DeviceCommands);
            fn version(&self) -> u32;
            fn clone_feature(&self) -> Box<dyn Feature>;
        }

        impl Clone for Box<dyn Feature> {
            fn clone(&self) -> Self {
                self.clone_feature()
            }
        }

        macro_rules! noop {
            () => {
                assert!(true);
            };
        }

        trait VkExtension {
            fn extension_name(&self) -> &CStr;
        }

        trait VkExtensionLoader : VkExtension {
            fn load_instance_commands(&self, instance: Instance, commands: &mut InstanceCommands) {
                noop!();
            }
            fn load_device_commands(&self, device: Device, commands: &mut DeviceCommands) {
                noop!();
            }
        }

        #[derive(Clone, Copy)]
        pub struct ExtLoaderWrapper(&'static dyn VkExtensionLoader);

        impl From<&ExtensionProperties<'_>> for ExtLoaderWrapper {
            fn from(e: &ExtensionProperties<'_>) -> Self {
                ExtLoaderWrapper(ex_name_to_extension_loader(e))
            }
        }

        impl From<&ExtLoaderWrapper> for ExtLoaderWrapper {
            fn from(e: &ExtLoaderWrapper) -> Self {
                *e
            }
        }

        impl VkExtension for ExtensionProperties<'_> {
            fn extension_name(&self) -> &CStr {
                // self.extention_name should always be a valid c string
                // unless the vulkan implementation (driver) is wrong
                // do I need to guard against bad drivers?
                // hmmm?
                // or it is fine since this will only be used internally to
                // pass the string straigth back to the vulkan driver via
                // extension loading
                unsafe { CStr::from_ptr(self.extension_name.as_ptr()) }
            }
        }

        fn ex_name_to_extension_loader(e: &impl VkExtension) -> &'static dyn VkExtensionLoader {
            match e.extension_name().to_bytes() {
                #( #ext_c_names => &#ext_loader_names, )*
                _ => panic!("unrecognized extension name {:?}. Possibly unsupported by current version of vk.rs?",
                        e.extension_name()
                    )
            }
        }

        pub struct LayerWrapper<'a>(&'a dyn VkLayerName);

        trait VkLayerName {
            fn layer_name(&self) -> &CStr;
        }

        #[derive(Clone, Debug)]
        pub struct VkLayer(CString);

        impl VkLayer {
            fn from_str(s: &str) -> Self {
                Self(CString::new(s).expect("str should not have internal null"))
            }
        }

        impl VkLayerName for VkLayer {
            fn layer_name(&self) -> &CStr {
                self.0.as_c_str()
            }
        }

        impl<'a> From<&'a VkLayer> for LayerWrapper<'a> {
            fn from(vkl: &'a VkLayer) -> Self {
                Self(vkl)
            }
        }

        impl VkLayerName for LayerProperties<'_> {
            fn layer_name(&self) -> &CStr {
                // self.layer_name should always be a valid c string
                // unless the vulkan implementation (driver) is wrong
                // do I need to guard against bad drivers?
                // hmmm?
                // or it is fine since this will only be used internally to
                // pass the string straigth back to the vulkan driver via
                // extension loading
                unsafe { CStr::from_ptr(self.layer_name.as_ptr()) }
            }
        }

        impl<'a> From<&'a LayerProperties<'_>> for LayerWrapper<'a> {
            fn from(l: &'a LayerProperties) -> Self {
                Self(l)
            }
        }

        use std::mem::MaybeUninit;
        use std::marker::PhantomData;
        use std::os::raw::*;
        use std::ffi::{CStr, CString};

        use handles::{MutHandle, HandleOwner};

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

        trait Len {
            fn len(&self) -> usize;
        }

        impl<T> Len for Option<&[T]> {
            fn len(&self) -> usize {
                match self {
                    Some(a) => a.len(),
                    None => 0,
                }
            }
        }

        #[derive(Clone, Copy)]
        #[repr(transparent)]
        // & c_char here is a reference to the fits character of a c style stirng
        // use & for non-nullable pointer Option<MyStr> (for same ABI as MyStr)
        pub struct MyStr<'a>(&'a c_char);

        impl<'a, C: AsRef<CStr>> From<&'a C> for MyStr<'a> {
            fn from(c: &'a C) -> Self {
                // safe because CStr.as_ptr() should never return null-ptr unless improperly (and unsafely) created
                // also we borrow the owner of the CStr content so it should remain valid
                Self(unsafe { ::std::mem::transmute(c.as_ref().as_ptr()) } )
            }
        }

        impl<'a> From<&'a CStr> for MyStr<'a> {
            fn from(c: &'a CStr) -> Self {
                // safe because CStr.as_ptr() should never return null-ptr unless improperly (and unsafely) created
                // also we borrow the owner of the CStr content so it should remain valid
                Self(unsafe { ::std::mem::transmute(c.as_ptr()) } )
            }
        }

        #[repr(transparent)]
        #[derive(Clone, Copy)]
        struct ArrayString<A: AsRef<[c_char]>>(A);

        impl<A: AsRef<[c_char]>> ::std::fmt::Debug for ArrayString<A> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let char_array = self.0.as_ref();
                for c in char_array {
                    if *c == 0 {
                        break;
                    } else {
                        write!(f, "{}", *c as u8 as char)?;
                    }
                }
                Ok(())
            }
        }

        impl<A: AsRef<[c_char]>> ::std::ops::Deref for ArrayString<A> {
            type Target = A;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        // convert rust types to c types
        // used for building c structs witha rust interface, and
        // for building rust wrappers around commands
        trait ConvertToC<C> {
            fn to_c(self) -> C;
        }

        macro_rules! handle_slice {
            ( $( $($handle:expr),+ $(,)? )? ) => {
                [
                    $(
                        $( $handle.handle() ),+
                    )?
                ]
            }
        }

        macro_rules! mut_handle_slice {
            ( $( $($handle:expr),+ $(,)? )? ) => {
                [
                    $(
                        $( $handle.mut_handle() ),+
                    )?
                ]
            }
        }

        pub mod handles {

            use super::*;

            pub trait HandleOwner<'owner, H: 'owner> {
                fn handle(&'owner self) -> H;
                fn mut_handle(&'owner mut self) -> MutHandle<H> {
                    self.into()
                }
            }

            // ============= MutHandle<T> ================
            // Can only be created by mutably borrowing a HandleOwner
            #[derive(Debug, Clone, Copy)]
            #[repr(transparent)]
            pub struct MutHandle<T>(T);

            impl<'owner, H: 'owner> MutHandle<H> {
                pub fn new<O>(o: &'owner mut O) -> Self
                where
                    O: HandleOwner<'owner, H> + ?Sized
                {
                    Self(o.handle())
                }
            }

            impl<'owner, H: 'owner, O> From<&'owner mut O> for MutHandle<H>
                where
                    O: HandleOwner<'owner, H> + ?Sized
            {
                fn from(o: &'owner mut O) -> Self {
                    Self::new(o)
                }
            }

            impl<T> ConvertToC<T> for MutHandle<T> {
                fn to_c(self) -> T {
                    self.0
                }
            }

            impl<T> ConvertToC<Array<T>> for &[MutHandle<T>] {
                fn to_c(self) -> Array<T> {
                    // this is maybe a bit too unsafe.
                    // but the types we are dealing are transparent
                    // so it should be reliable
                    //
                    // TODO can this just be Array(...)?
                    unsafe { std::mem::transmute(self.as_ptr()) }
                }
            }

            impl<T> ConvertToC<T> for Option<MutHandle<T>> where T: Default {
                fn to_c(self) -> T {
                    match self {
                        Some(t) => t.to_c(),
                        None => T::default(),
                    }
                }
            }
        }

        #[derive(Debug)]
        #[repr(transparent)]
        pub struct OpaqueMutPtr(*mut c_void);

        impl ConvertToC<OpaqueMutPtr> for &mut [u8] {
            fn to_c(self) -> OpaqueMutPtr {
                OpaqueMutPtr(self.as_mut_ptr() as *mut c_void)
            }
        }

        impl ConvertToC<OpaqueMutPtr> for &mut Vec<u8> {
            fn to_c(self) -> OpaqueMutPtr {
                OpaqueMutPtr(self.as_mut_ptr() as *mut c_void)
            }
        }

        impl ConvertToC<OpaqueMutPtr> for Option<OpaqueMutPtr> {
            fn to_c(self) -> OpaqueMutPtr {
                match self {
                    Some(o) => o,
                    None => OpaqueMutPtr(::std::ptr::null_mut()),
                }
            }
        }

        // ============= Array<T> ===============
        // this is only intended to be used with *const and *mut
        // to indicate that the pointer is for an array of T

        #[derive(Debug)]
        #[repr(transparent)]
        pub struct Array<T>(*const T);

        impl<T> Copy for Array<T> {}

        impl<T> Clone for Array<T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        #[derive(Debug)]
        #[repr(transparent)]
        pub struct ArrayMut<T>(*mut T);

        impl<T> Copy for ArrayMut<T> {}

        impl<T> Clone for ArrayMut<T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        // Deref implementations
        // TODO consider if tese are even needed

        //impl<T> ::std::ops::Deref for Array<T> {
        //    type Target = T;
        //    fn deref(&self) -> &T {
        //        &self.0
        //    }
        //}

        //impl<T> ::std::ops::Deref for ArrayMut<T> {
        //    type Target = T;
        //    fn deref(&self) -> &T {
        //        &self.0
        //    }
        //}

        //impl<T> ::std::ops::DerefMut for ArrayMut<T> {
        //    fn deref_mut(&mut self) -> &mut T {
        //        &mut self.0
        //    }
        //}

        impl<T> ConvertToC<Array<T>> for &[T] {
            fn to_c(self) -> Array<T> {
                Array(self.as_ptr())
            }
        }

        impl<T> ConvertToC<Array<T>> for Option<&[T]> {
            fn to_c(self) -> Array<T> {
                Array(self.as_ptr())
            }
        }

        impl ConvertToC<Array<c_char>> for Option<MyStr<'_>> {
            fn to_c(self) -> Array<c_char> {
                Array(self.as_ptr())
            }
        }

        impl<T> ConvertToC<ArrayMut<T>> for &mut [T] {
            fn to_c(self) -> ArrayMut<T> {
                ArrayMut(self.as_mut_ptr())
            }
        }

        impl<T> ConvertToC<ArrayMut<T>> for Option<&mut [T]> {
            fn to_c(mut self) -> ArrayMut<T> {
                ArrayMut(self.as_mut_ptr())
            }
        }

        impl<T> ConvertToC<ArrayMut<T>> for &mut Vec<T> {
            fn to_c(self) -> ArrayMut<T> {
                ArrayMut(self.as_mut_ptr())
            }
        }

        impl ConvertToC<Array<c_char>> for &CStr {
            fn to_c(self) -> Array<c_char> {
                Array(self.as_ptr())
            }
        }

        impl ConvertToC<Array<c_char>> for MyStr<'_> {
            fn to_c(self) -> Array<c_char> {
                Array(self.0)
            }
        }

        // ============= Ref<T> ===============
        // this is only intended to be used with *const and *mut
        // to indicate that the pointer is for a single T

        #[derive(Debug)]
        #[repr(transparent)]
        pub struct Ref<T>(*const T);

        impl<T> Copy for Ref<T> {}

        impl<T> Clone for Ref<T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        #[derive(Debug)]
        #[repr(transparent)]
        pub struct RefMut<T>(*mut T);

        impl<T> Copy for RefMut<T> {}

        impl<T> Clone for RefMut<T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        // Deref implementations
        // TODO consider if tese are even needed

        //impl<T> ::std::ops::Deref for Ref<T> {
        //    type Target = T;
        //    fn deref(&self) -> &T {
        //        &self.0
        //    }
        //}

        //impl<T> ::std::ops::Deref for RefMut<T> {
        //    type Target = T;
        //    fn deref(&self) -> &T {
        //        &self.0
        //    }
        //}

        //impl<T> ::std::ops::DerefMut for RefMut<T> {
        //    fn deref_mut(&mut self) -> &mut T {
        //        &mut self.0
        //    }
        //}

        impl<T> ConvertToC<T> for T {
            fn to_c(self) -> T {
                self
            }
        }

        impl<T> ConvertToC<T> for Option<T> where T: Default {
            fn to_c(self) -> T {
                match self {
                    Some(t) => t,
                    None => T::default(),
                }
            }
        }

        impl<T> ConvertToC<Ref<T>> for &T {
            fn to_c(self) -> Ref<T> {
                Ref(self)
            }
        }

        impl<T> ConvertToC<Ref<T>> for Option<&T> {
            fn to_c(self) -> Ref<T> {
                Ref(self.as_ptr())
            }
        }

        impl<T> ConvertToC<RefMut<T>> for &mut T {
            fn to_c(self) -> RefMut<T> {
                RefMut(self)
            }
        }

        impl<T> ConvertToC<RefMut<T>> for Option<&mut T> {
            fn to_c(mut self) -> RefMut<T> {
                RefMut(self.as_mut_ptr())
            }
        }

        impl<T> ConvertToC<RefMut<T>> for &mut MaybeUninit<T> {
            fn to_c(self) -> RefMut<T> {
                RefMut(self.as_mut_ptr())
            }
        }

        // ================ types for complex c arrays =============
        // this is intended to be a type that can be targeted for easily generating
        // arrays to arrays that are compatible with multidimensional c arrays

        #[derive(Debug)]
        //#[repr(transparent)]
        pub struct ArrayArray<T>(Vec<T>);

        impl<T> ::std::ops::Deref for ArrayArray<T> {
            type Target = Vec<T>;
            fn deref(&self) -> &Vec<T> {
                &self.0
            }
        }

        impl<T> Default for ArrayArray<T> {
            fn default() -> Self {
                Self(Default::default())
            }
        }

        //impl<T> Into<Array<*const *const T>> for &ArrayArray<*const T> {
        //    fn into(&self) -> Array<*const *const T> {
        //        Array(self.0.as_ptr())
        //    }
        //}

        impl<T> ConvertToC<Array<*const T>> for &ArrayArray<*const T> {
            fn to_c(self) -> Array<*const T> {
                Array(self.0.as_ptr())
            }
        }

        impl ConvertToC<Array<*const c_char>> for &ArrayArray<MyStr<'_>> {
            fn to_c(self) -> Array<*const c_char> {
                Array( unsafe{::std::mem::transmute::<*const MyStr, *const *const c_char>(self.0.as_ptr())} )
            }
        }


        // ================ Return conversion type =============
        // this is for creating return variabls that can automatically convert into the types we
        // want

        trait Return<R> {
            fn ret(self) -> R;
        }

        impl<A, B> Return<A> for ((A), B) {
            fn ret(self) -> A {
                self.0
            }
        }

        // ================ Value trait =============
        // this trait lets us get the value from a plain value and from a reference in a consistent
        // way without explicit dereferencing

        trait Value : Copy {
            fn value(&self) -> Self {
                *self
            }
        }

        impl<T> Value for T where T: Copy {}

        macro_rules! vk_bitflags_wrapped {
            ($name: ident) => {

                impl Default for $name{
                    fn default() -> $name {
                        $name(0)
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
        // Some(&T) == *const T (non-null)
        // None == null ptr
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
        impl AsRawPtr<c_char> for Option<MyStr<'_>> {
            fn as_ptr(&self) -> *const c_char {
                // MyStr is a transparent &c_char. Thus, Option<MyStr> haa never zero optimization and is is safe to transmute to ptr
                unsafe { *(self as *const Option<MyStr> as *const Option<*const c_char> as *const *const c_char) }
            }
        }
        impl<'a, T> AsRawPtr<T> for Option<&'a [T]> {
            fn as_ptr(&self) -> *const T {
                //unsafe { ::std::mem::transmute::<Option<&T>, *const T>(*self) }
                self.map(|slice|slice.as_ptr()).unwrap_or(std::ptr::null())
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
            fn as_mut_ptr(&mut self) -> *mut T;
        }
        impl<'a, T> AsRawMutPtr<T> for Option<&'a mut T> {
            fn as_mut_ptr(&mut self) -> *mut T {
                //unsafe { ::std::mem::transmute::<Option<&mut T>, *mut T>(*self) }
                unsafe { *(self as *const Option<&mut T> as *const *mut T) }
            }
        }
        impl<'a, T> AsRawMutPtr<T> for Option<&'a mut [T]> {
            fn as_mut_ptr(&mut self) -> *mut T {
                //unsafe { ::std::mem::transmute::<Option<&T>, *const T>(*self) }
                self.as_mut().map(|slice|slice.as_mut_ptr()).unwrap_or(std::ptr::null_mut())
            }
        }
    };

    let feature_enums = registry2.0.iter().map(|item| {
        match item {
            vk_parse::RegistryChild::Feature(feature) => Some(extensions::generate_feature_enums_from_vk_parse_reg(feature)),
            _ => None,
        }
    });

    let mut q = quote!{
        #allow_vulkan_name_formats
        //#initial_test_code
        #util_code
        #platform_specific_types
        #(#tokens)*
        #(#feature_enums)*
        #(#aliases)*
    };

    let post_process_handles = post_process_handles(&parse_state);
    let post_process_enum_display_code = make_enumeration_display_code();

    q.extend(post_process_handles);
    q.extend(post_process_enum_display_code);

    q.extend(initial_test_code);

    //for node in parse_state.command_list.iter() {
    //    dbg!(&node.data().name);
    //    dbg!(commands::command_category(node.data()));
    //}

    println!("{}", q);

}


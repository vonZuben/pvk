#![recursion_limit = "1000"]

use quote::quote;

// just for coverting the xml file into a vkxml registry
extern crate vk_parse;

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
mod r#struct;
// mod methods;

use r#struct as stct;

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
            handle_commands(cmds)
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

pub fn generate(vk_xml_path: &str) -> String {
    let vk_xml_path = Path::new(vk_xml_path);
    // this it the easier to parse registry
    let registry = vk_parse::parse_file_as_vkxml(&vk_xml_path).expect("failed to parse and convert vk.xml");

    assert!(global_data::REGISTRY.set(registry).is_ok());

    let registry = global_data::REGISTRY.get().unwrap();

    // this registry is closer to the xml format, but it sucks to parse
    // but it does include the aliases
    let (registry2, _) = vk_parse::parse_file(&vk_xml_path).expect("failed to parse vk.xml");

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

    };

    let platform_specific_types = utils::platform_specific_types();

    //let result_members = global_data::result_members();

    // let result_ok: Vec<_> = result_members.iter().filter_map(|member| {
    //     if !member.is_err {
    //         let name = member.name.as_code();
    //         Some( quote!(#name) )
    //     }
    //     else {
    //         None
    //     }
    // }).collect();
    // let result_ok = result_ok.as_slice();

    // let result_err: Vec<_> = result_members.iter().filter_map(|member| {
    //     if member.is_err {
    //         let name = member.name.as_code();
    //         Some( quote!(#name) )
    //     }
    //     else {
    //         None
    //     }
    // }).collect();
    // let result_err = result_err.as_slice();

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

    let struct_names: Vec<_> = global_data::structure_types().iter().map(|x|x.name.as_code()).collect();
    let struct_st_names: Vec<_> = global_data::structure_types().iter().map(|x|x.st_name.as_code()).collect();

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
        pub struct InstanceCreator<'a> {
            app_name: CString,
            app_version: VkVersion,
            engine_name: CString,
            engine_version: VkVersion,
            // Vulkan spec: The patch version number specified in apiVersion is ignored when creating an instance object.
            // Only the major and minor versions of the instance must match those requested in apiVersion
            // therefore, I will only accept the custom Feature typs that I generate
            api_version: Option<Box<dyn Feature>>,
            enabled_layers: Vec<&'a dyn VkLayerName>,
            enabled_extensions: Vec<InstanceExtension>,
        }

        impl<'a> InstanceCreator<'a> {
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
                where E: Into<InstanceExtension> + 'a
            {
                self.enabled_extensions = enabled_extensions.into_iter()
                    .map( |e| e.into() )
                    .collect();
                self
            }

            pub fn create(&self) -> VkResult<InstanceOwner<'static, Owned>> {
                let app_name: MyStr = (&self.app_name).into();
                let engine_name: MyStr = (&self.engine_name).into();

                let api_version: Box<dyn Feature> = self.api_version
                    .as_ref()
                    .map(|boxed_ver| boxed_ver.clone())
                    .unwrap_or(Box::new(VERSION_1_0));

                // let app_info = crate::ApplicationInfo! {
                //     p_application_name: Some(app_name),
                //     application_version: self.app_version.make(),
                //     p_engine_name: Some(engine_name),
                //     engine_version: self.engine_version.make(),
                //     api_version: api_version.version(),
                // };

                // cannot use the exported macros internally due to
                // #[deny(macro-expanded-macro-exports-accessed-by-absolute-paths)]
                let app_info = ApplicationInfo {
                    s_type: StructureType::APPLICATION_INFO,
                    p_next: Pnext::new(),
                    p_application_name: app_name.to_c(),
                    application_version: self.app_version.make(),
                    p_engine_name: engine_name.to_c(),
                    engine_version: self.engine_version.make(),
                    api_version: api_version.version(),
                };

                let enabled_layers: ArrayArray<MyStr> = ArrayArray(self.enabled_layers.iter()
                                                .map( |layer| layer.layer_name().into() ).collect());
                let enabled_extensions: ArrayArray<MyStr> = ArrayArray(self.enabled_extensions.iter()
                                                .map( |extension| extension.0.extension_name().into() ).collect());

                // let create_info = crate::InstanceCreateInfo!{
                //     p_application_info: Some(&app_info),
                //     pp_enabled_layer_names: &enabled_layers,
                //     pp_enabled_extension_names: &enabled_extensions,
                // };

                let create_info = InstanceCreateInfo {
                    s_type: StructureType::INSTANCE_CREATE_INFO,
                    p_next: Pnext::new(),
                    flags: Default::default(),
                    p_application_info: (&app_info).to_c(),
                    enabled_layer_count: enabled_layers.len() as _,
                    pp_enabled_layer_names: (&enabled_layers).to_c(),
                    enabled_extension_count: enabled_extensions.len() as _,
                    pp_enabled_extension_names: (&enabled_extensions).to_c(),
                };

                let mut inst: MaybeUninit<Instance> = MaybeUninit::uninit();

                let vk_result = unsafe {
                    CreateInstance::call()(
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

                    let mut instance: InstanceOwner<'static, Owned> = InstanceOwner::new(inst, &());

                    // loading commands when creating an instance is guaranteed safe since it
                    // is impossible to be aliased
                    unsafe { api_version.load_instance_commands(instance.handle, &instance.commands); }
                    for extension in &self.enabled_extensions {
                        unsafe { extension.0.load_instance_commands(instance.handle, &instance.commands); }
                    }

                    instance.feature_version = api_version;

                    vk_result.success(instance)
                }
            }
        }

        pub struct DeviceCreator<'public, 'private> {
            physical_device: &'private PhysicalDeviceOwner<'public>,
            queue_create_info: &'private [DeviceQueueCreateInfo<'public, 'private>],
            enabled_layers: Vec<&'private dyn VkLayerName>,
            enabled_extensions: Vec<DeviceExtension>,
            enabled_features: Option<&'private PhysicalDeviceFeatures>,
        }

        impl<'public, 'private> DeviceCreator<'public, 'private> {
            pub fn enabled_layers<L>(mut self, enabled_layers: impl IntoIterator<Item = L>) -> Self
                where L: Into<LayerWrapper<'private>>
            {
                self.enabled_layers = enabled_layers.into_iter()
                    .map( |l| l.into().0 )
                    .collect();
                self
            }

            pub fn enabled_extensions<E>(mut self, enabled_extensions: impl IntoIterator<Item = E>) -> Self
                where E: Into<DeviceExtension> + 'private
            {
                self.enabled_extensions = enabled_extensions.into_iter()
                    .map( |e| e.into() )
                    .collect();
                self
            }
            pub fn enabled_features(mut self, enabled_features: &'private PhysicalDeviceFeatures) -> Self {
                self.enabled_features = enabled_features.into();
                self
            }
            pub unsafe fn create(&self) -> VkResult<DeviceOwner<'public, Owned>> {

                let enabled_layers: ArrayArray<MyStr> = ArrayArray(self.enabled_layers.iter()
                                                .map( |layer| layer.layer_name().into() ).collect());
                let enabled_extensions: ArrayArray<MyStr> = ArrayArray(self.enabled_extensions.iter()
                                                .map( |extension| extension.0.extension_name().into() ).collect());

                // let create_info = crate::DeviceCreateInfo!{
                //     flags: DeviceCreateFlags::empty(),
                //     p_queue_create_infos: self.queue_create_info,
                //     pp_enabled_layer_names: &enabled_layers,
                //     pp_enabled_extension_names: &enabled_extensions,
                //     p_enabled_features: self.enabled_features,
                // };

                let create_info = DeviceCreateInfo {
                    s_type: StructureType::DEVICE_CREATE_INFO,
                    p_next: Pnext::new(),
                    flags: Default::default(),
                    queue_create_info_count: self.queue_create_info.len() as _,
                    p_queue_create_infos: self.queue_create_info.to_c(),
                    enabled_layer_count: enabled_layers.len() as _,
                    pp_enabled_layer_names: (&enabled_layers).to_c(),
                    enabled_extension_count: enabled_extensions.len() as _,
                    pp_enabled_extension_names: (&enabled_extensions).to_c(),
                    p_enabled_features: self.enabled_features.to_c(),
                };

                let mut device: MaybeUninit<Device> = MaybeUninit::uninit();

                let vk_result = unsafe {
                    self.physical_device.dispatch_parent.commands
                        .CreateDevice.call()(
                            self.physical_device.handle(),
                            (&create_info).to_c(),
                            None.to_c(),
                            (&mut device).to_c(),
                            )
                };

                if vk_result.is_err() {
                    vk_result.err()
                }
                else {
                    let device = unsafe { device.assume_init() };

                    let device: DeviceOwner<'public, Owned> = DeviceOwner::new(device, self.physical_device.dispatch_parent);

                    self.physical_device.dispatch_parent
                        .feature_version
                        .load_device_commands(device.handle, &device.commands);
                    for extension in &self.enabled_extensions {
                        extension.0.load_device_commands(device.handle, &device.commands);
                        extension.0.load_instance_commands(self.physical_device.dispatch_parent.handle,
                                                         &self.physical_device.dispatch_parent.commands);
                    }

                    vk_result.success(device)
                }

            }
        }

        impl<'public> PhysicalDeviceOwner<'public> {
            pub fn device_creator<'private>(&'private self, queue_create_info: &'private [DeviceQueueCreateInfo<'public, 'private>]) -> DeviceCreator<'public, 'private> {
                DeviceCreator {
                    physical_device: self,
                    queue_create_info: queue_create_info,
                    enabled_layers: Default::default(),
                    enabled_extensions: Default::default(),
                    enabled_features: Default::default(),
                }
            }
        }

        pub struct VkResult<T> {
            val: MaybeUninit<T>,
            result_code: VkResultRaw,
        }

        impl<T> VkResult<T> {
            pub fn unwrap(self) -> T {
                self.to_std_result().unwrap()
            }
            pub fn to_std_result(self) -> Result<T, VkResultRaw> {
                if self.result_code.is_err() {
                    Err(self.result_code)
                }
                else {
                    unsafe { Ok(self.val.assume_init()) }
                }
            }
            pub fn result_code(&self) -> VkResultRaw {
                self.result_code
            }
        }

        impl<T> Debug for VkResult<T> where T: Debug {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                // use to get Display impl of VkResultRaw inside Debug context
                struct VkErr(VkResultRaw);
                impl Debug for VkErr {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        write!(f, "{}", self.0)
                    }
                }
                if self.result_code.is_err() {
                    f.debug_struct("VkResult")
                        .field("code", &VkErr(self.result_code))
                        .field("value", &())
                        .finish()
                }
                else {
                    f.debug_struct("VkResult")
                        .field("code", &VkErr(self.result_code))
                        .field("value", unsafe { &*self.val.as_ptr() })
                        .finish()
                }
            }
        }

        // this is made so it can be called in the same way as VkResultRaw to make code gen easier
        trait FakeResult {
            fn success<T>(self, t: T) -> Self;
            fn err<T>(self) -> !;
            fn is_err(self) -> bool;
        }

        impl<T> FakeResult for T {
            fn success<F>(self, t: F) -> Self {
                self
            }
            fn err<F>(self) -> ! {
                panic!("shouldn't call err() with FakeResult");
            }
            fn is_err(self) -> bool {
                false
            }
        }

        impl crate::VkResultRaw {
            // takes a possible value to wrap
            fn success<T>(self, t: T) -> VkResult<T> {
                assert!(!self.is_err());
                VkResult {
                    val: MaybeUninit::new(t),
                    result_code: self,
                }
            }
            // if err then we don't want to return anything
            fn err<T>(self) -> VkResult<T> {
                assert!(self.is_err());
                VkResult {
                    val: MaybeUninit::uninit(),
                    result_code: self,
                }
            }
            fn is_err(self) -> bool {
                self.0 < 0
            }
        }

        #[derive(Debug, Default)]
        pub struct Owned;

        #[derive(Debug, Default)]
        pub struct ManuallyManaged;

        #[derive(Debug, Default)]
        pub struct Borrowed;

        pub struct FeatureWrapper(Box<dyn Feature>);

        trait Feature : FeatureCore + Send + Sync + 'static {}
        impl<T> Feature for T where T: FeatureCore + Send + Sync + 'static {}

        trait FeatureCore {
            unsafe fn load_instance_commands(&self, instance: Instance, inst_cmds: &InstanceCommands);
            unsafe fn load_device_commands(&self, device: Device, dev_cmds: &DeviceCommands);
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

        trait InstanceExtensionMarker: VkExtensionLoader + Debug {}
        type InsEx = &'static dyn InstanceExtensionMarker;

        trait DeviceExtensionMarker: VkExtensionLoader + Debug {}
        type DevEx = &'static dyn DeviceExtensionMarker;

        trait VkExtension {
            fn extension_name(&self) -> &CStr;
        }

        trait VkExtensionLoader : VkExtension {
            unsafe fn load_instance_commands(&self, instance: Instance, commands: &InstanceCommands) {
                noop!();
            }
            unsafe fn load_device_commands(&self, device: Device, commands: &DeviceCommands) {
                noop!();
            }
        }

        enum Ext {
            Instance(InsEx),
            Device(DevEx),
        }

        impl From<InsEx> for Ext {
            fn from(ex: InsEx) -> Self {
                Ext::Instance(ex)
            }
        }

        impl From<DevEx> for Ext {
            fn from(ex: DevEx) -> Self {
                Ext::Device(ex)
            }
        }

        impl From<Ext> for InsEx {
            fn from(ex: Ext) -> Self {
                match ex {
                    Ext::Instance(ex) => ex,
                    Ext::Device(ex) => panic!("{:?} is not an Instance Extension", ex.extension_name()),
                }
            }
        }

        impl From<Ext> for DevEx {
            fn from(ex: Ext) -> Self {
                match ex {
                    Ext::Device(ex) => ex,
                    Ext::Instance(ex) => panic!("{:?} is not an Device Extension", ex.extension_name()),
                }
            }
        }

        #[derive(Clone, Copy)]
        pub struct InstanceExtension(InsEx);

        impl<E> From<E> for InstanceExtension where E: AsRef<str> {
            fn from(e: E) -> Self {
                Self(str_to_extension_loader(e.as_ref()).into())
            }
        }

        impl From<&InstanceExtension> for InstanceExtension {
            fn from(e: &InstanceExtension) -> Self {
                *e
            }
        }

        #[derive(Clone, Copy)]
        pub struct DeviceExtension(DevEx);

        impl<E> From<E> for DeviceExtension where E: AsRef<str> {
            fn from(e: E) -> Self {
                Self(str_to_extension_loader(e.as_ref()).into())
            }
        }

        impl From<&DeviceExtension> for DeviceExtension {
            fn from(e: &DeviceExtension) -> Self {
                *e
            }
        }

        impl AsRef<str> for ExtensionProperties {
            fn as_ref(&self) -> &str {
                // self.extention_name should always be a valid c string
                // unless the vulkan implementation (driver) is wrong
                // do I need to guard against bad drivers?
                // hmmm?
                // or it is fine since this will only be used internally to
                // pass the string straigth back to the vulkan driver via
                // extension loading
                unsafe { CStr::from_ptr(self.extension_name.as_ptr()).to_str().unwrap() }
            }
        }

        //fn ex_name_to_extension_loader(e: &impl VkExtension) -> &'static dyn VkExtensionLoader {
        fn str_to_extension_loader(e: &str) -> Ext {
            match e.as_bytes() {
                #( #ext_c_names => #ext_loader_names.as_trait_obj().into(), )*
                _ => panic!("unrecognized extension name {:?}. Possibly unsupported by current version of vk.rs?",
                        e
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

        impl VkLayerName for LayerProperties {
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

        impl<'a> From<&'a LayerProperties> for LayerWrapper<'a> {
            fn from(l: &'a LayerProperties) -> Self {
                Self(l)
            }
        }

        mod private_struct_interface {
            use super::BaseStructure;
            pub trait StypeInit<'public>: Sized + super::Base<'public> {
                fn init_s_type() -> Self {
                    let mut this = std::mem::MaybeUninit::zeroed();
                    {
                        let ptr = this.as_mut_ptr() as *mut BaseStructure;
                        unsafe { (*ptr).s_type = <Self as super::Base>::ST; }
                    }
                    unsafe { this.assume_init() }
                }
            }
            pub trait AddChain : Sized + super::Base<'static> {
                fn add_chain<C: PnChain<Self>>(&mut self, c: &mut C) {
                    let ptr = self as *mut Self as *mut BaseStructure;
                    unsafe {
                        (*ptr).p_next = c.head();
                    }
                }
            }
            pub trait PnLink<Extendee> : super::Base<'static> {
                fn link<T: PnLink<Extendee>>(&mut self, l: &mut T) {
                    let ptr = self as *mut Self as *mut BaseStructure;
                    unsafe {
                        (*ptr).p_next = l as *mut T as *mut BaseStructure;
                    }
                }
            }
            pub trait PnChain<Extendee> {
                fn new_chain() -> Self;
                fn link_chain(&mut self);
                fn head(&mut self) -> *mut BaseStructure<'static>;
            }
        }

        macro_rules! tuple_impl {
            ( @link_list $this:ident $id1:tt, ) => {};
            ( @link_list $this:ident $id1:tt, $id2:tt, $($rest:tt)* ) => {
                $this.$id1.link(&mut $this.$id2);
                tuple_impl!( @link_list $this $id2, $($rest)* )
            };
            ( @dbg $this:ident ( ) ; $($dbl:tt)* ) => {
                $($dbl)* .finish()
            };
            ( @dbg $this:ident ( $id:tt, $($rest:tt)* ) ; $($dbl:tt)* ) => {
                tuple_impl!( @dbg $this ( $($rest)* ) ; $($dbl)* .entry(&$this.$id) )
            };
            ( $($id:tt, $t:ident);* ) => {
                impl<$($t),* , Extendee> PnChain<Extendee> for ($($t,)*)
                where
                    $($t: PnLink<Extendee> + StypeInit<'static>),*
                {
                    fn new_chain() -> Self {
                        ($($t::init_s_type(),)*)
                    }
                    fn link_chain(&mut self) {
                        tuple_impl!( @link_list self $($id,)* );
                    }
                    fn head(&mut self) -> *mut BaseStructure<'static> {
                        &mut self.0 as *mut T0 as *mut BaseStructure
                    }
                }

                impl<$($t: fmt::Debug),*, Extendee: fmt::Debug> fmt::Debug for PnTuple<Extendee, ($($t,)*)>
                {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        self.0.fmt(f)?;
                        let this = &self.1;
                        tuple_impl!(@dbg this ( $($id,)* ) ; f.debug_list() )
                    }
                }
            }
        }

        macro_rules! tuple_combos {
            ( @inner ( $($list:tt)* ) => ( ) ) => {
                tuple_impl!($($list)*);
            };
            ( @inner ( $($list:tt)* ) => ( $id:tt, $t:ident $( ; $($rest:tt)* )? ) ) => {
                tuple_impl!($($list)*);
                tuple_combos!( @inner ( $($list)* ; $id, $t ) => ( $( $($rest)* )? ) );
            };
            ( $first_id:tt, $first_t:ident; $($id:tt, $t:ident);* ) => {
                tuple_combos!( @inner ( $first_id, $first_t ) => ( $($id, $t);* ) );
            }

        }

        impl<Extendee> PnChain<Extendee> for () {
            fn new_chain() -> () {
                ()
            }
            fn link_chain(&mut self) {}
            fn head(&mut self) -> *mut BaseStructure<'static> {
                std::ptr::null_mut()
            }
        }

        // there are (at this time) at least 27 extensions to VkPhysicalDeviceProperties2
        // so we should be able to support that many in the PnChain tuple
        tuple_combos! {
            0, T0;
            1, T1;
            2, T2;
            3, T3;
            4, T4;
            5, T5;
            6, T6;
            7, T7;
            8, T8;
            9, T9;
            10, T10;
            11, T11;
            12, T12;
            13, T13;
            14, T14;
            15, T15;
            16, T16;
            17, T17;
            18, T18;
            19, T19;
            20, T20;
            21, T21;
            22, T22;
            23, T23;
            24, T24;
            25, T25;
            26, T26;
            27, T27;
            28, T28;
            29, T29;
            30, T30
        }

        pub struct PnTuple<A, C>(A, C);

        impl<A: StypeInit<'static> + AddChain, C: PnChain<A>> PnTuple<A, C> {
            fn new() -> Self {
                Self(A::init_s_type(), C::new_chain())
            }
            #[allow(unused)]
            fn from_parts(a: A, c: C) -> Self {
                Self(a, c)
            }
            fn link_list(&mut self) {
                self.1.link_chain();
                self.0.add_chain(&mut self.1);
            }
            pub fn pn(&self) -> &C {
                &self.1
            }
        }

        impl<A, C> std::ops::Deref for PnTuple<A, C> {
            type Target = A;
            fn deref(&self) -> &A {
                &self.0
            }
        }

        impl<Extendee: fmt::Debug> fmt::Debug for PnTuple<Extendee, ()> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<'a, A, C> ConvertToC<RefMut<'a, A>> for &'a mut PnTuple<A, C> where A: 'a {
            fn to_c(self) -> RefMut<'a, A> {
                RefMut::new(&mut self.0)
            }
        }

        #[repr(C)]
        pub struct BaseStructure<'public> {
            s_type: StructureType,
            p_next: *mut BaseStructure<'public>,
        }

        impl<'public> BaseStructure<'public> {
            fn raw_s_type(&self) -> i32 {
                self.s_type.0
            }
            unsafe fn cast_ref_unchecked<T: Base<'public>>(&self) -> &T {
                &*(self as *const Self as *const T)
            }
            pub fn cast_ref<T: Base<'public>>(&self) -> Option<&T> {
                if self.s_type == T::ST {
                    unsafe {
                        Some( self.cast_ref_unchecked() )
                    }
                }
                else {
                    None
                }
            }
            pub fn cast_ref_enum<'private>(&'private self) -> Stype<'public, 'private> {
                match self.s_type {
                    #( StructureType::#struct_st_names => Stype::#struct_names(unsafe{ self.cast_ref_unchecked() }), )*
                    _=> Stype::Unknown(self),
                }
            }
        }

        impl fmt::Debug for BaseStructure<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self.cast_ref_enum() {
                    #( Stype::#struct_names(s) => s.fmt(f), )*
                    Stype::Unknown(s) => write!(f, "Unknown Structure type {}", s.s_type.0),
                }
            }
        }

        pub enum Stype<'public, 'private> {
            #( #struct_names(&'private #struct_names<'public, 'private>), )*
            Unknown(&'private BaseStructure<'public>),
        }

        // only impl for things that can be intepreted with BaseStructure
        pub unsafe trait Base<'public> {
            const ST: StructureType;
        }

        // node in p_next chain
        #[repr(transparent)]
        #[derive(Clone, Copy)]
        struct Pnext<'public, 'private> {
            base: *mut BaseStructure<'public>,
            _ext: PhantomData<&'private mut BaseStructure<'public>>,
        }

        // pub struct PnextIter<'public, 'private> {
        //     base: *const BaseStructure<'public>,
        //     _pn: PhantomData<&'private Pnext<'public, 'private>>,
        // }

        // impl<'public, 'pn> Iterator for PnextIter<'public, 'pn> {
        //     type Item = &'pn BaseStructure<'public>;
        //     fn next(&mut self) -> Option<Self::Item> {
        //         if self.base.is_null() {
        //             None
        //         }
        //         else {
        //             unsafe {
        //                 let ret = &*self.base;
        //                 self.base = ret.p_next;
        //                 Some(ret)
        //             }
        //         }
        //     }
        // }

        impl Pnext<'_, '_> {
            fn new() -> Self {
                Self {
                    base: ptr::null_mut(),
                    _ext: PhantomData,
                }
            }
        }

        impl<'public, 'private> Pnext<'public, 'private> {
            fn push<E: Base<'public>>(&mut self, e: &'private mut E) {
                let base = e as *mut E as *mut BaseStructure;
                unsafe { (*base).p_next = self.base };
                self.base = base;
            }

            // fn iter<'pn>(&'pn self) -> PnextIter<'public, 'pn> {
            //     PnextIter {
            //         base: self.base,
            //         _pn: PhantomData,
            //     }
            // }

            // fn base(&self) -> Option<&BaseStructure<'public>> {
            //     if self.base.is_null() {
            //         None
            //     }
            //     else {
            //         Some( unsafe { &*self.base } )
            //     }
            // }
        }

        // impl fmt::Debug for Pnext<'_, '_> {
        //     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //         if let Some(base) = self.base() {
        //             if f.alternate() {
        //                 writeln!(f)?;
        //             }
        //             write!(f, "::pNext -> ")?;
        //             base.fmt(f)
        //         }
        //         else {
        //             Ok(())
        //         }
        //     }
        // }

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

        impl<T> Len for Option<&mut [T]> {
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
        pub struct ArrayString<A: AsRef<[c_char]>>(A);

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

            pub trait Handle: Copy {}

            pub trait CreateOwner<'parent> {
                type Handle: Handle + 'static;
                type DispatchParent: 'parent;
                fn new(handle: Self::Handle, dispatch_parent: &'parent Self::DispatchParent) -> Self;
                fn disassemble(self) -> (Self::Handle, &'parent Self::DispatchParent);
            }

            pub trait HandleOwner<'owner> {
                type Handle: Handle + 'owner;
                fn handle(&'owner self) -> Self::Handle;
                fn mut_handle(&'owner mut self) -> MutHandle<Self::Handle> {
                    self.into()
                }
            }

            pub struct TempOwner<'parent, O> {
                owner: ManuallyDrop<O>,
                _p: PhantomData<&'parent ()>,
            }

            impl<O> TempOwner<'_, O> {
                fn new(o: O) -> Self {
                    Self {
                        owner: ManuallyDrop::new(o),
                        _p: PhantomData,
                    }
                }
            }

            impl<O> ::std::ops::Deref for TempOwner<'_, O> {
                type Target = O;
                fn deref(&self) -> &Self::Target {
                    &self.owner
                }
            }

            impl<O: ::std::fmt::Debug> ::std::fmt::Debug for TempOwner<'_, O> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    (*self.owner).fmt(f)
                }
            }

            pub struct TempOwnerMut<'parent, O> {
                owner: ManuallyDrop<O>,
                _p: PhantomData<&'parent ()>,
            }

            impl<O> TempOwnerMut<'_, O> {
                fn new(o: O) -> Self {
                    Self {
                        owner: ManuallyDrop::new(o),
                        _p: PhantomData,
                    }
                }
            }

            impl<O> ::std::ops::Deref for TempOwnerMut<'_, O> {
                type Target = O;
                fn deref(&self) -> &Self::Target {
                    &self.owner
                }
            }

            impl<O> ::std::ops::DerefMut for TempOwnerMut<'_, O> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.owner
                }
            }

            impl<O: ::std::fmt::Debug> ::std::fmt::Debug for TempOwnerMut<'_, O> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    (*self.owner).fmt(f)
                }
            }

            pub struct Handles<'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> {
                dispatch_parent: &'parent O::DispatchParent,
                handles: A,
                destroy_handles: bool,
                _owners: PhantomData<[O]>,
            }

            pub type HandleVec<'parent, O> = Handles<'parent, O, Vec<<O as CreateOwner<'parent>>::Handle>>;

            impl<'parent, O: CreateOwner<'parent> + ::std::fmt::Debug, A: AsRef<[O::Handle]>> ::std::fmt::Debug for Handles<'parent, O, A> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    f.debug_list().entries(self).finish()
                }
            }

            pub struct HandleIter<'h, 'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> {
                index: usize,
                len: usize,
                handles: &'h Handles<'parent, O, A>,
            }

            pub struct HandleIterMut<'h, 'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> {
                index: usize,
                len: usize,
                handles: &'h mut Handles<'parent, O, A>,
            }

            pub struct HandleDrain<'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> {
                index: usize,
                len: usize,
                handles: Handles<'parent, O, A>,
            }

            impl<'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> Handles<'parent, O, A> {
                fn new(dispatch_parent: &'parent O::DispatchParent, handles: A) -> Self {
                    Self {
                        dispatch_parent,
                        handles,
                        destroy_handles: true,
                        _owners: PhantomData,
                    }
                }
                pub fn len(&self) -> usize {
                    self.handles.as_ref().len()
                }

                pub fn index<'h>(&'h self, i: usize) -> TempOwner<'h, O> {
                    let handles = self.handles.as_ref();
                    TempOwner::new(O::new(handles[i].value(), self.dispatch_parent))
                }

                pub fn index_mut<'h>(&'h mut self, i: usize) -> TempOwnerMut<'h, O> {
                    let handles = self.handles.as_ref();
                    TempOwnerMut::new(O::new(handles[i].value(), self.dispatch_parent))
                }

                pub fn iter<'h>(&'h self) -> HandleIter<'h, 'parent, O, A> {
                    HandleIter {
                        index: 0,
                        len: self.handles.as_ref().len(),
                        handles: self,
                    }
                }

                pub fn iter_mut<'h>(&'h mut self) -> HandleIterMut<'h, 'parent, O, A> {
                    HandleIterMut {
                        index: 0,
                        len: self.handles.as_ref().len(),
                        handles: self,
                    }
                }

                pub fn drain(mut self) -> HandleDrain<'parent, O, A> {
                    self.destroy_handles = false;
                    HandleDrain {
                        index: 0,
                        len: self.handles.as_ref().len(),
                        handles: self,
                    }
                }
            }

            // impl<'parent, O: CreateOwner<'parent>>, F: IntoIterator<Item=O>> From<F> for HandleVec<'parent, O> {
            //     fn from(f: F) -> Self {
            //         let f= f.into_iter();
            //         let handles = Vec::with_capcity(f.size_hint());

            //         for o in f {
            //             let (handle, dispatch_owner) = o.disassemble();

            //         }
            //     }
            // }

            // impl<'parent, O: CreateOwner<'parent> + HandleOwner<'static, O::Handle>> HandleVec<'parent, O>
            // where
            //     O::Handle: Handle,
            // {
            //     // unsafe since user needs to ensure that all owsners have the same dispatch parent
            //     unsafe fn from_slice(o: impl AsRef<[O]> + 'static) -> Self {

            //         let owners = o.as_ref();

            //         let dispatch_parent = owners[0].dispatch_parent();

            //         let handles: Vec<O::Handle> = owners.iter().map(|o| o.handle()).collect();

            //         Handles {
            //             dispatch_parent,
            //             handles,
            //             _owners: PhantomData,
            //         }
            //     }
            // }

            impl<'h, 'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> Iterator for HandleIter<'h, 'parent, O, A> {
                type Item = TempOwner<'h, O>;
                fn next(&mut self) -> Option<Self::Item>{
                    if self.index == self.len {
                        None
                    }
                    else {
                        let handle = unsafe {self.handles.handles.as_ref().get_unchecked(self.index)};
                        self.index += 1;
                        Some(
                            TempOwner::new(O::new(handle.value(), self.handles.dispatch_parent))
                        )
                    }
                }
            }

            impl<'h, 'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> Iterator for HandleIterMut<'h, 'parent, O, A> {
                type Item = TempOwnerMut<'h, O>;
                fn next(&mut self) -> Option<Self::Item>{
                    if self.index == self.len {
                        None
                    }
                    else {
                        let handle = unsafe {self.handles.handles.as_ref().get_unchecked(self.index)};
                        self.index += 1;
                        Some(
                            TempOwnerMut::new(O::new(handle.value(), self.handles.dispatch_parent))
                        )
                    }
                }
            }

            impl<'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> Iterator for HandleDrain<'parent, O, A> {
                type Item = O;
                fn next(&mut self) -> Option<Self::Item>{
                    if self.index == self.len {
                        None
                    }
                    else {
                        let handle = unsafe {self.handles.handles.as_ref().get_unchecked(self.index)};
                        self.index += 1;
                        Some(
                            O::new(handle.value(), self.handles.dispatch_parent)
                        )
                    }
                }
            }

            impl<'h, 'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> IntoIterator for &'h Handles<'parent, O, A> {
                type Item = TempOwner<'h, O>;
                type IntoIter = HandleIter<'h, 'parent, O, A>;
                fn into_iter(self) -> Self::IntoIter {
                    self.iter()
                }
            }

            impl<'h, 'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> IntoIterator for &'h mut Handles<'parent, O, A> {
                type Item = TempOwnerMut<'h, O>;
                type IntoIter = HandleIterMut<'h, 'parent, O, A>;
                fn into_iter(self) -> Self::IntoIter {
                    self.iter_mut()
                }
            }

            impl<'h, 'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> IntoIterator for Handles<'parent, O, A> {
                type Item = O;
                type IntoIter = HandleDrain<'parent, O, A>;
                fn into_iter(self) -> Self::IntoIter {
                    self.drain()
                }
            }

            impl<'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> Drop for Handles<'parent, O, A> {
                fn drop(&mut self) {
                    if std::mem::needs_drop::<O>() && self.destroy_handles {
                        for handle in self.handles.as_ref().iter().copied() {
                            O::new(handle, self.dispatch_parent); // create the owner which will be imediatly dropped
                        }
                    }
                }
            }

            impl<'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> Drop for HandleDrain<'parent, O, A> {
                fn drop(&mut self) {
                    if std::mem::needs_drop::<O>() {
                        let remaining_handles = &self.handles.handles.as_ref()[self.index..]; // only drop handles which havn't been retured in the iterator
                        for handle in remaining_handles.iter().copied() {
                            O::new(handle, self.handles.dispatch_parent); // create the owner which will be imediatly dropped
                        }
                    }
                }
            }

            impl<'parent, 'h, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> ConvertToC<Array<'h, O::Handle>> for Handles<'parent, O, A> where Self: 'h {
                fn to_c(self) -> Array<'h, O::Handle> {
                    unsafe { Array::from_ptr(self.handles.as_ref().as_ptr()) }
                }
            }

            impl<'parent, O: CreateOwner<'parent>, A: AsRef<[O::Handle]>> Return<Handles<'parent, O, A>>
                for ((A), &'parent O::DispatchParent)
            {
                fn ret(self) -> Handles<'parent, O, A> {
                    Handles::new(self.1, self.0)
                }
            }

            // ============= MutHandle<T> ================
            // Can only be created by mutably borrowing a HandleOwner
            #[derive(Debug, Clone, Copy)]
            #[repr(transparent)]
            pub struct MutHandle<H>(H);

            impl<'owner, H: Handle + 'owner> MutHandle<H> {
                pub fn new<O>(o: &'owner mut O) -> Self
                where
                    O: HandleOwner<'owner, Handle = H> + ?Sized,
                {
                    Self(o.handle())
                }
            }

            impl<'owner, H: Handle + 'owner, O> From<&'owner mut O> for MutHandle<H>
                where
                    O: HandleOwner<'owner, Handle = H> + ?Sized,
            {
                fn from(o: &'owner mut O) -> Self {
                    Self::new(o)
                }
            }

            impl<H> ConvertToC<H> for MutHandle<H> {
                fn to_c(self) -> H {
                    self.0
                }
            }

            impl<'a, H> ConvertToC<Array<'a, H>> for &'a [MutHandle<H>] {
                fn to_c(self) -> Array<'a, H> {
                    // this is maybe a bit too unsafe.
                    // but the types we are dealing are transparent
                    // so it should be reliable
                    //
                    // TODO can this just be Array(...)?
                    unsafe { Array::from_ptr(self.as_ptr() as *const H) }
                }
            }

            impl<H> ConvertToC<H> for Option<MutHandle<H>> where H: Default {
                fn to_c(self) -> H {
                    match self {
                        Some(t) => t.to_c(),
                        None => H::default(),
                    }
                }
            }
        }

        #[derive(Debug)]
        #[repr(transparent)]
        pub struct OpaqueMutPtr<'a>(*mut c_void, PhantomData<&'a mut ()>);

        impl<'a> OpaqueMutPtr<'a> {
            fn new<T>(p: &'a mut T) -> Self {
                Self(p as *mut T as *mut c_void, PhantomData)
            }
            unsafe fn from_ptr<T>(p: *mut T) -> Self {
                Self(p as *mut c_void, PhantomData)
        }
        }

        impl<'a> ConvertToC<OpaqueMutPtr<'a>> for &'a mut [u8] {
            fn to_c(self) -> OpaqueMutPtr<'a> {
                unsafe { OpaqueMutPtr::from_ptr(self.as_mut_ptr()) }
            }
        }

        impl<'a> ConvertToC<OpaqueMutPtr<'a>> for &'a mut Vec<u8> {
            fn to_c(self) -> OpaqueMutPtr<'a> {
                unsafe { OpaqueMutPtr::from_ptr(self.as_mut_ptr()) }
            }
        }

        impl<'a> ConvertToC<OpaqueMutPtr<'a>> for Option<OpaqueMutPtr<'a>> {
            fn to_c(self) -> OpaqueMutPtr<'a> {
                match self {
                    Some(o) => o,
                    None => unsafe { OpaqueMutPtr::from_ptr(::std::ptr::null_mut::<()>()) },
                }
            }
        }

        // ============= Array<T> ===============
        // this is only intended to be used with *const and *mut
        // to indicate that the pointer is for an array of T

        #[repr(transparent)]
        pub struct Array<'a, T>(*const T, PhantomData<&'a [T]>);

        impl<'a, T> Array<'a, T> {
            fn from_ptr(p: *const T) -> Self {
                Self(p, PhantomData)
            }
        }

        impl<T> Copy for Array<'_, T> {}

        impl<T> Clone for Array<'_, T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<T: Debug> Debug for Array<'_, T> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.debug_struct("Array")
                    .field("ptr", &self.0)
                    // .field("val", unsafe{ &*self.0 })
                    .finish()
            }
        }

        #[repr(transparent)]
        pub struct ArrayMut<'a, T>(*mut T, PhantomData<&'a mut [T]>);

        impl<'a, T> ArrayMut<'a, T> {
            fn from_ptr(p: *mut T) -> Self {
                Self(p, PhantomData)
            }
        }

        impl<T> Copy for ArrayMut<'_, T> {}

        impl<T> Clone for ArrayMut<'_, T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<T: Debug> Debug for ArrayMut<'_, T> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.debug_struct("ArrayMut")
                    .field("ptr", &self.0)
                    // .field("val", unsafe{ &*self.0 })
                    .finish()
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

        impl<'a, T> ConvertToC<Array<'a, T>> for &'a [T] {
            fn to_c(self) -> Array<'a, T> {
                unsafe { Array::from_ptr(self.as_ptr()) }
            }
        }

        impl<'a, T> ConvertToC<Array<'a, T>> for Option<&'a [T]> {
            fn to_c(self) -> Array<'a, T> {
                unsafe { Array::from_ptr(self.as_ptr()) }
            }
        }

        impl<'a> ConvertToC<Array<'a, c_char>> for Option<MyStr<'a>> {
            fn to_c(self) -> Array<'a, c_char> {
                unsafe { Array::from_ptr(self.as_ptr()) }
            }
        }

        impl<'a, T> ConvertToC<ArrayMut<'a, T>> for &'a mut [T] {
            fn to_c(self) -> ArrayMut<'a, T> {
                unsafe { ArrayMut::from_ptr(self.as_mut_ptr()) }
            }
        }

        impl<'a, T> ConvertToC<ArrayMut<'a, T>> for Option<&'a mut [T]> {
            fn to_c(mut self) -> ArrayMut<'a, T> {
                unsafe { ArrayMut::from_ptr(self.as_mut_ptr()) }
            }
        }

        impl<'a, T> ConvertToC<ArrayMut<'a, T>> for &'a mut Vec<T> {
            fn to_c(self) -> ArrayMut<'a, T> {
                unsafe { ArrayMut::from_ptr(self.as_mut_ptr()) }
            }
        }

        impl<'a> ConvertToC<Array<'a, c_char>> for &'a CStr {
            fn to_c(self) -> Array<'a, c_char> {
                unsafe { Array::from_ptr(self.as_ptr()) }
            }
        }

        impl<'a> ConvertToC<Array<'a, c_char>> for MyStr<'a> {
            fn to_c(self) -> Array<'a, c_char> {
                unsafe { Array::from_ptr(self.0) }
            }
        }

        // ============= Ref<T> ===============
        // this is only intended to be used with *const and *mut
        // to indicate that the pointer is for a single T

        // #[derive(Debug)]
        #[repr(transparent)]
        pub struct Ref<'a, T>(*const T, PhantomData<&'a T>);

        impl<'a, T> Ref<'a, T> {
            fn new(r: &'a T) -> Self {
                Self(r, PhantomData)
            }
            fn from_ptr(p: *const T) -> Self {
                Self(p, PhantomData)
            }
        }

        impl<T> Copy for Ref<'_, T> {}

        impl<T> Clone for Ref<'_, T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<T: Debug> Debug for Ref<'_, T> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.debug_struct("Ref")
                    .field("ptr", &self.0)
                    // .field("val", unsafe{ &*self.0 })
                    .finish()
            }
        }

        #[repr(transparent)]
        pub struct RefMut<'a, T>(*mut T, PhantomData<&'a mut T>);

        impl<'a, T> RefMut<'a, T> {
            fn new(r: &'a mut T) -> Self {
                Self(r, PhantomData)
            }
            fn from_ptr(p: *mut T) -> Self {
                Self(p, PhantomData)
            }
        }

        impl<T> Copy for RefMut<'_, T> {}

        impl<T> Clone for RefMut<'_, T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<T: Debug> Debug for RefMut<'_, T> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.debug_struct("RefMut")
                    .field("ptr", &self.0)
                    // .field("val", unsafe{ &*self.0 })
                    .finish()
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

        impl<'a, T> ConvertToC<Ref<'a, T>> for &'a T {
            fn to_c(self) -> Ref<'a, T> {
                Ref::new(self)
            }
        }

        impl<'a, T> ConvertToC<Ref<'a, T>> for Option<&'a T> {
            fn to_c(self) -> Ref<'a, T> {
                unsafe { Ref::from_ptr(self.as_ptr()) }
            }
        }

        impl<'a, T> ConvertToC<RefMut<'a, T>> for &'a mut T {
            fn to_c(self) -> RefMut<'a, T> {
                RefMut::new(self)
            }
        }

        impl<'a, T> ConvertToC<RefMut<'a, T>> for Option<&'a mut T> {
            fn to_c(mut self) -> RefMut<'a, T> {
                unsafe { RefMut::from_ptr(self.as_mut_ptr()) }
            }
        }

        impl<'a, T> ConvertToC<RefMut<'a, T>> for &'a mut MaybeUninit<T> {
            fn to_c(self) -> RefMut<'a, T> {
                unsafe { RefMut::from_ptr(self.as_mut_ptr()) }
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

        impl<'a, T> ConvertToC<Array<'a, *const T>> for &'a ArrayArray<*const T> {
            fn to_c(self) -> Array<'a, *const T> {
                unsafe { Array::from_ptr(self.0.as_ptr()) }
            }
        }

        impl<'a> ConvertToC<Array<'a, *const c_char>> for &'a ArrayArray<MyStr<'_>> {
            fn to_c(self) -> Array<'a, *const c_char> {
                unsafe { Array::from_ptr( ::std::mem::transmute::<*const MyStr, *const *const c_char>(self.0.as_ptr()) ) }
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
        use std::mem::MaybeUninit;
        use std::marker::PhantomData;
        use std::os::raw::*;
        use std::ffi::{CStr, CString};
        use std::mem::ManuallyDrop;
        use std::cell::UnsafeCell;
        use std::fmt::Debug;
        use std::fmt;
        use std::ptr;

        use handles::*;
        use private_struct_interface::*;

        #util_code
        #platform_specific_types
        #(#tokens)*
        #(#feature_enums)*
        #(#aliases)*
    };

    let mut sorted_enums: Vec<_> = global_data::all_enums().iter().collect();
    sorted_enums.sort();

    let post_process_handles = post_process_handles(&parse_state);
    let post_process_enum_display_code = make_enumeration_display_code(&sorted_enums);

    q.extend(post_process_handles);
    q.extend(post_process_enum_display_code);

    q.extend(initial_test_code);

    //for node in parse_state.command_list.iter() {
    //    dbg!(&node.data().name);
    //    dbg!(commands::command_category(node.data()));
    //}

    format!("{}", q)

}


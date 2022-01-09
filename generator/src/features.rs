
use std::collections::HashMap;

use quote::{quote, ToTokens};

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::commands::*;

// =================================================================
#[derive(Copy, Clone)]
enum RequireRemove {
    Require(VkTyName),
    Remove(VkTyName),
}

impl RequireRemove {
    fn require(name: VkTyName) -> Self {
        RequireRemove::Require(name)
    }
    fn remove(&mut self) {
        use RequireRemove::*;
        match self {
            Require(name) => *self = RequireRemove::Remove(*name),
            Remove(_) => {}
        }
    }
}

impl ToTokens for RequireRemove {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use RequireRemove::*;
        match self {
            Require(name) => {
                quote!( #name ).to_tokens(tokens);
            }
            Remove(_) => panic!("should not turn Remove into code"),
        }
    }
}

// =================================================================
// for keeping track of which list the command is in
#[derive(Copy, Clone)]
enum List {
    Instance(usize),
    Device(usize),
    Entry(usize),
}

// =================================================================
/// Command Names for a given version
/// intended to generate code within a instance/device command_names module
#[derive(Clone)]
pub struct FeatureCommands {
    version: VkTyName,
    instance_command_names: Vec<RequireRemove>,
    device_command_names: Vec<RequireRemove>,
    entry_command_names: Vec<RequireRemove>,
    // internal for quickly converting Require commands into Remove Commands
    vec_map: HashMap<VkTyName, List>,
}

impl FeatureCommands {
    pub fn new(version: impl Into<VkTyName>) -> Self {
        let version = version.into();
        Self {
            version,
            instance_command_names: Default::default(),
            device_command_names: Default::default(),
            entry_command_names: Default::default(),
            vec_map: Default::default(),
        }
    }
    pub fn as_new_version(&self, version: impl Into<VkTyName>) -> Self {
        let mut new_version = self.clone();
        new_version.version = version.into();
        new_version
    }
    pub fn push_instance_command(&mut self, command: impl Into<VkTyName>) {
        // insert index of to-be-inserted instance command and ensure not already there
        let command = command.into();
        assert!(self.vec_map.insert(command, List::Instance(self.instance_command_names.len())).is_none());
        self.instance_command_names.push(RequireRemove::require(command));
    }
    pub fn push_device_command(&mut self, command: impl Into<VkTyName>) {
        // insert index of to-be-inserted instance command and ensure not already there
        let command = command.into();
        assert!(self.vec_map.insert(command, List::Device(self.device_command_names.len())).is_none());
        self.device_command_names.push(RequireRemove::require(command));
    }
    pub fn push_entry_command(&mut self, command: impl Into<VkTyName>) {
        // insert index of to-be-inserted instance command and ensure not already there
        let command = command.into();
        assert!(self.vec_map.insert(command, List::Entry(self.entry_command_names.len())).is_none());
        self.entry_command_names.push(RequireRemove::require(command));
    }
    pub fn remove_command(&mut self, command: impl Into<VkTyName>) {
        let command = command.into();
        match self.vec_map.get(&command) {
            Some(List::Instance(index)) => self.instance_command_names[*index].remove(),
            Some(List::Device(index)) => self.device_command_names[*index].remove(),
            Some(List::Entry(index)) => self.entry_command_names[*index].remove(),
            None => panic!("should not be trying to remove command that was never requiered"),
        }
    }
}

impl ToTokens for FeatureCommands {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let version = self.version.as_code();
        let instance_command_names: Vec<_> = self.instance_command_names.iter().filter(|cmd|matches!(cmd,RequireRemove::Require(_))).collect();
        let instance_command_names = &instance_command_names;
        let device_command_names: Vec<_> = self.device_command_names.iter().filter(|cmd|matches!(cmd,RequireRemove::Require(_))).collect();
        let device_command_names = &device_command_names;
        let entry_command_names: Vec<_> = self.entry_command_names.iter().filter(|cmd|matches!(cmd,RequireRemove::Require(_))).collect();
        let entry_command_names = &entry_command_names;
        quote!(
            macro_rules! #version {
                ( @INSTANCE $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#instance_command_names),* );
                };
                ( @DEVICE $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#device_command_names),* );
                };
                ( @ENTRY $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#entry_command_names),* );
                };
                ( @ALL $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#instance_command_names),* ; #(#device_command_names),* );
                };
            }
        ).to_tokens(tokens);
    }
}

// =================================================================
/// list of all existing Vulkan versions
#[derive(Default)]
pub struct VulkanVersionNames<'a> {
    versions: Vec<&'a str>,
}

impl<'a> VulkanVersionNames<'a> {
    pub fn push_version(&mut self, version: &'a str) {
        self.versions.push(version);
    }
}

impl ToTokens for VulkanVersionNames<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let versions = self.versions.iter()
            .map(StrAsCode::as_code);
        let version_tuple = self.versions.iter()
            .map(|v| parse_version(v).as_code());
        quote!(
            macro_rules! use_all_vulkan_version_names {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#versions => #version_tuple),* );
                }
            }
        ).to_tokens(tokens);
    }
}

fn parse_version(ver: &str) -> String {

    let mut tokens = ver.split('_');

    // assert that first text is equal to VK and VERSION
    tokens.next().map(|version|assert_eq!(version, "VK")).expect("Error parsing version, no 'VK' ...");
    tokens.next().map(|version|assert_eq!(version, "VERSION")).expect("Error parsing version, no 'VERSION' ...");
    let major = tokens.next().expect("error: parsing version can't get major number");
    let minor = tokens.next().expect("error: parsing version can't get minor number");

    // Note: I am assuming that the major and minor that are parsed are integers

    format!("({}, {}, {})", major, minor, 0)

}
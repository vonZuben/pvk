
use quote::quote;
use quote::ToTokens;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::utils;
use crate::commands::*;
use crate::constants;

use std::borrow::Cow;

// used to represent names of commands that are enabled by an extension and possible extra commands when other features/extensions are available
// base: base extension
// extra: feature or extension that adds more commands
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExtensionName<'a> {
    Noraml(&'a str),
    Extended(&'a str, &'a str),
}

impl<'a> ExtensionName<'a> {
    pub fn new(base: &'a str, extra: Option<&'a str>) -> Self {
        match extra {
            Some(extra) => {
                ExtensionName::Extended(base, extra)
            }
            None => {
                ExtensionName::Noraml(base)
            }
        }
    }
}

impl ToTokens for ExtensionName<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExtensionName::Noraml(name) => {
                let name = name.as_code();
                quote!(#name).to_tokens(tokens);
            }
            ExtensionName::Extended(base, extend) => {
                let name = format!("{}_WITH_{}", base, extend).as_code();
                quote!(#name).to_tokens(tokens);
            }
        }
    }
}

// =================================================================
#[derive(Clone, Copy)]
pub enum ExtensionKind {
    Instance,
    Device,
}

impl ToTokens for ExtensionKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Instance => quote!(INSTANCE).to_tokens(tokens),
            Self::Device => quote!(DEVICE).to_tokens(tokens),
        }
    }
}

/// Command Names for a given extension
/// intended to generate code within a instance/device extension_names module
pub struct ExtensionInfo<'a> {
    extension_name: ExtensionName<'a>,
    instance_command_names: Vec<&'a str>,
    device_command_names: Vec<&'a str>,
    kind: ExtensionKind,
    required: Vec<&'a str>,
}

impl<'a> ExtensionInfo<'a> {
    pub fn new(extension_name: ExtensionName<'a>, kind: ExtensionKind) -> Self {
        Self {
            extension_name,
            instance_command_names: Default::default(),
            device_command_names: Default::default(),
            kind,
            required: Default::default(),
        }
    }
    pub fn push_instance_command(&mut self, command: &'a str) {
        self.instance_command_names.push(command);
    }
    pub fn push_device_command(&mut self, command: &'a str) {
        self.device_command_names.push(command);
    }
    pub fn require(&mut self, require: impl Iterator<Item = &'a str>) {
        self.required.extend(require);
    }
}

impl ToTokens for ExtensionInfo<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let kind = self.kind;
        let extension_name = &self.extension_name;
        let instance_command_names: Vec<_> = self.instance_command_names.iter().map(StrAsCode::as_code).collect();
        let instance_command_names = &instance_command_names;
        let device_command_names: Vec<_> = self.device_command_names.iter().map(StrAsCode::as_code).collect();
        let device_command_names = &device_command_names;
        let all_commands_names = instance_command_names.iter().chain(device_command_names.iter());
        let required = self.required.iter().map(StrAsCode::as_code);
        let load = match self.extension_name {
            ExtensionName::Noraml(name) => Some(name),
            ExtensionName::Extended(_, _) => None,
        };
        quote!(
            macro_rules! #extension_name {
                ( @KIND $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #kind );
                };
                ( @INSTANCE_COMMANDS $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#instance_command_names),* );
                };
                ( @DEVICE_COMMANDS $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#device_command_names),* );
                };
                ( @ALL_COMMANDS $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#all_commands_names),* );
                };
                ( @REQUIRE $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #load ; #(#required),* );
                };
            }
        ).to_tokens(tokens);
    }
}

// =================================================================
/// list of all existing Vulkan extensions
#[derive(Default)]
pub struct VulkanExtensionNames<'a> {
    extensions: Vec<ExtensionName<'a>>,
}

impl<'a> VulkanExtensionNames<'a> {
    pub fn push_extension(&mut self, extension_name: ExtensionName<'a>) {
        self.extensions.push(extension_name);
    }
}

impl ToTokens for VulkanExtensionNames<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let extension_names = &self.extensions;
        quote!(
            macro_rules! use_all_vulkan_extension_names {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#extension_names),* );
                }
            }
        ).to_tokens(tokens);
    }
}
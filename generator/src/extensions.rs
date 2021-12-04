
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
pub enum ExtensionCommandName<'a> {
    Noraml(&'a str),
    Extended(&'a str, &'a str),
}

impl<'a> ExtensionCommandName<'a> {
    pub fn new(base: &'a str, extra: Option<&'a str>) -> Self {
        match extra {
            Some(extra) => {
                ExtensionCommandName::Extended(base, extra)
            }
            None => {
                ExtensionCommandName::Noraml(base)
            }
        }
    }
}

impl ToTokens for ExtensionCommandName<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExtensionCommandName::Noraml(name) => {
                let name = name.as_code();
                quote!(#name).to_tokens(tokens);
            }
            ExtensionCommandName::Extended(base, extend) => {
                let name = format!("{}_WITH_{}", base, extend).as_code();
                quote!(#name).to_tokens(tokens);
            }
        }
    }
}

// =================================================================
/// Command Names for a given extension
/// intended to generate code within a instance/device extension_names module
pub struct ExtensionCommands<'a> {
    extension: ExtensionCommandName<'a>,
    instance_command_names: Vec<&'a str>,
    device_command_names: Vec<&'a str>,
    required: Vec<&'a str>,
}

impl<'a> ExtensionCommands<'a> {
    pub fn new(extension: ExtensionCommandName<'a>) -> Self {
        Self {
            extension,
            instance_command_names: Default::default(),
            device_command_names: Default::default(),
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

impl ToTokens for ExtensionCommands<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let extension = &self.extension;
        let instance_command_names: Vec<_> = self.instance_command_names.iter().map(StrAsCode::as_code).collect();
        let instance_command_names = &instance_command_names;
        let device_command_names: Vec<_> = self.device_command_names.iter().map(StrAsCode::as_code).collect();
        let device_command_names = &device_command_names;
        let required = self.required.iter().map(StrAsCode::as_code);
        quote!(
            macro_rules! #extension {
                ( @INSTANCE $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#instance_command_names),* );
                };
                ( @DEVICE $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#device_command_names),* );
                };
                ( @ALL $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#instance_command_names),* ; #(#device_command_names),* );
                };
                ( @REQUIRE $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#required),* );
                };
            }
        ).to_tokens(tokens);
    }
}

// =================================================================
/// list of all existing Vulkan extensions
#[derive(Default)]
pub struct VulkanExtensionNames<'a> {
    extensions: Vec<&'a str>,
}

impl<'a> VulkanExtensionNames<'a> {
    pub fn push_extension(&mut self, extension: &'a str) {
        self.extensions.push(extension);
    }
}

impl ToTokens for VulkanExtensionNames<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let extensions = self.extensions.iter().map(StrAsCode::as_code);
        quote!(
            macro_rules! use_all_vulkan_extension_names {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#extensions),* );
                }
            }
        ).to_tokens(tokens);
    }
}

// generate macro to use all extensions with commands (cover cases of extension WITH feature/extension)
pub struct VulkanExtensionNamesExtended<I> {
    extensions: I,
}

impl<I> VulkanExtensionNamesExtended<I> {
    pub fn new(extensions: I) -> Self {
        Self {
            extensions,
        }
    }
}

impl<'a, I: Iterator<Item = &'a ExtensionCommands<'a>> + Clone> ToTokens for VulkanExtensionNamesExtended<I> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let extensions = self.extensions.clone().map(|dc|dc.extension);
        quote!(
            macro_rules! use_all_vulkan_extension_names_extended {
                ( $call:ident $($pass:tt)* ) => {
                    $call!( $($pass)* #(#extensions),* );
                }
            }
        ).to_tokens(tokens);
    }
}
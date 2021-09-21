
use quote::quote;
use quote::ToTokens;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::utils;
use crate::commands::*;
use crate::constants;

// =================================================================
/// Command Names for a given extension
/// intended to generate code within a instance/device extension_names module
pub struct ExtensionCommands<'a> {
    extension: &'a str,
    instance_command_names: Vec<&'a str>,
    device_command_names: Vec<&'a str>,
}

impl<'a> ExtensionCommands<'a> {
    pub fn new(extension: &'a str) -> Self {
        Self {
            extension,
            instance_command_names: Default::default(),
            device_command_names: Default::default(),
        }
    }
    pub fn push_instance_command(&mut self, command: &'a str) {
        self.instance_command_names.push(command);
    }
    pub fn push_device_command(&mut self, command: &'a str) {
        self.device_command_names.push(command);
    }
}

impl ToTokens for ExtensionCommands<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let extension = self.extension.as_code();
        let instance_command_names: Vec<_> = self.instance_command_names.iter().map(StrAsCode::as_code).collect();
        let instance_command_names = &instance_command_names;
        let device_command_names: Vec<_> = self.device_command_names.iter().map(StrAsCode::as_code).collect();
        let device_command_names = &device_command_names;
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
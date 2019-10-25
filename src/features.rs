
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::commands::*;

pub fn handle_features(features: &Features, parse_state: &mut crate::ParseState) -> TokenStream {

    // a given feature should also load all previous features
    //
    // e.g. 1.1 only specifies things added in 1.1 and still needs
    // to load things specified in 1.0
    //
    // by including the previous feature in every feature
    // we recursively load everything
    //
    // using parse_state to keep track of this
    //
    // NOTE this assumes that all features will be parsed in order of earliest to latest

    let q = features.elements.iter().map(|feature| {

        let name = feature.name.as_code();

        let requiered_command_names = feature.elements.iter().filter_map(
            |feature_elem| match feature_elem {
                FeatureElement::Require(spec) => Some(spec.elements.iter()),
                _ => None,
            }).flatten()
        .filter_map(|feature_ref| match feature_ref {
            FeatureReference::CommandReference(cmd_ref) => Some(&cmd_ref.name),
            _ => None,
        });

        let instance_macro_names = requiered_command_names.clone().map(|cmd_name| {
            let instance_macro_name = make_macro_name_instance(&cmd_name);
            quote!( #instance_macro_name )
        });

        let device_macro_names = requiered_command_names.clone().map(|cmd_name| {
            let device_macro_name = make_macro_name_device(&cmd_name);
            quote!( #device_macro_name )
        });

        let previous_feature_instance = &parse_state.previous_feature_instance;
        let previous_feature_device = &parse_state.previous_feature_device;

        let q = quote!{

            pub struct #name;

            impl Feature for #name {
                fn load_instance_commands(&self, instance: &Instance, inst_cmds: &mut InstanceCommands) {
                    #( #instance_macro_names!(instance, inst_cmds); )*
                    #( #previous_feature_instance )*
                }
                fn load_device_commands(&self, device: &Device, dev_cmds: &mut DeviceCommands) {
                    #( #device_macro_names!(device, dev_cmds); )*
                    #( #previous_feature_device )*
                }
            }
        };

        parse_state.previous_feature_instance = Some(quote!{
            let previous_feature = #name;
            previous_feature.load_instance_commands(instance, inst_cmds);
        });
        parse_state.previous_feature_device = Some(quote!{
            let previous_feature = #name;
            previous_feature.load_device_commands(device, dev_cmds);
        });
        q

    });

    quote!( #( #q )* )

}

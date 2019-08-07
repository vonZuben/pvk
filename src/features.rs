
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::commands::*;

pub fn handle_features(features: &Features) -> TokenStream {

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

        //for name in requiered_command_names {
        //    dbg!(name);
        //}


        //let instance_load_instructions = requiered_command_names.clone().map(|cmd_name| {
        //    let instance_macro_name = make_macro_name_instance(&cmd_name);
        //    //let device_macro_name = make_macro_name_device(&cmd_name);
        //    quote!(
        //});

        let instance_macro_names = requiered_command_names.clone().map(|cmd_name| {
            let instance_macro_name = make_macro_name_instance(&cmd_name);
            quote!( #instance_macro_name )
        });

        let device_macro_names = requiered_command_names.clone().map(|cmd_name| {
            let device_macro_name = make_macro_name_device(&cmd_name);
            quote!( #device_macro_name )
        });

        quote!{

            //macro_rules! expand_appripriate_cmds {

            //    ( $cmd_container:ident, ) => { }

            //    ( $cmd_container:ident -> $cmd_name:ident ) => {
            //        $cmd_container.$cmd_name.0.load();
            //    }

            //    ( $cmd_container:ident, $cmd_name:tt ) => {
            //        exp
            //    }

            //}

            pub struct #name;

            impl #name {
                fn load_instance_commands(instance: &Instance, inst_cmds: &mut InstanceCommands) {
                    #( #instance_macro_names!(instance, inst_cmds); )*
                }
                fn load_device_commands(device: &Device, dev_cmds: &mut DeviceCommands) {
                    #( #device_macro_names!(device, dev_cmds); )*
                }
            }
        }

    });

    quote!( #( #q )* )

}

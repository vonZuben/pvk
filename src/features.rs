
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::commands::*;
use crate::global_data;

fn parse_version(ver: &str) -> TokenStream {

    let mut tokens = ver.split('_');

    // assert that first text is equal to VK and VERSION
    tokens.next().map(|version|assert_eq!(version, "VK")).expect("Error parsing version, no 'VK' ...");
    tokens.next().map(|version|assert_eq!(version, "VERSION")).expect("Error parsing version, no 'VERSION' ...");
    let major = tokens.next().expect("error: parsing version can't get major number").as_code();
    let minor = tokens.next().expect("error: parsing version can't get minor number").as_code();

    // Note: I am assuming that the major and minor that are parsed are integers

    quote!( vk_make_version(#major, #minor, 0) )

}

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

        let ver = parse_version(&feature.name);

        let requiered_command_names = feature.elements.iter().filter_map(
            |feature_elem| match feature_elem {
                FeatureElement::Require(spec) => Some(spec.elements.iter()),
                _ => None,
            }).flatten()
        .filter_map(|feature_ref| match feature_ref {
            FeatureReference::CommandReference(cmd_ref) => Some(&cmd_ref.name),
            _ => None,
        });

        macro_rules! filter_global_command_type{
            ( $varient:path ) => {
                |val| match global_data::command_type(val) {
                    $varient => true,
                    _ => false,
                }
            }
        }
        let instance_commands = requiered_command_names.clone()
            .filter(filter_global_command_type!(CommandCategory::Instance)).map(StrAsCode::as_code);
        let device_commands = requiered_command_names.clone()
            .filter(filter_global_command_type!(CommandCategory::Device)).map(StrAsCode::as_code);

        let previous_feature_instance = &parse_state.previous_feature_instance;
        let previous_feature_device = &parse_state.previous_feature_device;

        let q = quote!{

            #[derive(Clone, Copy, Debug)]
            pub struct #name;

            impl FeatureCore for #name {
                fn load_instance_commands(&self, instance: Instance, inst_cmds: &mut InstanceCommands) {
                    let loader = |raw_cmd_name: &CStr| unsafe { GetInstanceProcAddr(instance, raw_cmd_name.to_c()) };
                    #( inst_cmds.#instance_commands.load(loader); )*
                    #previous_feature_instance
                }
                fn load_device_commands(&self, device: Device, dev_cmds: &mut DeviceCommands) {
                    let loader = |raw_cmd_name: &CStr| unsafe { GetDeviceProcAddr(device, raw_cmd_name.to_c()) };
                    #( dev_cmds.#device_commands.load(loader); )*
                    #previous_feature_device
                }
                fn version(&self) -> u32 {
                    #ver
                }
                fn clone_feature(&self) -> Box<dyn Feature> {
                    Box::new(self.clone())
                }
            }

            impl From<#name> for FeatureWrapper {
                fn from(f: #name) -> Self {
                    Self( Box::new(f) )
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

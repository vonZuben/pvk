
use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::commands::*;
use crate::global_data;

pub fn handle_extensions<'a>(extensions: &'a Extensions, parse_state: &mut crate::ParseState<'a>) -> TokenStream {

    let q = extensions.elements.iter().map(|extension| {

        // some extensions are just placeholders and do not have a type
        // thus, we should not generate any code for them since they have no function
        if extension.ty.is_none() {
            return quote!();
        }

        macro_rules! filter_varients {
            ( $varient:path ) => {
                |spec| {
                    match spec {
                        $varient(inner) => Some(inner),
                        _ => None,
                    }
                }
            }
        };

        // NOTE the current code does not handle 'Removed' functionality
        // i.e. at the time of writing this, the vulkan spec does not remove
        // any functions in any features or extensions. Thus, we ignore the
        // Remove case for now

        let enum_constants_name_cache = &mut parse_state.enum_constants_name_cache;
        let command_alias_cache = &parse_state.command_alias_cache;
        let enum_cache = &mut parse_state.enum_cache;

        let enum_extensions = extension.elements.iter()
            .filter_map(filter_varients!(ExtensionElement::Require))
            .map(|extension_spec| extension_spec.elements.iter()
                 .filter_map(filter_varients!(ExtensionSpecificationElement::Enum))
                 )
            .flatten()
            .map(|enum_extension| {

                // add enum constant names to cache and if the name already exists, then do not
                // generate duplicate
                match enum_constants_name_cache.insert(enum_extension.name.as_str(), ()) {
                    Some(_) => return quote!() ,
                    None => {}
                }

                enum_cache.get_mut(enum_extension.extends.as_str())
                    .expect("error: extension enum not in cahce")
                    .push(enum_extension.name.as_str());

                let name = enum_extension.extends.as_code();
                let const_name = crate::enumerations
                    ::make_varient_name(enum_extension.extends.as_str(), enum_extension.name.as_str());

                let val = one_option!(

                    &enum_extension.offset , |offset: &usize|
                    {
                        // see vulkan spec style guide regarding this equation
                        let val = 1000000000 + (extension.number - 1) * 1000 + *offset as i32;
                        if enum_extension.negate {
                            -val
                        }
                        else {
                            val
                        }.to_string().as_code()
                    };

                    &enum_extension.number , |num: &i32|
                        if enum_extension.negate {
                           -*num
                        }
                        else {
                           *num
                        }.to_string().as_code() ;

                    &enum_extension.hex , |_hex| panic!(
                        format!("not expecting hex in enum extension definition: {}", enum_extension.name)) ;

                    // shouldn't have negative bit positions
                    &enum_extension.bitpos , |bitpos: &u32| format!("0x{:0>8X}", (1u32 << bitpos)).as_code() ;

                );

                quote!{
                    impl #name {
                        pub const #const_name: Self = #name(#val);
                    }
                }

            });

        let constant_extensions = extension.elements.iter()
            .filter_map(filter_varients!(ExtensionElement::Require))
            .map(|extension_spec| extension_spec.elements.iter()
                 .filter_map(filter_varients!(ExtensionSpecificationElement::Constant))
                 )
            .flatten()
            .map(|const_extension| {
                let name = const_extension.name();
                let ty = const_extension.ty();
                let val = const_extension.val();
                quote!( pub const #name: #ty = #val; )
            });

        let commands_to_load = extension.elements.iter()
            .filter_map(filter_varients!(ExtensionElement::Require))
            .map(|extension_spec| extension_spec.elements.iter()
                 .filter_map(filter_varients!(ExtensionSpecificationElement::CommandReference))
                 )
            .flatten()
            .map(|command_ref| {
                // check the command_alias_cache to see if the extension identifies an alias
                let name = command_alias_cache.get(command_ref.name.as_str())
                    .map_or(command_ref.name.as_str(), |alias| *alias);
                let name_code = name.as_code();
                match global_data::command_type(name) {
                    CommandCategory::Instance => {
                        quote!( inst_cmds.#name_code.load( |raw_cmd_name|
                                                 unsafe { GetInstanceProcAddr(*instance, raw_cmd_name.to_c()) } ) )
                    }
                    CommandCategory::Device => {
                        quote!( dev_cmds.#name_code.load( |raw_cmd_name|
                                                 unsafe { GetDeviceProcAddr(*device, raw_cmd_name.to_c()) } ) )
                    }
                    CommandCategory::Static => panic!(
                        format!("error: extension command is for static command: {}",
                                command_ref.name.as_str()) ),
                }
            });

        //for _ in commands_to_load {}

        //for x in commands_to_load {
        //    dbg!(&extension);
        //    dbg!(extension.name.as_str());
        //    dbg!(x);
        //}

        let name = extension.name.as_code();
        let command_load_code;
        if commands_to_load.clone().count() == 0 || extension.name.as_str() == "VK_EXT_debug_utils" {
            command_load_code = quote!();
        }
        else {
            match extension.ty.as_ref().expect(format!("error: extension without type {}", extension.name).as_str()) {
                ExtensionType::Instance => {
                    // when loading instance extensions, only instance commands can be loaded
                    //
                    // NOTE there is one known exception to the above
                    // in particular, EXT_debug_utils is a combination of a
                    // previously separate instance and device extensions
                    command_load_code = quote!{
                        fn load_commands(&self,
                                         instance: &Instance, inst_cmds: &mut InstanceCommands) {
                            #( #commands_to_load; )*
                        }
                    };
                }
                ExtensionType::Device => {
                    // when loading device extensions, it is possible to also load some instance
                    // commands
                    command_load_code = quote!{
                        fn load_commands(&self,
                                         instance: &Instance, inst_cmds: &mut InstanceCommands,
                                         device: &Device, dev_cmds: &mut DeviceCommands) {
                            #( #commands_to_load; )*
                        }
                    };
                }
            }
        }

        quote!{
            #( #enum_extensions )*
            #( #constant_extensions )*
            struct #name;
            impl #name {
                #command_load_code
            }
        }

    });

    quote!( #( #q )* )

}

// similar to ConstExt in constant.rs for Constant from vkxml
// except it is modified for ExtensionConstant from vkxml
trait ConstExtExt {
    fn ty(&self) -> TokenStream;
    fn val(&self) -> TokenStream;
    fn name(&self) -> TokenStream;
}

impl ConstExtExt for vkxml::ExtensionConstant {

    fn ty(&self) -> TokenStream {
        one_option!{

            &self.text , |_| quote!(&'static str) ;

            &self.enumref , |_| quote!(usize) ; // TODO: is this correct?

            &self.number , |_| quote!(usize) ;

            &self.hex , |_| panic!(
                format!("error: trying to get hex type not implemented for ConstExtExt -> {}", self.name)) ;

            &self.bitpos , |_| panic!(
                format!("error: trying to get bitpos type not implemented for ConstExtExt -> {}", self.name)) ;

            &self.c_expression , |_expr: &str| panic!(
                format!("error: trying to get c_expression type not implemented for ConstExtExt -> {}", self.name)) ;

        }

    }
    fn val(&self) -> TokenStream {
        one_option!{

            &self.text , |sval| quote!(#sval) ;

            &self.enumref , |r: &str| r.as_code() ;

            &self.number , |num: &i32| { num.to_string().as_code() } ;

            &self.hex , |_| panic!(
                format!("error: trying to get hex type not implemented for ConstExtExt -> {}", self.name)) ;

            &self.bitpos , |_| panic!(
                format!("error: trying to get bitpos type not implemented for ConstExtExt -> {}", self.name)) ;

            &self.c_expression , |_expr: &str| panic!(
                format!("error: trying to get c_expression type not implemented for ConstExtExt -> {}", self.name)) ;

        }
    }
    fn name(&self) -> TokenStream {
        self.name.as_code()
    }
}

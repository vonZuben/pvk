
use quote::quote;
use quote::ToTokens;

use vkxml::*;

use proc_macro2::{TokenStream};

#[macro_use]
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

        // NOTE the current code does not handle 'Removed' functionality
        // i.e. at the time of writing this, the vulkan spec does not remove
        // any functions in any features or extensions. Thus, we ignore the
        // Remove case for now

        let enum_constants_name_cache = &mut parse_state.enum_constants_name_cache;
        let command_alias_cache = &parse_state.command_alias_cache;
        let enum_cache = &mut parse_state.enum_cache;

        let enum_extensions = extension.elements.iter()
            .filter_map(variant!(ExtensionElement::Require))
            .map(|extension_spec| extension_spec.elements.iter()
                 .filter_map(variant!(ExtensionSpecificationElement::Enum))
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
                    ::make_variant_name(enum_extension.extends.as_str(), enum_extension.name.as_str()).as_code();

                let val = enum_extension.val(extension.number);

                quote!{
                    impl #name {
                        pub const #const_name: Self = #name(#val);
                    }
                }

            });

        let constant_extensions = extension.elements.iter()
            .filter_map(variant!(ExtensionElement::Require))
            .map(|extension_spec| extension_spec.elements.iter()
                 .filter_map(variant!(ExtensionSpecificationElement::Constant))
                 )
            .flatten()
            .map(|const_extension| {
                // every extension should define an extension name
                // we will add a method for easily obtianing a C string 
                // of the extension name
                let extension_name_impl = if const_extension.name.ends_with("_EXTENSION_NAME") {
                    let name = const_extension.text.as_ref().expect("error: extension name without text value");
                    let c_name = name.to_string() + "\0";
                    
                    let extension_name = extension.name.as_code();

                    Some(
                        quote!{
                            impl #extension_name {
                                fn name(&self) -> &'static CStr {
                                    const NAME: &'static str = #c_name;
                                    let name_ptr = NAME.as_bytes().as_ptr() as *const c_char;
                                    // c_name must always be a valid c string name as defined in vulkan spec (i'm pretty sure)
                                    unsafe { CStr::from_ptr(name_ptr) }
                                }
                            }
                        }
                    )
                }
                else {
                    None
                };
                let name = const_extension.name();
                let ty = const_extension.ty();
                let val = const_extension.val();
                quote!{ 
                    pub const #name: #ty = #val;
                    #extension_name_impl
                }
            });

        let commands_to_load = extension.elements.iter()
            .filter_map(variant!(ExtensionElement::Require))
            .map(|extension_spec| extension_spec.elements.iter()
                 .filter_map(variant!(ExtensionSpecificationElement::CommandReference))
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

pub struct EnumVal {
    bitpos: bool,
    val: i32,
}

impl std::ops::Deref for EnumVal {
    type Target = i32;
    fn deref(&self) -> &i32 {
        &self.val
    }
}

impl ToTokens for EnumVal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.bitpos {
            format!("0x{:0>8X}", self.val).as_code().to_tokens(tokens);
        }
        else {
            format!("{}", self.val).as_code().to_tokens(tokens);
        }
    }
}

pub trait EnumExtExt {
    fn val(&self, extension_number: i32) -> EnumVal; // this is s String representation of a value of any type for converting into code
}

impl EnumExtExt for ExtensionEnum {
    fn val(&self, extension_number: i32) -> EnumVal {
        one_option!{

            &self.offset , |offset: &usize|
            {
                // see vulkan spec style guide regarding this equation
                let val = 1000000000 + (extension_number - 1) * 1000 + *offset as i32;
                if self.negate {
                    EnumVal{ val: -val, bitpos: false }
                }
                else {
                    EnumVal{ val: val, bitpos: false }
                }
            };

            &self.number , |num: &i32|
                if self.negate {
                    EnumVal{ val: -*num, bitpos: false }
                }
                else {
                    EnumVal{ val: *num, bitpos: false }
                } ;

            &self.hex , |_hex| panic!(
                format!("not expecting hex in enum extension definition: {}", self.name)) ;

            // shouldn't have negative bit positions
            &self.bitpos , |bitpos: &u32| EnumVal{ val: 1i32 << bitpos, bitpos: true } ;

        }
    }
}

pub fn generate_feature_enums_from_vk_parse_reg<'a>(feature: &'a vk_parse::Feature) -> TokenStream {
    let q = feature.children.iter()
        .filter_map(
            |feature| {
                match feature {
                    vk_parse::ExtensionChild::Require{items, ..} => {
                        Some( items.iter() )
                    }
                    _ => None,
                }
            }
        )
        .flatten()
        .filter_map(
            |interface_item| {
                match interface_item {
                    vk_parse::InterfaceItem::Enum(enm) => {
                        match &enm.spec {
                            vk_parse::EnumSpec::Alias{alias: _, extends: _} => {
                                unimplemented!("not yet dealing with Aliases for enums defined by features");
                            }
                            vk_parse::EnumSpec::Offset{offset, extends, extnumber, dir} => {
                                let extension_number = extnumber.expect("Feature defined enum with Offset value and no extnumber");
                                let mut val = 1000000000 + (extension_number - 1) * 1000 + *offset;
                                if !dir {
                                    val = -val;
                                }
                                let val = format!("{}", val).as_code();
                                let const_name = crate::enumerations
                                    ::make_variant_name(extends.as_str(), enm.name.as_str()).as_code();
                                let extends = extends.as_code();
                                Some(
                                    quote!{
                                        impl #extends {
                                            pub const #const_name: Self = #extends(#val);
                                        }
                                    }
                                )
                            }
                            vk_parse::EnumSpec::Bitpos{bitpos, extends} => {
                                let val = 1i32 << bitpos;
                                let val = format!("0x{:0>8X}", val).as_code();
                                match extends {
                                    Some(extends) => {
                                        let const_name = crate::enumerations
                                            ::make_variant_name(extends.as_str(), enm.name.as_str()).as_code();
                                        let extends = extends.as_code();
                                        Some(
                                            quote!{
                                                impl #extends {
                                                    pub const #const_name: Self = #extends(#val);
                                                }
                                            }
                                        )
                                    }
                                    None => unimplemented!("not yet handle Feature defined enum with Bitset that dosn't extend another enum"),
                                }
                            }
                            vk_parse::EnumSpec::Value{value: _, extends: _} => {
                                unimplemented!("not yet handle Feature defined enum with Value")
                            }
                            vk_parse::EnumSpec::None => None
                        }
                    }
                    _ => None,
                }
            }
        );
        
    quote!( #(#q)* )
}
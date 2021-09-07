use std::collections::HashMap;

use quote::{quote, ToTokens};
use proc_macro2::TokenStream;

use crate::vkxml_visitor;
use crate::vkxml_visitor::{VisitExtension, VisitFeature, VisitVkxml};

use crate::vk_parse_visitor::{VisitVkParse};

use crate::utils;

use crate::cfield;
use crate::commands;
use crate::constants;
use crate::ctype;
use crate::definitions;
use crate::enumerations;
use crate::extensions;
use crate::features;

#[derive(Copy, Clone)]
enum CommandType {
    Instance,
    Device,
    Static,
    DoNotGenerate,
    Entry,
}

fn command_type(command: &vkxml::Command) -> CommandType {
    match command.name.as_str() {
        "vkGetInstanceProcAddr" | "vkGetDeviceProcAddr" => CommandType::Static,
        "vkEnumerateInstanceVersion" => CommandType::DoNotGenerate, // this function is manually created in lib.rs in order to support VK 1.0
        _ => match command.param[0].basetype.as_str() {
            "VkDevice" | "VkCommandBuffer" | "VkQueue" => CommandType::Device,
            "VkInstance" | "VkPhysicalDevice" => CommandType::Instance,
            _ => CommandType::Entry,
        },
    }
}

#[derive(Default)]
pub struct Generator<'a> {
    // metadata
    // when generating commands to load per feature, we use this to determine command_types
    command_types: HashMap<&'a str, CommandType>,

    // code generation
    definitions: definitions::Definitions2<'a>,
    constants: Vec<constants::Constant2<'a>>,
    enum_variants: utils::VecMap<&'a str, enumerations::EnumVariants<'a>>,
    commands: commands::Commands2<'a>,
    vulkan_version_names: features::VulkanVersionNames<'a>,
    feature_commands: Vec<features::FeatureCommands<'a>>,
    vulkan_extension_names: extensions::VulkanExtensionNames<'a>,
    extension_commands: Vec<extensions::ExtensionCommands<'a>>,
    aliases: utils::VecMap<&'a str, definitions::TypeDef<'a>>,
}

impl Generator<'_> {
    fn get_command_type(&self, cmd: &str) -> Option<CommandType> {
        match self.command_types.get(cmd) {
            Some(cmd_type) => Some(*cmd_type),
            None => {
                let alias_name = self.aliases.get(cmd)?.ty;
                self.command_types.get(alias_name).map(|ct|*ct)
            }
        }
    }
}

impl ToTokens for Generator<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let definitions = &self.definitions;
        let constants = &self.constants;
        let enum_variants = self.enum_variants.iter();
        let commands = &self.commands;
        let vulkan_version_names = &self.vulkan_version_names;
        let feature_commands = &self.feature_commands;
        let vulkan_extension_names = &self.vulkan_extension_names;
        let extension_commands = &self.extension_commands;

        quote!(
            #definitions
            #(#constants)*
            #(#enum_variants)*
            #commands
            #vulkan_version_names
            #(#feature_commands)*
            vulkan_extension_names
            #(#extension_commands)*
        ).to_tokens(tokens);
    }
}

// =================================================================
// vkxml
// =================================================================
impl<'a> VisitVkxml<'a> for Generator<'a> {
    fn visit_type_def(&mut self, type_def: &'a vkxml::Typedef) {
        let type_def = definitions::TypeDef::new(&type_def.name, &type_def.basetype);
        self.definitions.type_defs.push(type_def);
    }

    fn visit_bitmask(&mut self, bitmask: &'a vkxml::Bitmask) {
        let bitmask = definitions::Bitmask::new(&bitmask.name, &bitmask.basetype);
        self.definitions.bitmasks.push(bitmask);
    }

    fn visit_struct(&mut self, struct_def: &'a vkxml::Struct) {
        let mut stct = definitions::Struct2::new(&struct_def.name);
        let fields = struct_def.elements.iter().filter_map(|struct_element| {
            use vkxml::StructElement;
            match struct_element {
                StructElement::Notation(_) => None,
                StructElement::Member(field) => Some(make_cfield(field, FieldPurpose::StructField)),
            }
        });
        stct.extend_fields(fields);
        self.definitions.structs.push(stct);
    }

    fn visit_union(&mut self, union_def: &'a vkxml::Union) {
        let mut uni = definitions::Union::new(&union_def.name);
        let fields = union_def
            .elements
            .iter()
            .map(|field| make_cfield(field, FieldPurpose::StructField));
        uni.extend_fields(fields);
        self.definitions.unions.push(uni);
    }

    fn visit_handle(&mut self, handle: &'a vkxml::Handle) {
        let handle = definitions::Handle2::new(&handle.name);
        self.definitions.handles.push(handle);
    }

    fn visit_enum_def(&mut self, enum_def: &'a vkxml::EnumerationDeclaration) {
        let enum_def = definitions::Enum2::new(&enum_def.name);
        self.definitions.enumerations.push(enum_def);
    }

    fn visit_function_pointer(&mut self, function_pointer: &'a vkxml::FunctionPointer) {
        let mut fptr = definitions::FunctionPointer::new(&function_pointer.name);
        let fields = function_pointer
            .param
            .iter()
            .map(|field| make_cfield(field, FieldPurpose::FunctionParam));
        fptr.extend_fields(fields);
        fptr.set_return_type(make_return_ctype(&function_pointer.return_type));
        self.definitions.function_pointers.push(fptr);
    }

    fn visit_constant(&mut self, constant: &'a vkxml::Constant) {
        self.constants.push(constants::Constant2::new(
            &constant.name,
            constants::TypeValueExpresion::literal(constant),
        ));
    }

    fn visit_enum_variants(&mut self, enumeration: &'a vkxml::Enumeration) {
        let mut enum_variant = enumerations::EnumVariants::new(&enumeration.name);
        let variants = enumeration
            .elements
            .iter()
            .filter_map(|enumeration_element| {
                use vkxml::EnumerationElement;
                match enumeration_element {
                    EnumerationElement::Notation(_) => None,
                    EnumerationElement::UnusedRange(_) => None,
                    EnumerationElement::Enum(constant) => Some(constants::Constant2::new(
                        &constant.name,
                        constants::TypeValueExpresion::simple_self(constant),
                    )),
                }
            });
        enum_variant.extend_variants(variants);
        self.enum_variants.push(&enumeration.name, enum_variant);
    }

    fn visit_command(&mut self, command: &'a vkxml::Command) {
        // get CommandType metadata for feature and extension code generation
        self.command_types
            .insert(&command.name, command_type(command));

        // generate command function_pointers
        let mut function_pointer = definitions::FunctionPointer::new(&command.name);
        let fields = command
            .param
            .iter()
            .map(|field| make_cfield(field, FieldPurpose::FunctionParam));
        function_pointer.extend_fields(fields);
        function_pointer.set_return_type(make_return_ctype(&command.return_type));
        self.commands.push(function_pointer);
    }

    fn visit_feature(&mut self, feature: &'a vkxml::Feature) {
        // collect feature/version names
        self.vulkan_version_names.push_version(&feature.name);

        // collect commands per feature/version
        let mut fc = match self.feature_commands.last() {
            Some(previous_feature) => previous_feature.as_new_version(&feature.name),
            None => features::FeatureCommands::new(&feature.name),
        };
        self.feature_commands.push(fc);
        vkxml_visitor::visit_feature(feature, self);
    }

    fn visit_extension(&mut self, extension: &'a vkxml::Extension) {
        // collect extension anmes
        self.vulkan_extension_names.push_extension(&extension.name);

        // collect command, constants, and eneum variants from extension
        let mut ex = extensions::ExtensionCommands::new(&extension.name);
        self.extension_commands.push(ex);
        vkxml_visitor::visit_extension(extension, self);
    }
}

// when visiting a Feature, the generator will modify the last FeatureCommands added
// thus, you must add the FeatureCommands you want to modify as the last one
impl<'a> VisitFeature<'a> for Generator<'a> {
    fn visit_require_command_ref(&mut self, command_ref: &'a vkxml::NamedIdentifier) {
        let cmd_name = command_ref.name.as_str();
        let fc = self
            .feature_commands
            .last_mut()
            .expect("error: no feature_command created");
        match self
            .command_types
            .get(cmd_name)
            .expect("error: feature identifies unknown command")
        {
            CommandType::Instance => fc.push_instance_command(cmd_name),
            CommandType::Device => fc.push_device_command(cmd_name),
            CommandType::Entry => fc.push_entry_command(cmd_name),
            CommandType::Static => {}
            CommandType::DoNotGenerate => {}
        }
    }

    fn visit_remove_command_ref(&mut self, command_ref: &'a vkxml::NamedIdentifier) {
        let cmd_name = command_ref.name.as_str();
        let fc = self
            .feature_commands
            .last_mut()
            .expect("error: no feature_command created");
        fc.remove_command(cmd_name);
    }
}

impl<'a> VisitExtension<'a> for Generator<'a> {
    fn visit_require_command_ref(&mut self, command_ref: &'a vkxml::NamedIdentifier) {
        let cmd_name = command_ref.name.as_str();
        let cmd_type = self
            .get_command_type(cmd_name)
            .expect("error: feature identifies unknown command");
        let ex = self
            .extension_commands
            .last_mut()
            .expect("error: no extension_commands created");
        match cmd_type {
            CommandType::Instance => ex.push_instance_command(cmd_name),
            CommandType::Device => ex.push_device_command(cmd_name),
            CommandType::Entry => {
                panic!("error: entry level command added by extension not handled")
            }
            CommandType::Static => {
                panic!("error: static level command added by extension not handled")
            }
            CommandType::DoNotGenerate => {}
        }
    }

    fn visit_require_constant(&mut self, constant: &'a vkxml::ExtensionConstant) {
        self.constants.push(constants::Constant2::new(
            &constant.name,
            constants::TypeValueExpresion::literal(constant),
        ));
    }

    fn visit_require_enum_variant(&mut self, enum_def: vkxml_visitor::VkxmlExtensionEnum<'a>) {
        let enum_name = enum_def.enum_extension.name.as_str();
        let mut enum_variants = self
            .enum_variants
            .get_mut_or_default(enum_name, enumerations::EnumVariants::new(enum_name));

        enum_variants.push_variant(constants::Constant2::new(
            &enum_def.enum_extension.name,
            constants::TypeValueExpresion::simple_self(enum_def),
        ));
    }
}

enum FieldPurpose {
    StructField,
    FunctionParam,
}

fn make_cfield<'a>(field: &'a vkxml::Field, purpose: FieldPurpose) -> cfield::Cfield<'a> {
    let mut ctype = ctype::Ctype::new(&field.basetype);
    ctype.set_public();

    set_ctype_pointer_or_array(field, purpose, &mut ctype);

    cfield::Cfield::new(
        field
            .name
            .as_ref()
            .expect("error: field in this position must have name")
            .as_str(),
        ctype,
    )
}

fn make_return_ctype<'a>(field: &'a vkxml::Field) -> ctype::ReturnType<'a> {
    if field.basetype.as_str() == "void" && field.reference.is_none() {
        return Default::default();
    } else {
        let mut ctype = ctype::Ctype::new(&field.basetype);
        // for now, assuming return type cannot be a pointer to a static size array
        assert!(
            !(matches!(field.reference, Some(_))
                && matches!(field.array, Some(vkxml::ArrayType::Static)))
        );
        set_ctype_pointer_or_array(field, FieldPurpose::FunctionParam, &mut ctype);
        ctype.into()
    }
}

fn set_ctype_pointer_or_array<'a>(
    field: &'a vkxml::Field,
    purpose: FieldPurpose,
    ctype: &mut ctype::Ctype<'a>,
) {
    use vkxml::ArrayType;

    match field.array {
        Some(ArrayType::Static) => match purpose {
            FieldPurpose::StructField => {
                let size = field
                    .size
                    .as_ref()
                    .or_else(|| field.size_enumref.as_ref())
                    .expect("error: field is static size array with no size");
                ctype.set_array(size);
            }
            FieldPurpose::FunctionParam => {
                if field.reference.is_some() {
                    // ctype.set_pointer_from_vkxml(&field.reference, field.is_const)
                    panic!("error: not sure yet how to handle static array with reference type");
                } else {
                    match field.is_const {
                        true => ctype.set_pointer(ctype::Pointer::Const),
                        false => ctype.set_pointer(ctype::Pointer::Mut),
                    }
                }
            }
        },
        Some(ArrayType::Dynamic) | None => {
            ctype.set_pointer_from_vkxml(&field.reference, field.is_const);
        }
    }
}

// =================================================================
// vk_parse
// =================================================================

impl<'a> VisitVkParse<'a> for Generator<'a> {
    fn visit_alias(&mut self, name: &'a str, alias: &'a str) {
        self.aliases.push(name, definitions::TypeDef::new(name, alias));
    }
}
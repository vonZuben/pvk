use std::collections::HashMap;

use quote::{quote, ToTokens};
use proc_macro2::TokenStream;

use krs_quote::{my_quote, my_quote_with};
use crate::utils::ToTokensInterop;

use crate::vkxml_visitor;
use crate::vkxml_visitor::{VisitExtension, VisitFeature, VisitVkxml};

use crate::vk_parse_visitor::{VisitVkParse};

use crate::utils::{self, VecMap};

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

fn command_type(command: &vk_parse::CommandDefinition) -> CommandType {
    let name = command.proto.name.as_str();
    match name {
        "vkGetInstanceProcAddr" | "vkGetDeviceProcAddr" => CommandType::Static,
        // "vkEnumerateInstanceVersion" => CommandType::DoNotGenerate, // this function is manually created in lib.rs in order to support VK 1.0
        _ => match command.params[0].definition.type_name.as_ref().expect("error: command param with no type").as_str() {
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
    command_types: HashMap<utils::VkTyName, CommandType>,

    // code generation
    definitions: definitions::Definitions2,
    constants: VecMap<utils::VkTyName, constants::Constant3<'a>>,
    enum_variants: utils::VecMap<utils::VkTyName, enumerations::EnumVariants<'a>>,
    commands: commands::Commands2,
    vulkan_version_names: features::VulkanVersionNames<'a>,
    feature_commands: Vec<features::FeatureCommands>,
    vulkan_extension_names: extensions::VulkanExtensionNames,
    extension_infos: VecMap<extensions::ExtensionName, extensions::ExtensionInfo>,
    aliases: utils::VecMap<utils::VkTyName, definitions::TypeDef>,
}

impl<'a> Generator<'a> {
    fn get_command_type(&self, cmd: &str) -> Option<CommandType> {
        let cmd = utils::VkTyName::new(cmd);
        match self.command_types.get(&cmd) {
            Some(cmd_type) => Some(*cmd_type),
            None => {
                let alias_name = self.aliases.get(cmd)?.ty;
                self.command_types.get(&alias_name).map(|ct|*ct)
            }
        }
    }

    // if there is an alias, return the alias, otherwise, return name
    fn get_alias_or_name(&self, name: utils::VkTyName) -> utils::VkTyName {
        match self.aliases.get(name) {
            Some(td) => td.ty,
            None => name,
        }
    }

    pub fn generate_output_for_single_file(&self) -> String {
        let static_code = crate::static_code::StaticCode;

        let definitions = &self.definitions;
        let constants = self.constants.iter();
        let enum_variants = self.enum_variants.iter();
        let commands = &self.commands;
        let vulkan_version_names = &self.vulkan_version_names;
        let feature_commands = &self.feature_commands;
        let vulkan_extension_names = &self.vulkan_extension_names;
        let extension_commands = self.extension_infos.iter();
        let aliases = self.aliases.iter();

        let cmd_aliases = crate::aliases::CmdAliasNames::new(
            aliases.clone().filter(|td|commands.contains(td.ty)).map(Clone::clone)
        );

        my_quote!(
            {@static_code}
            {@definitions}
            {@* {@constants}}
            {@* {@enum_variants}}
            {@commands}
            {@* {@aliases}}
            {@vulkan_version_names}
            {@* {@feature_commands}}
            {@vulkan_extension_names}
            {@* {@extension_commands}}
            {@cmd_aliases}
        ).to_string()
    }
}

// impl ToTokens for Generator<'_> {
//     fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
//         let definitions = &self.definitions;
//         let constants = &self.constants;
//         let enum_variants = self.enum_variants.iter();
//         let commands = &self.commands;
//         let vulkan_version_names = &self.vulkan_version_names;
//         let feature_commands = &self.feature_commands;
//         let vulkan_extension_names = &self.vulkan_extension_names;
//         let extension_commands = &self.extension_commands;

//         quote!(
//             #definitions
//             #(#constants)*
//             #(#enum_variants)*
//             #commands
//             #vulkan_version_names
//             #(#feature_commands)*
//             vulkan_extension_names
//             #(#extension_commands)*
//         ).to_tokens(tokens);
//     }
// }

// =================================================================
// vkxml
// =================================================================
impl<'a> VisitVkxml<'a> for Generator<'a> {
    fn visit_type_def(&mut self, type_def: &'a vkxml::Typedef) {
        if type_def.basetype.as_str() == "" {
            return; // some types are defined with no basetype. These should correspond to extern types which are hard coded into static_code.rs
            // TODO: maybe some day we can use extern types?
        }
        let type_def = definitions::TypeDef::new(&type_def.name, &type_def.basetype);
        self.definitions.type_defs.push(type_def);
    }

    fn visit_bitmask(&mut self, bitmask: &'a vkxml::Bitmask) {
        let name = utils::VkTyName::new(bitmask.name.as_str());
        assert!(bitmask.name.as_str().contains("Flags"));
        self.enum_variants.contains_or_default(name, enumerations::EnumVariants::new(name, enumerations::EnumKind::BitFlags));
        let bitmask = definitions::Bitmask::new(&bitmask.name, &bitmask.basetype);
        self.definitions.bitmasks.push(bitmask);
    }

    fn visit_struct(&mut self, struct_def: &'a vkxml::Struct) {
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
        let dispatch = match handle.ty {
            vkxml::HandleType::Dispatch => true,
            vkxml::HandleType::NoDispatch => false,
        };
        let handle = definitions::Handle2::new(&handle.name, dispatch);
        self.definitions.handles.push(handle);
    }

    fn visit_enum_def(&mut self, enum_def: &'a vkxml::EnumerationDeclaration) {
        // let enum_def = definitions::Enum2::new(&enum_def.name);
        // self.definitions.enumerations.push(enum_def);
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

    fn visit_constant(&mut self, constant: &'a vkxml::Constant) {}

    fn visit_enum_variants(&mut self, enumeration: &'a vkxml::Enumeration) {}

    fn visit_command(&mut self, command: &'a vkxml::Command) {
        // // get CommandType metadata for feature and extension code generation
        // self.command_types
        //     .insert(&command.name, command_type(command));

        // generate command function_pointers
        let mut function_pointer = definitions::FunctionPointer::new(&command.name);
        let fields = command
            .param
            .iter()
            .map(|field| make_cfield(field, FieldPurpose::FunctionParam));
        function_pointer.extend_fields(fields);
        function_pointer.set_return_type(make_return_ctype(&command.return_type));
        self.commands.push(&command.name, function_pointer);
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
        // TODO - this code should be removed soon
        // ensure all extensions get a macro generated even if no commands are included, since it will be expected when crating user extension lists
        let ex_name = extensions::ExtensionName::new(&extension.name, None);

        let kind = match extension.ty.as_ref().expect("error: expected extension type") {
            vkxml::ExtensionType::Instance => extensions::ExtensionKind::Instance,
            vkxml::ExtensionType::Device => extensions::ExtensionKind::Device,
        };

        self.extension_infos.contains_or_default(ex_name, extensions::ExtensionInfo::new(ex_name, kind));

        vkxml_visitor::visit_extension(extension, self);
    }
}

// when visiting a Feature, the generator will modify the last FeatureCommands added
// thus, you must add the FeatureCommands you want to modify as the last one
impl<'a> VisitFeature<'a> for Generator<'a> {
    fn visit_require_command_ref(&mut self, command_ref: &'a vkxml::NamedIdentifier) {
        let cmd_name = utils::VkTyName::new(command_ref.name.as_str());
        let fc = self
            .feature_commands
            .last_mut()
            .expect("error: no feature_command created");
        match self
            .command_types
            .get(&cmd_name)
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
    }

    fn visit_require_constant(&mut self, constant: &'a vkxml::ExtensionConstant) {
        let name = utils::VkTyName::new(constant.name.as_str());
        let val = constants::ConstValue2::from_vkxml(constant, constants::ConstantContext::GlobalConstant, None);
        let ty = val.type_of(&self.constants);

        self.constants.push(name, constants::Constant3::new(name, ty, val, None));
    }

    fn visit_require_enum_variant(&mut self, enum_def: vkxml_visitor::VkxmlExtensionEnum<'a>) {

    }
}

#[derive(Copy, Clone)]
enum FieldPurpose {
    StructField,
    FunctionParam,
}

fn make_cfield(field: &vkxml::Field, purpose: FieldPurpose) -> ctype::Cfield {
    let mut ctype = ctype::Ctype::new(&field.basetype);

    set_ctype_pointer_or_array(field, purpose, &mut ctype);

    let mut field = ctype::Cfield::new(
        field
            .name
            .as_ref()
            .expect("error: field in this position must have name")
            .as_str(),
        ctype,
    );

    match purpose {
        FieldPurpose::FunctionParam => {}
        FieldPurpose::StructField => field.set_public(),
    }

    field
}

fn make_return_ctype(field: &vkxml::Field) -> ctype::ReturnType {
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
    ctype: &mut ctype::Ctype,
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
                ctype.push_array(size);
            }
            FieldPurpose::FunctionParam => {
                if field.reference.is_some() {
                    // ctype.set_pointer_from_vkxml(&field.reference, field.is_const)
                    panic!("error: not sure yet how to handle static array with reference type");
                } else {
                    match field.is_const {
                        true => ctype.push_pointer(ctype::Pointer::Const),
                        false => ctype.push_pointer(ctype::Pointer::Mut),
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
        if name.contains("FlagBits") {
            return;
        }
        else {
            let name = utils::VkTyName::new(name);
            self.aliases.push(name, definitions::TypeDef::new(name, alias));
        }
    }
    fn visit_enum(&mut self, enm: &'a vk_parse::Type) {
        let enum_name = enm.name.as_deref().expect("error: enum with no name");
        let enum_def = definitions::Enum2::new(enum_name);
        self.definitions.enumerations.push(enum_def);
    }
    fn visit_command(&mut self, command: &'a vk_parse::CommandDefinition) {
        // get CommandType metadata for feature and extension code generation
        let name = utils::VkTyName::new(command.proto.name.as_str());
        self.command_types
            .insert(name, command_type(command));
    }
    fn visit_ex_enum(&mut self, spec: crate::vk_parse_visitor::VkParseEnumConstant<'a>) {
        let number = spec.number;
        let enm = spec.enm;
        let target = utils::VkTyName::new(spec.target.expect("error: enum with no target"));
        let is_alias = spec.is_alias;

        let kind;
        if spec.target.unwrap().contains("FlagBits") {
            kind = enumerations::EnumKind::BitFlags;
        }
        else {
            kind = enumerations::EnumKind::Normal;
        }

        let mut enum_variants = self
            .enum_variants
            .get_mut_or_default(target, enumerations::EnumVariants::new(target, kind));

        let name = enm.name.as_str();
        let val = constants::ConstValue2::from_vk_parse(spec, constants::ConstantContext::Enum, Some(target));
        let ty = ctype::Ctype::new("Self");

        enum_variants.push_variant_once(constants::Constant3::new(
            name,
            ty,
            val,
            Some(target),
        ));
    }
    fn visit_ex_require_node<I: Iterator<Item=&'a str>>(&mut self, info: crate::vk_parse_visitor::ExtensionInfo<'a, I>) {
        let ex_name = extensions::ExtensionName::new(info.name_parts.extension_name, info.name_parts.further_extended);

        self.vulkan_extension_names.push_extension(ex_name);

        let kind = match info.kind {
            "instance" => extensions::ExtensionKind::Instance,
            "device" => extensions::ExtensionKind::Device,
            _ => panic!("error: unexpected extension kind"),
        };

        let mut extension_commands = extensions::ExtensionInfo::new(ex_name, kind);

        match info.required {
            Some(req) => extension_commands.require(req),
            None => {}
        }

        let ex = self
            .extension_infos
            .push(ex_name, extension_commands);
    }
    fn visit_ex_cmd_ref(&mut self, cmd_name: &'a str, parts: &crate::vk_parse_visitor::VkParseExtensionParts<'a>) {
        let cmd_name = utils::VkTyName::new(cmd_name);
        let cmd_name = self.get_alias_or_name(cmd_name);
        let cmd_type = self
            .get_command_type(&cmd_name)
            .expect("error: feature identifies unknown command");

        let ex_name = extensions::ExtensionName::new(parts.extension_name, parts.further_extended);
        let ex = self
            .extension_infos
            .get_mut(ex_name)
            .expect("error: this should already exist from visiting the node");

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
    fn visit_struct_member(&mut self, part: crate::vk_parse_visitor::StructPart<'a>) {
        use crate::vk_parse_visitor::StructPartKind;
        let struct_name = utils::VkTyName::new(part.struct_name);
        let mut stct = self.definitions.structs.get_mut_or_default(struct_name, definitions::Struct2::new(part.struct_name));
        match part.part {
            StructPartKind::Code(code) => {
                let mut field = parse_field(code)
                    .expect("error: failed to parse struct member code");
                field.set_public();
                stct.push_field(field);
            }
            StructPartKind::Comment(comment) => {
                if comment.contains("non-normative") {
                    stct.non_normative();
                }
            }
        }
    }
    fn visit_constant(&mut self, spec: crate::vk_parse_visitor::VkParseEnumConstant<'a>) {
        let name = utils::VkTyName::new(spec.enm.name.as_str());
        let val = constants::ConstValue2::from_vk_parse(spec, constants::ConstantContext::GlobalConstant, None);
        let ty = val.type_of(&self.constants);

        self.constants.push(name, constants::Constant3::new(name, ty, val, None));
    }
}

fn parse_field(code: &str) -> Result<ctype::Cfield, ()> {
    use crate::simple_parse::*;

    let input = crate::simple_parse::TokenIter::new(code);

    let (input, c) = opt(tag("const"))(input)?;
    let (input, _) = opt(tag("struct"))(input)?;
    let (input, bt) = token()(input)?;
    let (input, p) = opt(tag("*"))(input)?;

    let mut ty = ctype::Ctype::new(bt);

    if p.is_some() && c.is_some() {
        ty.push_pointer(ctype::Pointer::Const);
    }
    else if p.is_some() {
        ty.push_pointer(ctype::Pointer::Mut);
    }

    let (input, _) = repeat(
        input,
        followed(opt(tag("const")), tag("*")),
        |(c, _)| {
            if c.is_some() {
                ty.push_pointer(ctype::Pointer::Const);
            }
            else {
                ty.push_pointer(ctype::Pointer::Mut);
            }
        }
    )?;

    let (input, name) = token()(input)?;

    let (input, bit_width) = opt(followed(tag(":"), token()))(input)?;

    if let Some((_colon, bit_width)) = bit_width {
        let bit_width: u8 = str::parse(bit_width).expect("error: can't parse bit_width");
        ty.set_bit_width(bit_width);
    }

    let (mut input, _) = repeat(
        input,
        delimited(tag("["), token(), tag("]")),
        |(_, size, _)| ty.push_array(size)
    )?;

    // this is expected to consume all tokens
    if input.next().is_some() {
        Err(())
    }
    else {
        Ok(ctype::Cfield::new(name, ty))
    }
}

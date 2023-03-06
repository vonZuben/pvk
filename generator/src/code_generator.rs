use std::collections::{HashMap, HashSet};

use krs_quote::krs_quote;

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
    // in order to avoid external ".h" files and c libraries, we do not generate the external types and just treat them generically
    // to achieve this, we treat such types as generic, and a user needs to determine the correct type
    generic_types: HashSet<utils::VkTyName>,

    // code generation
    definitions: definitions::Definitions2,
    constants: VecMap<utils::VkTyName, constants::Constant3<'a>>,
    enum_variants: utils::VecMap<utils::VkTyName, enumerations::EnumVariants<'a>>,
    commands: commands::Commands2,
    feature_collection: features::FeatureCollection,
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

        let commands_trait = crate::traits::VulkanCommand;
        let version_trait = crate::traits::VulkanVersion;
        let extension_traits = crate::traits::VulkanExtension;

        let definitions = &self.definitions;
        let constants = self.constants.iter();
        let enum_variants = self.enum_variants.iter();
        let commands = &self.commands;
        let feature_collection = &self.feature_collection;
        let extension_commands = self.extension_infos.iter();
        let aliases = self.aliases.iter();

        krs_quote!(
            {@static_code}
            {@commands_trait}
            {@version_trait}
            {@extension_traits}
            {@definitions}
            {@* {@constants}}
            {@* {@enum_variants}}
            {@commands}
            {@* {@aliases}}
            {@feature_collection}
            {@* {@extension_commands}}
        ).to_string()
    }

    /// internally I call it static code (see [StaticCode](crate::static_code::StaticCode))
    /// externally, this is utility code that other generated code relies on
    pub fn util_code(&self) -> String {
        let static_code = crate::static_code::StaticCode;
        krs_quote!({@static_code}).to_string()
    }

    pub fn vulkan_traits(&self) -> String {
        let commands_trait = crate::traits::VulkanCommand;
        let version_trait = crate::traits::VulkanVersion;
        let extension_traits = crate::traits::VulkanExtension;
        krs_quote!(
            {@commands_trait}
            {@version_trait}
            {@extension_traits}
        ).to_string()
    }

    pub fn c_type_defs(&self) -> String {
        let c_type_defs = &self.definitions.type_defs;
        krs_quote!({@* {@c_type_defs} }).to_string()
    }

    pub fn bitmasks(&self) -> String {
        let bitmasks = &self.definitions.bitmasks;
        krs_quote!({@* {@bitmasks} }).to_string()
    }

    pub fn structs(&self) -> String {
        let structs = self.definitions.structs.iter();
        krs_quote!({@* {@structs} }).to_string()
    }

    pub fn unions(&self) -> String {
        let unions = &self.definitions.unions;
        krs_quote!({@* {@unions} }).to_string()
    }

    pub fn handles(&self) -> String {
        let handles = &self.definitions.handles;
        krs_quote!({@* {@handles} }).to_string()
    }

    pub fn enumerations(&self) -> String {
        let enumerations = &self.definitions.enumerations;
        krs_quote!({@* {@enumerations} }).to_string()
    }

    pub fn enum_variants(&self) -> String {
        let enum_variants = self.enum_variants.iter();
        krs_quote!({@* {@enum_variants} }).to_string()
    }

    pub fn function_pointers(&self) -> String {
        let function_pointers = &self.definitions.function_pointers;
        krs_quote!({@* {@function_pointers} }).to_string()
    }

    pub fn constants(&self) -> String {
        let constants = self.constants.iter();
        krs_quote!({@* {@constants} }).to_string()
    }

    pub fn commands(&self) -> String {
        let commands = &self.commands;
        krs_quote!({@commands}).to_string()
    }

    pub fn versions(&self) -> String {
        let versions = &self.feature_collection;
        krs_quote!({@versions}).to_string()
    }

    pub fn extensions(&self) -> String {
        let extensions = self.extension_infos.iter();
        krs_quote!({@* {@extensions} }).to_string()
    }

    pub fn aliases(&self) -> String {
        let aliases = self.aliases.iter();
        krs_quote!({@* {@aliases} }).to_string()
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
    fn visit_command(&mut self, def_wrapper: crate::vk_parse_visitor::CommandDefWrapper<'a>) {
        let def = def_wrapper.def;
        // get CommandType metadata for feature and extension code generation
        let command_name = utils::VkTyName::new(def.name);
        self.command_types
            .insert(command_name, command_type(&def_wrapper.raw));

        // generate actual command
        let mut function_pointer = definitions::FunctionPointer::new(command_name);
        let fields = def.params.into_iter().map(|mut field| {
            if self.generic_types.contains(&field.ty.name()) {
                field.ty.set_external();
            }
            field
        });
        function_pointer.extend_fields(fields);
        function_pointer.set_return_type(def.return_type);
        self.commands.push(command_name, function_pointer);
    }
    fn visit_ex_enum(&mut self, spec: crate::vk_parse_visitor::VkParseEnumConstant<'a>) {
        let enm = spec.enm;
        let target = utils::VkTyName::new(spec.target.expect("error: enum with no target"));

        let kind;
        if spec.target.unwrap().contains("FlagBits") {
            kind = enumerations::EnumKind::BitFlags;
        }
        else {
            kind = enumerations::EnumKind::Normal;
        }

        let enum_variants = self
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

        self.extension_infos.push(ex_name, extension_commands);
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
        }
    }
    fn visit_struct_def(&mut self, def: crate::vk_parse_visitor::StructDef<'a>) {
        let struct_name = utils::VkTyName::new(def.name);
        let stct = self.definitions.structs.get_mut_or_default(struct_name, definitions::Struct2::new(def.name));
        let mut generic_struct = false;
        for member in def.members {
            use crate::vk_parse_visitor::MemberKind;
            match member {
                MemberKind::Member(mut field) => {
                    field.set_public();
                    if self.generic_types.contains(&field.ty.name()) {
                        field.ty.set_external();
                        generic_struct = true;
                    }
                    stct.push_field(field);
                }
                MemberKind::Comment(comment) => {
                    if comment.contains("non-normative") {
                        stct.non_normative();
                    }
                }
            }
        }
        if generic_struct {
            self.generic_types.insert(struct_name);
        }
    }
    fn visit_union(&mut self, def: crate::vk_parse_visitor::UnionDef<'a>) {
        let mut uni = definitions::Union::new(def.name);
        let fields = def
            .members
            .filter_map(|member| match member {
                crate::vk_parse_visitor::MemberKind::Comment(_) => {
                    None
                }
                crate::vk_parse_visitor::MemberKind::Member(mut member) => {
                    member.set_public();
                    Some(member)
                }
            });
        uni.extend_fields(fields);
        self.definitions.unions.push(uni);
    }
    fn visit_constant(&mut self, spec: crate::vk_parse_visitor::VkParseEnumConstant<'a>) {
        let name = utils::VkTyName::new(spec.enm.name.as_str());
        let val = constants::ConstValue2::from_vk_parse(spec, constants::ConstantContext::GlobalConstant, None);
        let ty = val.type_of(&self.constants);

        self.constants.push(name, constants::Constant3::new(name, ty, val, None));
    }
    fn visit_basetype(&mut self, basetype: crate::vk_parse_visitor::VkBasetype<'a>) {
        let type_def = definitions::TypeDef::new(basetype.name, basetype.ty);
        self.definitions.type_defs.push(type_def);
    }
    fn visit_bitmask(&mut self, basetype: crate::vk_parse_visitor::VkBasetype<'a>) {
        let name = utils::VkTyName::new(basetype.name);
        assert!(name.contains("Flags"));
        self.enum_variants.contains_or_default(name, enumerations::EnumVariants::new(name, enumerations::EnumKind::BitFlags));
        let bitmask = definitions::Bitmask::new(name, basetype.ty);
        self.definitions.bitmasks.push(bitmask);
    }
    fn visit_handle(&mut self, def: crate::vk_parse_visitor::HandleDef<'a>) {
        let dispatch = match def.kind{
            crate::vk_parse_visitor::HandleKind::Dispatchable => true,
            crate::vk_parse_visitor::HandleKind::NonDispatchable => false,
        };
        let handle = definitions::Handle2::new(def.name, dispatch);
        self.definitions.handles.push(handle);
    }
    fn visit_fptr(&mut self, def: crate::vk_parse_visitor::FptrDef<'a>) {
        if def.name == "PFN_vkVoidFunction" {
            return; // This is a special case that has a unique definition in the generated code
        }
        let mut fptr = definitions::FunctionPointer::new(def.name);
        fptr.extend_fields(def.params);
        fptr.set_return_type(def.return_type);
        self.definitions.function_pointers.push(fptr);
    }
    fn visit_feature_name(&mut self, _name: utils::VkTyName) {

    }
    fn visit_require_command(&mut self, def: crate::vk_parse_visitor::CommandRef<'a>) {
        let cmd_name = utils::VkTyName::new(def.name);
        let fcc = &mut self.feature_collection;
        match self
            .command_types
            .get(&cmd_name)
            .expect("error: feature identifies unknown command")
        {
            CommandType::Instance => fcc.modify_with(def.version, |fc| fc.push_instance_command(cmd_name)),
            CommandType::Device => fcc.modify_with(def.version, |fc| fc.push_device_command(cmd_name)),
            CommandType::Entry => fcc.modify_with(def.version, |fc| fc.push_entry_command(cmd_name)),
            CommandType::Static => {}
        }
    }
    fn visit_remove_command(&mut self, def: crate::vk_parse_visitor::CommandRef<'a>) {
        self.feature_collection.modify_with(def.version, |fc| fc.remove_command(def.name));
    }
    fn visit_external_type(&mut self, name: crate::utils::VkTyName) {
        self.generic_types.insert(name);
    }
}

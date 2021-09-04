use std::collections::HashMap;

use crate::vkxml_visitor::VisitVkxml;

use crate::cfield;
use crate::ctype;
use crate::definitions;
use crate::constants;
use crate::enumerations;
use crate::commands;
use crate::features;
use crate::extensions;

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
        _ =>
            match command.param[0].basetype.as_str() {
                "VkDevice" | "VkCommandBuffer" | "VkQueue" => CommandType::Device,
                "VkInstance" | "VkPhysicalDevice" => CommandType::Instance,
                _ => CommandType::Entry,
            }
    }
}

struct Generator<'a> {
    // metadata
    // when generating commands to load per feature, we use this to determine command_types
    command_types: HashMap<&'a str, CommandType>,

    // code generation
    definitions: definitions::Definitions2<'a>,
    constants: Vec<constants::Constant2<'a>>,
    enum_variants: Vec<enumerations::EnumVariants<'a>>,
    commands: commands::Commands2<'a>,
    feature_commands: Vec<features::FeatureCommands<'a>>,
    extension_commands: Vec<extensions::ExtensionCommands<'a>>,
}

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
        self.constants.push(constants::make_vulkan_const_from_vkxml(constant));
    }

    fn visit_enum_variants(&mut self, enumeration: &'a vkxml::Enumeration) {
        let mut enum_variant = enumerations::EnumVariants::new(&enumeration.name);
        let variants = enumeration.elements.iter().filter_map(|enumeration_element| {
            use vkxml::EnumerationElement;
            match enumeration_element {
                EnumerationElement::Notation(_) => None,
                EnumerationElement::UnusedRange(_) => None,
                EnumerationElement::Enum(constant) => {
                    Some(enumerations::make_enumeration_variant_from_vkxml(constant))
                }
            }
        });
        enum_variant.extend_variants(variants);
        self.enum_variants.push(enum_variant);
    }

    fn visit_command(&mut self, command: &'a vkxml::Command) {
        // get CommandType metadata for feature code generation
        self.command_types.insert(&command.name, command_type(command));

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
        let mut fc = match self.feature_commands.last() {
            Some(previous_feature) => previous_feature.as_new_version(&feature.name),
            None => features::FeatureCommands::new(&feature.name),
        };

        for feature_element in feature.elements.iter() {
            use vkxml::FeatureElement;
            match feature_element {
                FeatureElement::Notation(_) => {}
                FeatureElement::Require(feature_spec) => {
                    for feature_reference in feature_spec.elements.iter() {
                        use vkxml::FeatureReference;
                        match feature_reference {
                            // nothing we need here
                            FeatureReference::Notation(_) => {}
                            // simply indicates definitions that should exist but we always
                            // generate everything
                            FeatureReference::DefinitionReference(_) => {}
                            // should include some definitions of some Extension promoted enums
                            // but vkxml does not include so we need to parse the more raw xml
                            // from vk-parse to get these
                            FeatureReference::EnumeratorReference(_) => {}
                            // indicates the commands we should load for the specified version
                            FeatureReference::CommandReference(cmd) => {
                                match self.command_types.get(cmd.name.as_str()).expect("error: feature identifies unknown command") {
                                    CommandType::Instance => fc.push_instance_command(&cmd.name),
                                    CommandType::Device => fc.push_device_command(&cmd.name),
                                    CommandType::Entry => fc.push_entry_command(&cmd.name),
                                    CommandType::Static => {}
                                    CommandType::DoNotGenerate => {}
                                }
                            }
                        }
                    }
                }
                FeatureElement::Remove(feature_spec) => {
                    for feature_reference in feature_spec.elements.iter() {
                        use vkxml::FeatureReference;
                        match feature_reference {
                            FeatureReference::Notation(_) => {}
                            // simply indicates definitions that should *not* exist but we always
                            // generate everything
                            FeatureReference::DefinitionReference(_) => {}
                            // similar to DefinitionReference but for enumerations
                            FeatureReference::EnumeratorReference(_) => {}
                            // indicates the commands we should *not* load for the specified version
                            FeatureReference::CommandReference(cmd) => {
                                fc.remove_command(&cmd.name);
                            }
                        }
                    }
                }
            }
        }

        self.feature_commands.push(fc);
    }

    fn visit_extension(&mut self, extension: &'a vkxml::Extension) {
        let mut ex = extensions::ExtensionCommands::new(&extension.name);

        for extension_element in extension.elements.iter() {
            use vkxml::ExtensionElement;
            match extension_element {
                ExtensionElement::Notation(_) => {}
                ExtensionElement::Require(extension_spec) => {
                    for ex_spec_element in extension_spec.elements.iter() {
                        use vkxml::ExtensionSpecificationElement;
                        match ex_spec_element {
                            ExtensionSpecificationElement::Notation(_) => {}
                            // simply indicates definitions that should exist but we always
                            // generate everything
                            ExtensionSpecificationElement::DefinitionReference(_) => {}
                            ExtensionSpecificationElement::CommandReference(cmd) => {
                                match self.command_types.get(cmd.name.as_str()).expect("error: feature identifies unknown command") {
                                    CommandType::Instance => ex.push_instance_command(&cmd.name),
                                    CommandType::Device => ex.push_device_command(&cmd.name),
                                    CommandType::Entry => panic!("error: entry level command added by extension not handled"),
                                    CommandType::Static => panic!("error: static level command added by extension not handled"),
                                    CommandType::DoNotGenerate => {}
                                }
                            }
                            // similar to DefinitionReference but for enumerations
                            ExtensionSpecificationElement::EnumeratorReference(_) => {}
                            ExtensionSpecificationElement::Constant(constant) => {
                                self.constants.push(extensions::make_extention_constant_from_vkxmk(constant));
                            }
                            ExtensionSpecificationElement::Enum(enum_ex) => {}
                        }
                    }
                }
                ExtensionElement::Remove(_) => panic!("error: extension should not remove anything"),
            }
        }

        self.extension_commands.push(ex);
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
    }
    else {
        let mut ctype = ctype::Ctype::new(&field.basetype);
        // for now, assuming return type cannot be a pointer to a static size array
        assert!(!(matches!(field.reference, Some(_)) && matches!(field.array, Some(vkxml::ArrayType::Static))));
        set_ctype_pointer_or_array(field, FieldPurpose::FunctionParam, &mut ctype);
        ctype.into()
    }
}

fn set_ctype_pointer_or_array<'a>(field: &'a vkxml::Field, purpose: FieldPurpose, ctype: &mut ctype::Ctype<'a>) {
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

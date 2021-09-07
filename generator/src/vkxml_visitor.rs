pub trait VisitVkxml<'a> {
    fn visit_type_def(&mut self, type_def: &'a vkxml::Typedef) {}
    fn visit_bitmask(&mut self, bitmask: &'a vkxml::Bitmask) {}
    fn visit_struct(&mut self, struct_def: &'a vkxml::Struct) {}
    fn visit_union(&mut self, union_def: &'a vkxml::Union) {}
    fn visit_handle(&mut self, handle: &'a vkxml::Handle) {}
    fn visit_enum_def(&mut self, enum_def: &'a vkxml::EnumerationDeclaration) {}
    fn visit_function_pointer(&mut self, function_pointer: &'a vkxml::FunctionPointer) {}
    fn visit_constant(&mut self, constant: &'a vkxml::Constant) {}
    fn visit_enum_variants(&mut self, enumeration: &'a vkxml::Enumeration) {}
    fn visit_command(&mut self, command: &'a vkxml::Command) {}
    fn visit_feature(&mut self, feature: &'a vkxml::Feature) {}
    fn visit_extension(&mut self, extension: &'a vkxml::Extension) {}
}

pub fn visit_vkxml<'a>(registry: &'a vkxml::Registry, visitor: &mut impl VisitVkxml<'a>) {
    for reg_child in registry.elements.iter() {
        use vkxml::RegistryElement;
        match reg_child {
            RegistryElement::Notation(_) => {}
            RegistryElement::VendorIds(_) => {}
            RegistryElement::Tags(_) => {}
            RegistryElement::Definitions(definitions ) => {
                for definition_element in definitions.elements.iter() {
                    use vkxml::DefinitionsElement;
                    match definition_element {
                        DefinitionsElement::Notation(_) => {}
                        DefinitionsElement::Include(_)=> {}
                        DefinitionsElement::Typedef(type_def) => visitor.visit_type_def(type_def),
                        DefinitionsElement::Reference(_) => {}
                        DefinitionsElement::Bitmask(bitmask) => visitor.visit_bitmask(bitmask),
                        DefinitionsElement::Struct(struct_def) => visitor.visit_struct(struct_def),
                        DefinitionsElement::Union(union_def) => visitor.visit_union(union_def),
                        DefinitionsElement::Define(_) => {}
                        DefinitionsElement::Handle(handle) => visitor.visit_handle(handle),
                        DefinitionsElement::Enumeration(enum_def) => visitor.visit_enum_def(enum_def),
                        DefinitionsElement::FuncPtr(function_pointer) => visitor.visit_function_pointer(function_pointer),
                    }
                }
            }
            RegistryElement::Constants(constants) => {
                for constant in constants.elements.iter() {
                    visitor.visit_constant(constant);
                }
            }
            RegistryElement::Enums(enums) => {
                for enum_element in enums.elements.iter() {
                    use vkxml::EnumsElement;
                    match enum_element {
                        EnumsElement::Notation(_) => {}
                        EnumsElement::Enumeration(enumeration) => visitor.visit_enum_variants(enumeration),
                    }
                }
            }
            RegistryElement::Commands(commands) => {
                for command in commands.elements.iter() {
                    visitor.visit_command(command);
                }
            }
            RegistryElement::Features(features) => {
                for feature in features.elements.iter() {
                    visitor.visit_feature(feature);
                }
            }
            RegistryElement::Extensions(extensions) => {
                for extension in extensions.elements.iter() {
                    visitor.visit_extension(extension);
                }
            }
        }
    }
}

pub trait VisitFeature<'a> {
    fn visit_require_command_ref(&mut self, command_ref: &'a vkxml::NamedIdentifier) {}
    fn visit_remove_command_ref(&mut self, command_ref: &'a vkxml::NamedIdentifier) {}
}

pub fn visit_feature<'a>(feature: &'a vkxml::Feature, visitor: &mut impl VisitFeature<'a>) {
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
                            visitor.visit_require_command_ref(cmd);
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
                            visitor.visit_remove_command_ref(cmd);
                        }
                    }
                }
            }
        }
    }
}

pub struct VkxmlExtensionEnum<'a> {
    pub enum_extension: &'a vkxml::ExtensionEnum,
    pub number: i32,
}

pub trait VisitExtension<'a> {
    fn visit_require_command_ref(&mut self, command_ref: &'a vkxml::NamedIdentifier) {}
    fn visit_require_constant(&mut self, constant: &'a vkxml::ExtensionConstant) {}
    fn visit_require_enum_variant(&mut self, enum_def: VkxmlExtensionEnum<'a>) {}
}

pub fn visit_extension<'a>(extension: &'a vkxml::Extension, visitor: &mut impl VisitExtension<'a>) {
    if extension.disabled {
        return;
    }
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
                            visitor.visit_require_command_ref(cmd);
                        }
                        // similar to DefinitionReference but for enumerations
                        ExtensionSpecificationElement::EnumeratorReference(_) => {}
                        ExtensionSpecificationElement::Constant(constant) => {
                            visitor.visit_require_constant(constant);
                        }
                        ExtensionSpecificationElement::Enum(enum_def) => {
                            visitor.visit_require_enum_variant(VkxmlExtensionEnum { enum_extension: enum_def, number: extension.number });
                        }
                    }
                }
            }
            ExtensionElement::Remove(_) => panic!("error: extension should not remove anything"),
        }
    }
}
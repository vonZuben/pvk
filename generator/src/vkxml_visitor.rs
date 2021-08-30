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
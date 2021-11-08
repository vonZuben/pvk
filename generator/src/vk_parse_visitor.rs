pub trait VisitVkParse<'a> {
    fn visit_alias(&mut self, name: &'a str, alias: &'a str) {}
    fn visit_enum(&mut self, enm: &'a vk_parse::Type) {}
    fn visit_command(&mut self, command: &'a vk_parse::CommandDefinition) {}
    fn visit_ex_enum(&mut self, ex: VkParseEnumConstant<'a>) {}
    fn visit_ex_require_node(&mut self, parts: &VkParseExtensionParts<'a>) {}
    fn visit_ex_cmd_ref(&mut self, cmd_name: &'a str, parts: &VkParseExtensionParts<'a>) {}
    fn visit_struct_member(&mut self, member: StructPart<'a>) {}
    fn visit_constant(&mut self, spec: VkParseEnumConstant<'a>) {}
}

pub fn visit_vk_parse<'a>(registry: &'a vk_parse::Registry, visitor: &mut impl VisitVkParse<'a>) {
    for reg_child in registry.0.iter() {
        use vk_parse::RegistryChild::*;
        match reg_child {
            Comment(_) => {}
            VendorIds(_) => {}
            Platforms(_) => {}
            Tags(_) => {}
            Types(ty) => {
                for type_child in ty.children.iter() {
                    use vk_parse::TypesChild::*;
                    match type_child {
                        Comment(_) => {}
                        Type(ty) => {
                            if ty.name.is_some() && ty.alias.is_some() {
                                visitor.visit_alias(
                                    ty.name.as_ref().unwrap(),
                                    ty.alias.as_ref().unwrap(),
                                );
                            }
                            else {
                                match ty.category.as_ref().map(|s|s.as_str()) {
                                    Some("enum") => {
                                        if ty.name.as_ref().expect("error: enum with no name").contains("FlagBits") {
                                            continue;
                                        }
                                        visitor.visit_enum(ty);
                                    }
                                    Some("struct") => {
                                        // print!("");
                                        match ty.spec {
                                            vk_parse::TypeSpec::Code(ref ty_code) => {
                                                // eprintln!("TCODE: {:?}", ty_code);
                                            }
                                            vk_parse::TypeSpec::Members(ref members) => {
                                                for member in members {
                                                    match member {
                                                        vk_parse::TypeMember::Comment(cmnt) => {
                                                            visitor.visit_struct_member(StructPart {
                                                                struct_name: ty.name.as_deref().expect("error: struct with no name"),
                                                                part: StructPartKind::Comment(cmnt.as_str()),
                                                            });
                                                        }
                                                        vk_parse::TypeMember::Definition(def) => {
                                                            visitor.visit_struct_member(StructPart {
                                                                struct_name: ty.name.as_deref().expect("error: struct with no name"),
                                                                part: StructPartKind::Code(def.code.as_str()),
                                                            });
                                                        }
                                                        _ => panic!("error: unexpected TypeMember node"),
                                                    }
                                                }
                                            }
                                            vk_parse::TypeSpec::None => {}
                                            _ => panic!("error: unhandled TypSpec node"),
                                        }
                                    }
                                    Some(_) | None => {}
                                }
                            }
                        }
                        _ => panic!("unexpected TypeChild"),
                    }
                }
            }
            Enums(enms) => {
                match enms.kind.as_deref() {
                    None => { // API Constant or the like
                        for enum_child in enms.children.iter() {
                            use vk_parse::EnumsChild;
                            match enum_child {
                                EnumsChild::Enum(ref enm) => {
                                    visitor.visit_constant(VkParseEnumConstant {
                                        number: None,
                                        enm,
                                        target: None,
                                        is_alias: false,
                                    });
                                }
                                EnumsChild::Comment(_) => {}
                                EnumsChild::Unused(_) => {}
                                _ => panic!("error: unexpected EnumsChild"),
                            }
                        }
                    }
                    Some("enum" | "bitmask") => { // enum variants
                        for enum_child in enms.children.iter() {
                            use vk_parse::EnumsChild;
                            match enum_child {
                                EnumsChild::Enum(ref enm) => {
                                    visitor.visit_ex_enum(VkParseEnumConstant {
                                        number: None,
                                        enm,
                                        target: enms.name.as_deref(),
                                        is_alias: enm.spec.is_alias(),
                                    });
                                }
                                EnumsChild::Comment(_) => {}
                                EnumsChild::Unused(_) => {}
                                _ => panic!("error: unexpected EnumsChild"),
                            }
                        }
                    }
                    Some(_x) => {}
                }
            }
            Commands(commands) => {
                for command in commands.children.iter() {
                    use vk_parse::Command::*;
                    match command {
                        Alias { name, alias } => visitor.visit_alias(name, alias),
                        Definition(cmd_def) => {
                            visitor.visit_command(cmd_def);
                        }
                        _ => panic!("unexpected Command node"),
                    }
                }
            }
            Feature(feature) => {
                for feature_child in feature.children.iter() {
                    use vk_parse::ExtensionChild::*;
                    match feature_child {
                        Require {
                            api,
                            profile,
                            extension,
                            feature,
                            comment,
                            items,
                        } => {
                            for item in items.iter() {
                                use vk_parse::InterfaceItem::*;
                                match item {
                                    Comment(_) => {}
                                    Type { name, comment } => {}
                                    Enum(enm) => {
                                        let extends = enm.spec.extends();
                                        if extends.is_some() {
                                            visitor.visit_ex_enum(VkParseEnumConstant {
                                                number: None,
                                                enm,
                                                target: extends,
                                                is_alias: enm.spec.is_alias(),
                                            });
                                        }
                                    }
                                    Command { name, comment } => {}
                                    _ => panic!("unexpected InterfaceItem node"),
                                }
                            }
                        }
                        Remove {
                            api,
                            profile,
                            comment,
                            items,
                        } => {}
                        _ => panic!("unexpected Feature node"),
                    }
                }
            }
            Extensions(extensions) => {
                for extension in extensions.children.iter() {
                    if extension.supported.as_ref().map(String::as_str) == Some("disabled") {
                        continue;
                    }
                    for ex_child in extension.children.iter() {
                        use vk_parse::ExtensionChild::*;
                        match ex_child {
                            Require {
                                api,
                                profile,
                                extension: required_extension,
                                feature: requiered_feature,
                                comment,
                                items,
                            } => {
                                // assuming for now that feature and extension additions are exclusive
                                let further_extended = match (requiered_feature, required_extension) {
                                    (Some(feature), None) => Some(feature.as_str()),
                                    (None, Some(extension)) => Some(extension.as_str()),
                                    (None, None) => None,
                                    _ => panic!("error: not expecting feature and exteions additions at the same time"),
                                };
                                let parts = VkParseExtensionParts {
                                    extension_name: &extension.name,
                                    further_extended,
                                };
                                visitor.visit_ex_require_node(&parts);

                                for item in items.iter() {
                                    use vk_parse::InterfaceItem::*;
                                    match item {
                                        Comment(_) => {}
                                        Type { name, comment } => {}
                                        Enum(enm) => {
                                            let extends = enm.spec.extends();
                                            if extends.is_some() {
                                                visitor.visit_ex_enum(VkParseEnumConstant {
                                                    number: extension.number,
                                                    enm,
                                                    target: extends,
                                                    is_alias: enm.spec.is_alias(),
                                                });
                                            }
                                        }
                                        Command { name, comment } => {
                                            visitor.visit_ex_cmd_ref(name, &parts);
                                        }
                                        _ => panic!("unexpected InterfaceChild node"),
                                    }
                                }
                            }
                            Remove {
                                api,
                                profile,
                                comment,
                                items,
                            } => panic!("error: extension should not remove anything"),
                            _ => panic!("unexpected ExtensionChild node"),
                        }
                    }
                }
            }
            SpirvExtensions(_) => {}
            SpirvCapabilities(_) => {}
            _ => panic!("unexpected node"),
        }
    }
}

trait EnumSpecEx {
    fn is_alias(&self) -> bool;
    fn extends(&self) -> Option<&str>;
}

impl EnumSpecEx for vk_parse::EnumSpec {
    fn is_alias(&self) -> bool {
        matches!(self, Self::Alias { .. })
    }
    // when this is None, it is a sign that this defines a new constant
    fn extends(&self) -> Option<&str> {
        match self {
            Self::Alias { extends, .. } => extends.as_deref(),
            Self::Offset { extends, .. } => Some(extends),
            Self::Bitpos { extends, .. } => extends.as_deref(),
            Self::Value { extends, .. } => extends.as_deref(),
            Self::None => None,
            _ => panic!("unexpected EnumSpec node"),
        }
    }
}

// when defining enum value, target should be set
// when defining a constant, no target
// when defining an enum value from an extension, there needs to be a number
pub struct VkParseEnumConstant<'a> {
    pub number: Option<i64>,
    pub enm: &'a vk_parse::Enum,
    pub target: Option<&'a str>,
    pub is_alias: bool,
}

pub struct VkParseExtensionParts<'a> {
    pub extension_name: &'a str, 
    pub further_extended: Option<&'a str>, 
}

pub struct StructPart<'a> {
    pub struct_name: &'a str,
    pub part: StructPartKind<'a>,
}

pub enum StructPartKind<'a> {
    Code(&'a str),
    Comment(&'a str),
}
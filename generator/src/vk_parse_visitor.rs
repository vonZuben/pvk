pub trait VisitVkParse<'a> {
    fn visit_alias(&mut self, name: &'a str, alias: &'a str) {}
    fn visit_ex_enum(&mut self, ex: VkParseEnumConstantExtension<'a>) {}
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
                        }
                    }
                }
            }
            Enums(_) => {}
            Commands(commands) => {
                for command in commands.children.iter() {
                    use vk_parse::Command::*;
                    match command {
                        Alias { name, alias } => visitor.visit_alias(name, alias),
                        Definition(_) => {}
                    }
                }
            }
            Feature(_) => {}
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
                                            match enumspec_kind(&enm.spec) {
                                                EnumSpecKind::ExEnum(target, is_alias) => {
                                                    visitor.visit_ex_enum(VkParseEnumConstantExtension {
                                                        number: extension.number.expect("error: enum extension must have a number"),
                                                        enm,
                                                        target,
                                                        is_alias,
                                                    });
                                                }
                                                EnumSpecKind::Constant => {}
                                                EnumSpecKind::EnumeratorRef => {}
                                                _=>{}
                                            }
                                        }
                                        Command { name, comment } => {}
                                    }
                                }
                            }
                            Remove {
                                api,
                                profile,
                                comment,
                                items,
                            } => panic!("error: extension should not remove anything"),
                        }
                    }
                }
            }
        }
    }
}

enum EnumSpecKind<'a> {
    ExEnum(&'a str, bool),
    Constant,
    EnumeratorRef,
}

impl<'a> EnumSpecKind<'a> {
    fn from_extends(extends: &'a Option<String>, is_alias: bool) -> Self {
        match extends {
            Some(target) => Self::ExEnum(target, is_alias),
            None => Self::Constant,
        }
    }
}

fn enumspec_kind<'a>(enum_spec: &'a vk_parse::EnumSpec) -> EnumSpecKind<'a> {
    use vk_parse::EnumSpec::*;
    match enum_spec {
        Alias { ref extends, .. } => EnumSpecKind::from_extends(extends, true),
        Offset { extends: target, .. } => EnumSpecKind::ExEnum(target, false),
        Bitpos { ref extends, .. } => EnumSpecKind::from_extends(extends, false),
        Value { ref extends, .. } => EnumSpecKind::from_extends(extends, false),
        None => EnumSpecKind::EnumeratorRef,
    }
}

pub struct VkParseEnumConstantExtension<'a> {
    pub number: i64,
    pub enm: &'a vk_parse::Enum,
    pub target: &'a str,
    pub is_alias: bool,
}
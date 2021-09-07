pub trait VisitVkParse<'a> {
    fn visit_alias(&mut self, name: &'a str, alias: &'a str) {}
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
            Extensions(_) => {}
        }
    }
}
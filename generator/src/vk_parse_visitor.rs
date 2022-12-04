use crate::ctype;

pub trait VisitVkParse<'a> {
    fn visit_alias(&mut self, name: &'a str, alias: &'a str);
    fn visit_enum(&mut self, enm: &'a vk_parse::Type);
    fn visit_command(&mut self, def_wrapper: CommandDefWrapper<'a>);
    fn visit_ex_enum(&mut self, spec: VkParseEnumConstant<'a>);
    fn visit_ex_require_node<I: Iterator<Item=&'a str>>(&mut self, info: ExtensionInfo<'a, I>);
    fn visit_ex_cmd_ref(&mut self, cmd_name: &'a str, parts: &VkParseExtensionParts<'a>);
    fn visit_struct_def(&mut self, def: StructDef<'a>);
    fn visit_constant(&mut self, spec: VkParseEnumConstant<'a>);
    fn visit_basetype(&mut self, basetype: VkBastetype<'a>);
    fn visit_bitmask(&mut self, basetype: VkBastetype<'a>);
    fn visit_union(&mut self, def: UnionDef<'a>);
    fn visit_handle(&mut self, def: HandleDef<'a>);
    fn visit_fptr(&mut self, def: FptrDef<'a>);
    fn visit_feature_name(&mut self, name: crate::utils::VkTyName);
    fn visit_require_command(&mut self, def: CommandRef<'a>);
    fn visit_remove_command(&mut self, def: CommandRef<'a>);
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
                                        match ty.spec {
                                            vk_parse::TypeSpec::Code(ref _ty_code) => {}
                                            vk_parse::TypeSpec::Members(ref members) => {
                                                visitor.visit_struct_def(StructDef {
                                                    name: ty.name.as_deref().expect("error: struct with no name"),
                                                    members: Members { members: members.iter() },
                                                });
                                            }
                                            vk_parse::TypeSpec::None => {}
                                            _ => panic!("error: unhandled struct TypSpec node"),
                                        }
                                    }
                                    Some("basetype") => {
                                        match ty.spec {
                                            vk_parse::TypeSpec::Code(ref code) => {
                                                let basetype = parse_basetype(&code.code).expect("error: can't parse basetype in vk_parese");
                                                visitor.visit_basetype(basetype);
                                            }
                                            _ => panic!("unexpected basetype spec"),
                                        }
                                    }
                                    Some("bitmask") => {
                                        match ty.spec {
                                            vk_parse::TypeSpec::Code(ref code) => {
                                                let basetype = parse_basetype(&code.code).expect("error: can't parse bitmask in vk_parese");
                                                visitor.visit_bitmask(basetype);
                                            }
                                            _ => panic!("unexpected bitmask spec"),
                                        }
                                    }
                                    Some("union") => {
                                        match ty.spec {
                                            vk_parse::TypeSpec::Code(ref _ty_code) => {}
                                            vk_parse::TypeSpec::Members(ref members) => {
                                                visitor.visit_union(UnionDef {
                                                    name: ty.name.as_deref().expect("error: union with no name"),
                                                    members: Members { members: members.iter() },
                                                });
                                            }
                                            vk_parse::TypeSpec::None => {}
                                            _ => panic!("error: unhandled union TypSpec node"),
                                        }
                                    }
                                    Some("handle") => {
                                        match ty.spec {
                                            vk_parse::TypeSpec::Code(ref ty_code) => {
                                                let handle_def = parse_handle(&ty_code.code).expect("error: can't parse handle");
                                                visitor.visit_handle(handle_def);
                                            }
                                            _ => panic!("error: unhandled handle TypSpec node"),
                                        }
                                    }
                                    Some("funcpointer") => {
                                        match ty.spec {
                                            vk_parse::TypeSpec::Code(ref ty_code) => {
                                                let fptr_def = parse_fptr(&ty_code.code).expect("error: can't parse fptr");
                                                visitor.visit_fptr(fptr_def);
                                            }
                                            _ => panic!("error: unhandled handle TypSpec node"),
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
                            let def = match parse_command(&cmd_def.code) {
                                Ok(def) => def,
                                Err(_) => panic!("error: can't parse command"),
                            };
                            let def_wrapper = CommandDefWrapper {
                                def,
                                raw: cmd_def,
                            };
                            visitor.visit_command(def_wrapper);
                        }
                        _ => panic!("unexpected Command node"),
                    }
                }
            }
            Feature(feature) => {
                for feature_child in feature.children.iter() {
                    use vk_parse::ExtensionChild::*;
                    let feature_name = feature.name.as_str().into();
                    visitor.visit_feature_name(feature_name);
                    match feature_child {
                        Require {
                            api: _,
                            profile: _,
                            extension: _,
                            feature: _,
                            comment: _,
                            items,
                        } => {
                            for item in items.iter() {
                                use vk_parse::InterfaceItem::*;
                                match item {
                                    Comment(_) => {}
                                    Type { name: _, comment: _ } => {}
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
                                    Command { name: cmd_name, comment: _ } => {
                                        visitor.visit_require_command(CommandRef { name: &cmd_name, version: feature_name });
                                    }
                                    _ => panic!("unexpected InterfaceItem node"),
                                }
                            }
                        }
                        Remove {
                            api: _,
                            profile: _,
                            comment: _,
                            items,
                        } => {
                            for item in items.iter() {
                                use vk_parse::InterfaceItem::*;
                                match item {
                                    Comment(_) => {}
                                    Type { name: _, comment: _ } => {}
                                    Enum(_) => {}
                                    Command { name: cmd_name, comment: _ } => {
                                        visitor.visit_remove_command(CommandRef { name: &cmd_name, version: feature_name });
                                    }
                                    _ => panic!("unexpected InterfaceItem node"),
                                }
                            }
                        }
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
                                api: _,
                                profile: _,
                                extension: required_extension,
                                feature: requiered_feature,
                                comment: _,
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
                                visitor.visit_ex_require_node(ExtensionInfo {
                                    name_parts: parts,
                                    required: extension.requires.as_ref().map(|req|req.split(',')),
                                    kind: extension.ext_type.as_deref().expect("error: expected ex_type"),
                                });

                                for item in items.iter() {
                                    use vk_parse::InterfaceItem::*;
                                    match item {
                                        Comment(_) => {}
                                        Type { name: _, comment: _ } => {}
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
                                            else if enm.spec.is_some() {
                                                visitor.visit_constant(VkParseEnumConstant {
                                                    number: extension.number,
                                                    enm,
                                                    target: extends,
                                                    is_alias: enm.spec.is_alias(),
                                                })
                                            }
                                        }
                                        Command { name, comment: _ } => {
                                            visitor.visit_ex_cmd_ref(name, &parts);
                                        }
                                        _ => panic!("unexpected InterfaceChild node"),
                                    }
                                }
                            }
                            Remove {
                                api: _,
                                profile: _,
                                comment: _,
                                items: _,
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
    fn is_some(&self) -> bool;
    fn is_alias(&self) -> bool;
    fn extends(&self) -> Option<&str>;
}

impl EnumSpecEx for vk_parse::EnumSpec {
    fn is_some(&self) -> bool {
        ! matches!(self, Self::None)
    }
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

#[derive(Clone, Copy)]
pub struct VkParseExtensionParts<'a> {
    pub extension_name: &'a str,
    pub further_extended: Option<&'a str>,
}

pub struct StructDef<'a> {
    pub name: &'a str,
    pub members: Members<'a>,
}

pub struct UnionDef<'a> {
    pub name: &'a str,
    pub members: Members<'a>,
}

pub struct Members<'a> {
    members: std::slice::Iter<'a, vk_parse::TypeMember>,
}

impl<'a> Iterator for Members<'a> {
    type Item = MemberKind<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let member = self.members.next()?;

        use vk_parse::TypeMember;
        match member {
            TypeMember::Definition(ref def) => {
                let field = parse_field(def.code.as_str())
                    .expect("error: failed to parse struct member code");
                Some(MemberKind::Member(field))
            }
            TypeMember::Comment(ref cmnt) => {
                Some(MemberKind::Comment(cmnt))
            }
            _ => panic!("error: unexpected TypeMember node"),
        }
    }
}

pub enum MemberKind<'a> {
    Member(ctype::Cfield),
    Comment(&'a str)
}

pub struct ExtensionInfo<'a, I> {
    pub name_parts: VkParseExtensionParts<'a>,
    pub required: Option<I>,
    pub kind: &'a str,
}

pub struct VkBastetype<'a> {
    pub name: &'a str,
    pub ty: &'a str,
}

pub struct HandleDef<'a> {
    pub name: &'a str,
    pub kind: HandleKind,
}

pub enum HandleKind {
    Dispatchable,
    NonDispatchable,
}

pub struct FptrDef<'a> {
    pub name: &'a str,
    pub params: Parameters<'a>,
    pub return_type: ctype::Ctype,
}

pub struct CommandRef<'a> {
    pub name: &'a str,
    pub version: crate::utils::VkTyName,
}

pub struct CommandDef<'a> {
    pub name: &'a str,
    pub params: Parameters<'a>,
    pub return_type: ctype::Ctype,
}

pub struct CommandDefWrapper<'a> {
    pub def: CommandDef<'a>,
    pub raw: &'a vk_parse::CommandDefinition,
}

pub struct Parameters<'a> {
    members: crate::simple_parse::TokenIter<'a>,
}

impl<'a> Iterator for Parameters<'a> {
    type Item = ctype::Cfield;
    fn next(&mut self) -> Option<Self::Item> {
        use crate::simple_parse::*;

        let (_input, end1) = opt(followed(tag(")"), tag(";")))(self.members.clone())
            .expect("opt can't fail");

        let (input, end2) = opt(followed(tag("void"), tag(")")))(self.members.clone())
            .expect("opt can't fail");

        if end1.is_some() || end2.is_some() {
            self.members = input;
            return None;
        }
        else {
            let parse_cfield = || -> Result<(crate::simple_parse::TokenIter, ctype::Cfield), ()> {
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

                let (input, _) = repeat(
                    input,
                    delimited(tag("["), token(), tag("]")),
                    |(_, _size, _)| if c.is_some() {
                        ty.push_pointer(ctype::Pointer::Const);
                    }
                    else {
                        ty.push_pointer(ctype::Pointer::Mut);
                    }
                )?;

                let (input, _) = opt(tag(","))(input)?;

                Ok((input, ctype::Cfield::new(name, ty)))
            };

            let (input, param) = parse_cfield().expect(format!("error: can parse param from: {}", self.members.inner_str()).as_str());

            self.members = input;

            Some(param)
        }
    }
}

fn parse_basetype<'a>(code: &'a str) -> Result<VkBastetype, ()> {
    use crate::simple_parse::*;

    let input = TokenIter::new(code);
    let (input, _) = tag("typedef")(input)?;
    let (input, ty) = token()(input)?;
    let (input, name) = token()(input)?;
    let (_input, _) = tag(";")(input)?;
    Ok(VkBastetype {
        name,
        ty,
    })
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

fn parse_handle<'a>(code: &'a str) -> Result<HandleDef<'a>, ()> {
    use crate::simple_parse::*;

    let input = crate::simple_parse::TokenIter::new(code);

    let (_input, (kind, (_, name, _))) = followed(token(), delimited(tag("("), token(), tag(")")))(input)?;

    match kind {
        "VK_DEFINE_HANDLE" => {
            Ok(HandleDef {
                name,
                kind: HandleKind::Dispatchable,
            })
        }
        "VK_DEFINE_NON_DISPATCHABLE_HANDLE" => {
            Ok(HandleDef {
                name,
                kind: HandleKind::NonDispatchable,
            })
        }
        _ => panic!("error: unknown handle kind"),
    }
}

fn parse_fptr<'a>(code: &'a str) -> Result<FptrDef<'a>, ()> {
    use crate::simple_parse::*;

    let input = crate::simple_parse::TokenIter::new(code);

    let (input, _) = tag("typedef")(input)?;

    let (input, return_base_type) = token()(input)?;
    let (input, ptr) = opt(tag("*"))(input)?;

    let (input, _) = followed(tag("("), followed(tag("VKAPI_PTR"), tag("*")))(input)?;

    let (input, fptr_name) = token()(input)?;

    let (input, _) = followed(tag(")"), tag("("))(input)?;

    let mut return_type = if return_base_type == "void" && ptr.is_none() {
        ctype::Ctype::new("()")
    }
    else {
        ctype::Ctype::new(return_base_type)
    };

    if ptr.is_some() {
        return_type.push_pointer(ctype::Pointer::Mut);
    }

    Ok(FptrDef {
        name: fptr_name,
        params: Parameters { members: input },
        return_type,
    })
}

fn parse_command<'a>(code: &'a str) -> Result<CommandDef<'a>, ()> {
    use crate::simple_parse::*;

    let input = crate::simple_parse::TokenIter::new(code);

    let (input, c) = opt(tag("const"))(input)?;
    let (input, _) = opt(tag("struct"))(input)?;
    let (input, bt) = token()(input)?;
    let (input, p) = opt(tag("*"))(input)?;

    let mut return_type = if bt == "void" && p.is_none() {
        ctype::Ctype::new("()")
    }
    else {
        ctype::Ctype::new(bt)
    };

    if p.is_some() && c.is_some() {
        return_type.push_pointer(ctype::Pointer::Const);
    }
    else if p.is_some() {
        return_type.push_pointer(ctype::Pointer::Mut);
    }

    let (input, _) = repeat(
        input,
        followed(opt(tag("const")), tag("*")),
        |(c, _)| {
            if c.is_some() {
                return_type.push_pointer(ctype::Pointer::Const);
            }
            else {
                return_type.push_pointer(ctype::Pointer::Mut);
            }
        }
    )?;

    let (input, name) = token()(input)?;

    let (input, _) = tag("(")(input)?;

    Ok(CommandDef {
        name,
        params: Parameters { members: input },
        return_type,
    })
}
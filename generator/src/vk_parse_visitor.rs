use crate::{ctype, utils::VkTyName};

pub trait VisitVkParse<'a> {
    fn visit_alias(&mut self, name: &'a str, alias: &'a str);
    fn visit_enum(&mut self, enm: &'a vk_parse::Type);
    fn visit_command(&mut self, def_wrapper: CommandDefWrapper<'a>);
    fn visit_ex_enum(&mut self, spec: VkParseEnumConstant<'a>);
    fn visit_ex_require_node(&mut self, info: ExtensionInfo<'a, '_>);
    fn visit_ex_cmd_ref(&mut self, cmd_name: &'a str, parts: &VkParseExtensionParts<'a>);
    fn visit_struct_def(&mut self, def: StructDef<'a>);
    fn visit_constant(&mut self, spec: VkParseEnumConstant<'a>);
    fn visit_basetype(&mut self, basetype: VkBasetype<'a>);
    fn visit_bitmask(&mut self, basetype: VkBasetype<'a>);
    fn visit_union(&mut self, def: UnionDef<'a>);
    fn visit_handle(&mut self, def: HandleDef<'a>);
    fn visit_fptr(&mut self, def: FptrDef<'a>);
    fn visit_feature_name(&mut self, name: VkTyName);
    fn visit_require_command(&mut self, def: CommandRef<'a>);
    fn visit_remove_command(&mut self, def: CommandRef<'a>);
    fn visit_external_type(&mut self, name: VkTyName);
    fn visit_require_type(&mut self, name: &'a str, from: &'a str);
    // fn visit_api_version(&mut self, version: (u32, u32));
    // fn visit_header_version(&mut self, version: u32);
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
                            // skip type which are not for 'vulkan'
                            if !supported_api(ty.api.as_ref()) {
                                continue;
                            }
                            if ty.name.is_some() && ty.alias.is_some() {
                                visitor.visit_alias(
                                    ty.name.as_ref().unwrap(),
                                    ty.alias.as_ref().unwrap(),
                                );
                            } else {
                                match ty.category.as_deref() {
                                    Some("enum") => {
                                        if ty
                                            .name
                                            .as_ref()
                                            .expect("error: enum with no name")
                                            .contains("FlagBits")
                                        {
                                            continue;
                                        }
                                        visitor.visit_enum(ty);
                                    }
                                    Some("struct") => match ty.spec {
                                        vk_parse::TypeSpec::Code(ref _ty_code) => {}
                                        vk_parse::TypeSpec::Members(ref members) => {
                                            visitor.visit_struct_def(StructDef {
                                                name: ty
                                                    .name
                                                    .as_deref()
                                                    .expect("error: struct with no name"),
                                                members: Members {
                                                    members: members.iter(),
                                                },
                                                extends: Extends(ty.structextends.as_deref()),
                                            });
                                        }
                                        vk_parse::TypeSpec::None => {}
                                        _ => panic!("error: unhandled struct TypSpec node"),
                                    },
                                    Some("basetype") => {
                                        match ty.spec {
                                            vk_parse::TypeSpec::Code(ref code) => {
                                                if let Ok(basetype) = parse_basetype(&code.code) {
                                                    visitor.visit_basetype(basetype);
                                                } else if let Ok(extern_type) =
                                                    parse_external_opaque_type(&code.code)
                                                {
                                                    visitor.visit_external_type(extern_type);
                                                } else if code.code.contains("#ifdef __OBJC__")
                                                    || code.code.contains("typedef struct")
                                                {
                                                    if code.markup.len() != 1 {
                                                        panic!("error: can't parse __OBJC__ extern type?");
                                                    } else {
                                                        match &code.markup[0] {
                                                            vk_parse::TypeCodeMarkup::Name(name) => visitor.visit_external_type(name.into()),
                                                            _ => panic!("error: unexpected definition in #ifdef __OBJC__"),
                                                        };
                                                    }
                                                } else {
                                                    panic!(
                                                        "error: can't parse basetype code: {}",
                                                        code.code
                                                    );
                                                }
                                            }
                                            _ => panic!("unexpected basetype spec"),
                                        }
                                    }
                                    Some("bitmask") => match ty.spec {
                                        vk_parse::TypeSpec::Code(ref code) => {
                                            let basetype = parse_basetype(&code.code)
                                                .expect("error: can't parse bitmask in vk_parse");
                                            visitor.visit_bitmask(basetype);
                                        }
                                        _ => panic!("unexpected bitmask spec"),
                                    },
                                    Some("union") => match ty.spec {
                                        vk_parse::TypeSpec::Code(ref _ty_code) => {}
                                        vk_parse::TypeSpec::Members(ref members) => {
                                            visitor.visit_union(UnionDef {
                                                name: ty
                                                    .name
                                                    .as_deref()
                                                    .expect("error: union with no name"),
                                                members: Members {
                                                    members: members.iter(),
                                                },
                                            });
                                        }
                                        vk_parse::TypeSpec::None => {}
                                        _ => panic!("error: unhandled union TypSpec node"),
                                    },
                                    Some("handle") => match ty.spec {
                                        vk_parse::TypeSpec::Code(ref ty_code) => {
                                            let handle_def = parse_handle(&ty_code.code)
                                                .expect("error: can't parse handle");
                                            visitor.visit_handle(handle_def);
                                        }
                                        _ => panic!("error: unhandled handle TypSpec node"),
                                    },
                                    Some("funcpointer") => match ty.spec {
                                        vk_parse::TypeSpec::Code(ref ty_code) => {
                                            let fptr_def = parse_fptr(&ty_code.code)
                                                .expect("error: can't parse fptr");
                                            visitor.visit_fptr(fptr_def);
                                        }
                                        _ => panic!("error: unhandled handle TypSpec node"),
                                    },
                                    Some("define") => {
                                        match ty.spec {
                                            vk_parse::TypeSpec::Code(ref code) => {
                                                let code = code.code.as_str();
                                                if let Ok(extern_ty) =
                                                    parse_external_opaque_type(code)
                                                {
                                                    visitor.visit_external_type(extern_ty);
                                                }
                                                // else if let Ok(api_version) = parse_api_version(code) {
                                                //     visitor.visit_api_version(api_version);
                                                // }
                                                // else if let Ok(header_version) = parse_header_version(code) {
                                                //     visitor.visit_header_version(header_version);
                                                // }
                                            }
                                            vk_parse::TypeSpec::Members(_) => {}
                                            vk_parse::TypeSpec::None => {}
                                            _ => panic!("unhandled type spec kind"),
                                        }
                                    }
                                    None => {
                                        match ty.requires.as_deref() {
                                            Some("vk_platform") => {} // this defines normal types like uint32_t which we already know
                                            Some(_) => {
                                                // this should be types that are defined in an external library
                                                visitor.visit_external_type(
                                                    ty.name.as_ref().unwrap().into(),
                                                )
                                            }
                                            None => {}
                                        }
                                    }
                                    Some(_) => {}
                                }
                            }
                        }
                        _ => panic!("unexpected TypeChild"),
                    }
                }
            }
            Enums(enms) => {
                match enms.kind.as_deref() {
                    None => {
                        // API Constant or the like
                        for enum_child in enms.children.iter() {
                            use vk_parse::EnumsChild;
                            match enum_child {
                                EnumsChild::Enum(ref enm) => {
                                    visitor.visit_constant(VkParseEnumConstant {
                                        number: None,
                                        enm,
                                        target: None,
                                        _is_alias: false,
                                    });
                                }
                                EnumsChild::Comment(_) => {}
                                EnumsChild::Unused(_) => {}
                                _ => panic!("error: unexpected EnumsChild"),
                            }
                        }
                    }
                    Some("enum" | "bitmask") => {
                        // enum variants
                        for enum_child in enms.children.iter() {
                            use vk_parse::EnumsChild;
                            match enum_child {
                                EnumsChild::Enum(ref enm) => {
                                    visitor.visit_ex_enum(VkParseEnumConstant {
                                        number: None,
                                        enm,
                                        target: enms.name.as_deref(),
                                        _is_alias: enm.spec.is_alias(),
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
                            if supported_api(cmd_def.api.as_ref()) {
                                let def = match parse_command(&cmd_def.code) {
                                    Ok(def) => def,
                                    Err(_) => panic!("error: can't parse command"),
                                };
                                let def_wrapper = CommandDefWrapper { def, raw: cmd_def };
                                visitor.visit_command(def_wrapper);
                            }
                        }
                        _ => panic!("unexpected Command node"),
                    }
                }
            }
            Feature(feature) => {
                if !supported_api(Some(&feature.api)) {
                    continue;
                }
                for feature_child in feature.children.iter() {
                    use vk_parse::ExtensionChild::*;
                    let feature_name = feature.name.as_str().into();
                    visitor.visit_feature_name(feature_name);
                    match feature_child {
                        Require {
                            api,
                            profile: _,
                            extension: _,
                            feature: _,
                            comment: _,
                            depends: _,
                            items,
                        } => {
                            if !supported_api(api.as_ref()) {
                                continue;
                            }
                            for item in items.iter() {
                                use vk_parse::InterfaceItem::*;
                                match item {
                                    Comment(_) => {}
                                    Type {
                                        name: type_name,
                                        comment: _,
                                    } => {
                                        visitor.visit_require_type(&type_name, &feature.name);
                                    }
                                    Enum(enm) => {
                                        let extends = enm.spec.extends();
                                        if extends.is_some() {
                                            visitor.visit_ex_enum(VkParseEnumConstant {
                                                number: None,
                                                enm,
                                                target: extends,
                                                _is_alias: enm.spec.is_alias(),
                                            });
                                        }
                                    }
                                    Command {
                                        name: cmd_name,
                                        comment: _,
                                    } => {
                                        visitor.visit_require_command(CommandRef {
                                            name: &cmd_name,
                                            version: feature_name,
                                        });
                                    }
                                    _ => panic!("unexpected InterfaceItem node"),
                                }
                            }
                        }
                        Remove {
                            api,
                            profile: _,
                            comment: _,
                            items,
                        } => {
                            if !supported_api(api.as_ref()) {
                                continue;
                            }
                            for item in items.iter() {
                                use vk_parse::InterfaceItem::*;
                                match item {
                                    Comment(_) => {}
                                    Type {
                                        name: _,
                                        comment: _,
                                    } => {}
                                    Enum(_) => {}
                                    Command {
                                        name: cmd_name,
                                        comment: _,
                                    } => {
                                        visitor.visit_remove_command(CommandRef {
                                            name: &cmd_name,
                                            version: feature_name,
                                        });
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
                    if !supported_api(extension.supported.as_ref()) {
                        continue;
                    }
                    for ex_child in extension.children.iter() {
                        use vk_parse::ExtensionChild::*;
                        match ex_child {
                            Require {
                                api,
                                profile: _,
                                extension: required_extension,
                                feature: required_feature,
                                comment: _,
                                depends,
                                items,
                            } => {
                                if !supported_api(api.as_ref()) {
                                    continue;
                                }
                                // assuming for now that feature and extension additions are exclusive
                                let extended: Option<Term> = match (required_feature, required_extension, depends) {
                                    (Some(_feature), None, None) => panic!("not implemented term parsing for old vk xml"),
                                    (None, Some(_extension), None) => panic!("not implemented term parsing for old vk xml"),
                                    (None, None, Some(depends)) => Some(depends.as_str().into()),
                                    (None, None, None) => None,
                                    _ => panic!("error: not expecting feature, extension, and depends additions at the same time"),
                                };
                                let parts = match extended {
                                    Some(term) => VkParseExtensionParts::Extended(
                                        term.prepend(&extension.name),
                                    ),
                                    None => VkParseExtensionParts::Base(&extension.name),
                                };

                                let dependent_extensions =
                                    match (&extension.requires, &extension.depends) {
                                        (Some(_), None) => {
                                            panic!("not implemented term parsing for old vk xml")
                                        }
                                        (None, Some(d)) => Some(Term::from(d.as_str())),
                                        (None, None) => None,
                                        _ => panic!("unexpected extension dependency description"),
                                    };

                                visitor.visit_ex_require_node(ExtensionInfo {
                                    name_parts: &parts,
                                    dependencies: dependent_extensions,
                                    kind: extension
                                        .ext_type
                                        .as_deref()
                                        .expect("error: expected ex_type"),
                                    promoted_to: extension.promotedto.as_deref(),
                                });

                                for item in items.iter() {
                                    use vk_parse::InterfaceItem::*;
                                    match item {
                                        Comment(_) => {}
                                        Type {
                                            name: type_name,
                                            comment: _,
                                        } => {
                                            visitor.visit_require_type(&type_name, &extension.name);
                                        }
                                        Enum(enm) => {
                                            if !supported_api(enm.api.as_ref()) {
                                                continue;
                                            }
                                            let extends = enm.spec.extends();
                                            if extends.is_some() {
                                                visitor.visit_ex_enum(VkParseEnumConstant {
                                                    number: extension.number,
                                                    enm,
                                                    target: extends,
                                                    _is_alias: enm.spec.is_alias(),
                                                });
                                            } else if enm.spec.is_some() {
                                                visitor.visit_constant(VkParseEnumConstant {
                                                    number: extension.number,
                                                    enm,
                                                    target: extends,
                                                    _is_alias: enm.spec.is_alias(),
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
            Formats(_) => {}
            SpirvExtensions(_) => {}
            SpirvCapabilities(_) => {}
            Sync(_) => {}
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
        !matches!(self, Self::None)
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
    pub _is_alias: bool,
}

#[derive(Clone)]
pub enum VkParseExtensionParts<'a> {
    Base(&'a str),
    Extended(Term<'a>),
}

pub struct StructDef<'a> {
    pub name: &'a str,
    pub members: Members<'a>,
    pub extends: Extends<'a>,
}

#[derive(Clone, Copy)]
pub struct Extends<'a>(Option<&'a str>);

pub struct ExtendsIter<'a>(Option<std::str::Split<'a, char>>);

impl<'a> Iterator for ExtendsIter<'a> {
    type Item = VkTyName;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|i| i.next().map(Into::into))
    }
}

impl<'a> IntoIterator for Extends<'a> {
    type Item = VkTyName;

    type IntoIter = ExtendsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ExtendsIter(self.0.map(|e| e.split(',')).into())
    }
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
                if supported_api(def.api.as_ref()) {
                    let field = match parse_field(def.code.as_str()) {
                        Ok(field) => field,
                        // fallback with newer vk_parse on older vk.xml
                        // (there is code where name and type get mushed together, which appears like a formatting error in older vk.xml, but worked anyway with older vk_parse)
                        Err(ParseFieldError::NoName) => {
                            let mut name = None;
                            let mut ty = None;

                            for markup in def.markup.iter() {
                                match markup {
                                    vk_parse::TypeMemberMarkup::Name(def_name) => {
                                        assert!(
                                            name.is_none(),
                                            "ERROR: too many names in member markup"
                                        );
                                        name = Some(def_name);
                                    }
                                    vk_parse::TypeMemberMarkup::Type(def_type) => {
                                        assert!(
                                            ty.is_none(),
                                            "ERROR: too many types in member markup"
                                        );
                                        ty = Some(def_type);
                                    }
                                    vk_parse::TypeMemberMarkup::Enum(_) => {
                                        panic!("ERROR: not expecting Enum markup in member")
                                    }
                                    vk_parse::TypeMemberMarkup::Comment(_) => {}
                                    _ => panic!("ERROR: unhandled markup. Check to ensure nothing important is omitted"),
                                }
                            }

                            match (name, ty) {
                                (Some(name), Some(ty)) => {
                                    ctype::Cfield::new(name, ctype::Ctype::new(ty))
                                }
                                _ => panic!("ERROR: could not make Ctype from member markup"),
                            }
                        }
                        Err(ParseFieldError::Default) => {
                            panic!("ERROR: cannot parse struct/union member")
                        }
                    };
                    Some(MemberKind::Member(field))
                } else {
                    Some(MemberKind::UnsupportedApi)
                }
            }
            TypeMember::Comment(ref comment) => Some(MemberKind::Comment(comment)),
            _ => panic!("error: unexpected TypeMember node"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Term<'a> {
    Single(&'a str),
    And(Vec<Term<'a>>),
    Or(Vec<Term<'a>>),
}

impl<'a> Term<'a> {
    fn prepend(self, name: &'a str) -> Self {
        match self {
            Term::Single(other) => Term::And(vec![Term::Single(name), Term::Single(other)]),
            Term::And(mut terms) => {
                terms.insert(0, Term::Single(name));
                Term::And(terms)
            }
            Term::Or(_) => Term::And(vec![Term::Single(name), self]),
        }
    }

    pub fn name(&self) -> String {
        fn recurse(this: &Term, accumulate: &mut String) {
            match this {
                Term::Single(name) => accumulate.push_str(name),
                Term::And(terms) => {
                    let mut iter = terms.iter().peekable();
                    while let Some(term) = iter.next() {
                        recurse(term, accumulate);
                        if iter.peek().is_some() {
                            accumulate.push_str("__AND__");
                        }
                    }
                }
                Term::Or(terms) => {
                    let mut iter = terms.iter().peekable();
                    while let Some(term) = iter.next() {
                        recurse(term, accumulate);
                        if iter.peek().is_some() {
                            accumulate.push_str("__OR__");
                        }
                    }
                }
            }
        }

        let mut name = String::new();
        recurse(self, &mut name);
        name
    }
}

impl<'a> From<&'a str> for Term<'a> {
    fn from(value: &'a str) -> Self {
        let mut tokens = crate::simple_parse::TokenIter::new(value);
        get_term(&mut tokens)
    }
}

fn get_term<'a>(tokens: &mut crate::simple_parse::TokenIter<'a>) -> Term<'a> {
    let mut names = vec![];
    let mut and = true; // assume the terms are ANDed, and switch to OR (false) if a ',' is detected
    let mut found_and = false; // confirmed 'and' should be true, and if OR is found after this, we panic
    while let Some(token) = tokens.next() {
        match token {
            "(" => names.push(get_term(tokens)),
            ")" => break,
            "+" => found_and = true,
            "," => {
                assert!(
                    !found_and,
                    "Extension dependencies unclear mix of AND and OR conditions"
                );
                and = false
            }
            name => names.push(Term::Single(name)),
        }
    }

    if names.len() == 1 {
        names.pop().unwrap()
    } else if and {
        Term::And(names)
    } else {
        Term::Or(names)
    }
}

pub enum MemberKind<'a> {
    Member(ctype::Cfield),
    Comment(&'a str),
    UnsupportedApi,
}

pub struct ExtensionInfo<'a, 'p> {
    pub name_parts: &'p VkParseExtensionParts<'a>,
    pub dependencies: Option<Term<'a>>,
    pub kind: &'a str,
    pub promoted_to: Option<&'a str>,
}

pub struct VkBasetype<'a> {
    pub name: &'a str,
    pub ty: &'a str,
    pub ptr: bool,
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
    pub version: VkTyName,
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

        let (_input, end1) =
            opt(followed(tag(")"), tag(";")))(self.members.clone()).expect("opt can't fail");

        let (input, end2) =
            opt(followed(tag("void"), tag(")")))(self.members.clone()).expect("opt can't fail");

        if end1.is_some() || end2.is_some() {
            self.members = input;
            return None;
        } else {
            let parse_cfield = || -> Result<(crate::simple_parse::TokenIter, ctype::Cfield), ()> {
                let (input, c) = opt(tag("const"))(input)?;
                let (input, _) = opt(tag("struct"))(input)?;
                let (input, bt) = token()(input)?;
                let (input, p) = opt(tag("*"))(input)?;

                let mut ty = ctype::Ctype::new(bt);

                if p.is_some() && c.is_some() {
                    ty.push_pointer(ctype::Pointer::Const);
                } else if p.is_some() {
                    ty.push_pointer(ctype::Pointer::Mut);
                }

                let (input, _) = repeat(input, followed(opt(tag("const")), tag("*")), |(c, _)| {
                    if c.is_some() {
                        ty.push_pointer(ctype::Pointer::Const);
                    } else {
                        ty.push_pointer(ctype::Pointer::Mut);
                    }
                })?;

                let (input, name) = token()(input)?;

                let (input, _) = repeat(
                    input,
                    delimited(tag("["), token(), tag("]")),
                    |(_, _size, _)| {
                        if c.is_some() {
                            ty.push_pointer(ctype::Pointer::Const);
                        } else {
                            ty.push_pointer(ctype::Pointer::Mut);
                        }
                    },
                )?;

                let (input, _) = opt(tag(","))(input)?;

                Ok((input, ctype::Cfield::new(name, ty)))
            };

            let (input, param) = parse_cfield().expect(
                format!("error: can parse param from: {}", self.members.inner_str()).as_str(),
            );

            self.members = input;

            Some(param)
        }
    }
}

fn parse_basetype<'a>(code: &'a str) -> Result<VkBasetype<'a>, ()> {
    use crate::simple_parse::*;

    let input = TokenIter::new(code);
    let (input, _) = tag("typedef")(input)?;
    let (input, ty) = token()(input)?;
    let (input, ptr) = opt(tag("*"))(input)?;
    let (input, name) = token()(input)?;
    let (_input, _) = tag(";")(input)?;
    Ok(VkBasetype {
        name,
        ty,
        ptr: ptr.is_some(),
    })
}

enum ParseFieldError {
    NoName,
    Default,
}

impl From<()> for ParseFieldError {
    fn from(_value: ()) -> Self {
        ParseFieldError::Default
    }
}

fn parse_field(code: &str) -> Result<ctype::Cfield, ParseFieldError> {
    use crate::simple_parse::*;

    let input = crate::simple_parse::TokenIter::new(code);

    let (input, c) = opt(tag("const"))(input)?;
    let (input, _) = opt(tag("struct"))(input)?;
    let (input, bt) = token()(input)?;
    let (input, p) = opt(tag("*"))(input)?;

    let mut ty = ctype::Ctype::new(bt);

    if p.is_some() && c.is_some() {
        ty.push_pointer(ctype::Pointer::Const);
    } else if p.is_some() {
        ty.push_pointer(ctype::Pointer::Mut);
    }

    let (input, _) = repeat(input, followed(opt(tag("const")), tag("*")), |(c, _)| {
        if c.is_some() {
            ty.push_pointer(ctype::Pointer::Const);
        } else {
            ty.push_pointer(ctype::Pointer::Mut);
        }
    })?;

    let (input, name) = token()(input).map_err(|_| ParseFieldError::NoName)?;

    let (input, bit_width) = opt(followed(tag(":"), token()))(input)?;

    if let Some((_colon, bit_width)) = bit_width {
        let bit_width: u8 = str::parse(bit_width).expect("error: can't parse bit_width");
        ty.set_bit_width(bit_width);
    }

    let (mut input, _) = repeat(
        input,
        delimited(tag("["), token(), tag("]")),
        |(_, size, _)| ty.push_array(size),
    )?;

    // this is expected to consume all tokens
    if input.next().is_some() {
        Err(ParseFieldError::Default)
    } else {
        Ok(ctype::Cfield::new(name, ty))
    }
}

fn parse_handle<'a>(code: &'a str) -> Result<HandleDef<'a>, ()> {
    use crate::simple_parse::*;

    let input = crate::simple_parse::TokenIter::new(code);

    let (_input, (kind, (_, name, _))) =
        followed(token(), delimited(tag("("), token(), tag(")")))(input)?;

    match kind {
        "VK_DEFINE_HANDLE" => Ok(HandleDef {
            name,
            kind: HandleKind::Dispatchable,
        }),
        "VK_DEFINE_NON_DISPATCHABLE_HANDLE" => Ok(HandleDef {
            name,
            kind: HandleKind::NonDispatchable,
        }),
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
    } else {
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
    } else {
        ctype::Ctype::new(bt)
    };

    if p.is_some() && c.is_some() {
        return_type.push_pointer(ctype::Pointer::Const);
    } else if p.is_some() {
        return_type.push_pointer(ctype::Pointer::Mut);
    }

    let (input, _) = repeat(input, followed(opt(tag("const")), tag("*")), |(c, _)| {
        if c.is_some() {
            return_type.push_pointer(ctype::Pointer::Const);
        } else {
            return_type.push_pointer(ctype::Pointer::Mut);
        }
    })?;

    let (input, name) = token()(input)?;

    let (input, _) = tag("(")(input)?;

    Ok(CommandDef {
        name,
        params: Parameters { members: input },
        return_type,
    })
}

fn parse_external_opaque_type(code: &str) -> Result<VkTyName, ()> {
    use crate::simple_parse::*;

    let input = crate::simple_parse::TokenIter::new(code);

    let (input, _) = tag("struct")(input)?;
    let (input, name) = token()(input)?;
    let (mut input, _) = tag(";")(input)?;

    if input.next().is_some() {
        Err(())
    } else {
        Ok(name.into())
    }
}

fn supported_api<S: AsRef<str>>(api: Option<&S>) -> bool {
    api.map_or(true, |s| crate::vulkansc::api_for_vulkan(s.as_ref()))
}

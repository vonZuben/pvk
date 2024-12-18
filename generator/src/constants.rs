use krs_quote::krs_quote_with;

use crate::utils::*;

use crate::ctype::{self, Visibility};
use crate::vk_parse_visitor;

#[derive(PartialEq, Eq, Debug)]
pub struct Constant3 {
    name: VkTyName,
    ty: ctype::Ctype,
    val: ConstValue2,
    visibility: Visibility,
    target: Option<VkTyName>,
}

impl Constant3 {
    pub fn new(
        name: impl Into<VkTyName>,
        ty: ctype::Ctype,
        val: ConstValue2,
        target: Option<VkTyName>,
    ) -> Self {
        let name = name.into();
        let name = match target {
            Some(target) => crate::enumerations::make_variant_name(&target, name.as_str()).into(),
            None => name,
        };
        // at first, visibility was assumed to always be public, but there are some times we want private now
        // keep public as default so everything still works and set private when needed
        let visibility = Visibility::Public;
        Self {
            name,
            ty,
            val,
            target,
            visibility,
        }
    }

    pub fn name(&self) -> &VkTyName {
        &self.name
    }

    pub fn is_alias(&self) -> bool {
        matches!(self.val.value, ValueKind::Enumref(..))
    }

    pub fn value(&self) -> TokenWrapper {
        self.val.value()
    }
}

impl krs_quote::ToTokens for Constant3 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let ty = &self.ty;
        let val = &self.val;
        let visibility = self.visibility;

        krs_quote_with!( tokens <-
            #[allow(unused_parens)]
            #[allow(non_upper_case_globals)]
            {@visibility} const {@name}: {@ty} = {@val};
        );
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Negate2 {
    True,
    False,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ConstantContext {
    GlobalConstant,
    Enum,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConstValue2 {
    value: ValueKind,
    context: ConstantContext,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ValueKind {
    Offset(i64, Negate2),
    Text(String),
    Enumref(VkTyName, Option<VkTyName>),
    Number(i32),
    Hex(String),
    Bitpos(u32),
    Cexpr(String),
}

impl ConstValue2 {
    pub fn type_of(&self, constant_ref_map: &VecMap<VkTyName, Constant3>) -> ctype::Ctype {
        use ctype::Ctype;
        use ValueKind::*;
        match self.value {
            Offset(_, _) => panic!("I think this never happens"), // Ctype::new("usize"),
            Text(_) => Ctype::new("&'static str"),
            Enumref(enumref, _) => {
                let const_ref = constant_ref_map
                    .get(enumref)
                    .expect("error: enumref to constant that does not exist (yet?)");
                const_ref.ty.clone()
            }
            Number(_) => Ctype::new("usize"),
            Hex(_) => Ctype::new("usize"),
            Bitpos(_) => Ctype::new("Flags"),
            Cexpr(ref cexpr) => match cexpr {
                e if e.contains("ULL") => Ctype::new("u64"),
                e if e.contains("U") => Ctype::new("u32"),
                e if e.contains("f") || e.contains("F") => Ctype::new("f32"),
                _ => Ctype::new("u32"),
            },
        }
    }

    pub fn from_vk_parse(
        ex: vk_parse_visitor::VkParseEnumConstant,
        context: ConstantContext,
        target: Option<VkTyName>,
    ) -> Self {
        let enm = ex.enm;
        use vk_parse::EnumSpec::*;
        match enm.spec {
            Alias { ref alias, .. } => ConstValue2 {
                value: ValueKind::Enumref(alias.into(), target),
                context,
            },
            Offset {
                offset,
                extnumber,
                dir,
                ..
            } => {
                let number = extnumber.unwrap_or_else(|| {
                    ex.number.expect("error: enum extension must have a number")
                });
                let val = 1000000000 + (number - 1) * 1000 + offset;
                let negate = match dir {
                    true => Negate2::False,
                    false => Negate2::True,
                };
                ConstValue2 {
                    value: ValueKind::Offset(val, negate),
                    context,
                }
            }
            Bitpos { bitpos, .. } => ConstValue2 {
                value: ValueKind::Bitpos(bitpos.try_into().expect(
                    "error: expecting 32 bit number for Flags (is this not a Flags type?)",
                )),
                context,
            },
            Value { ref value, .. } => {
                if let Ok(val) = i32::from_str_radix(value, 10) {
                    ConstValue2 {
                        value: ValueKind::Number(val),
                        context,
                    }
                } else if value.starts_with('"') && value.ends_with('"') {
                    // TODO: in future, if I remove vkxml entirely, then this can just keep the quotes as part of the value rather than removing them
                    // TODO: vkxml is now removed. Is there some optimization possible now?
                    ConstValue2 {
                        value: ValueKind::Text(value[1..value.len() - 1].to_string()),
                        context,
                    }
                } else if value.starts_with("0x") {
                    // probably a hex value
                    ConstValue2 {
                        value: ValueKind::Hex(value[2..].to_string()),
                        context,
                    }
                } else {
                    // assume Cexpr
                    ConstValue2 {
                        value: ValueKind::Cexpr(value.to_string()),
                        context,
                    }
                }
            }
            None => panic!("error: enum has no value, is this somehow an enumref?"),
            _ => panic!("unexpected unknown value"),
        }
    }

    fn value(&self) -> TokenWrapper {
        use ValueKind::*;
        match self.value {
            Offset(calculated, negate) => match negate {
                Negate2::False => calculated.to_string().as_code(),
                Negate2::True => format!("-{}", calculated).as_code(),
            },
            Text(ref text) => krs_quote::Token::str_as_token(text).into(),
            Enumref(enumref, target) => match target {
                Some(target) => crate::enumerations::make_variant_name(&target, &enumref).as_code(),
                None => enumref.as_code(),
            },
            Number(num) => num.to_string().as_code(),
            Hex(ref hex) => format!("0x{:0>8}", hex).as_code(),
            Bitpos(bitpos) => format!("0x{:0>8X}", (1u64 << bitpos)).as_code(),
            Cexpr(ref cexpr) => cexpr
                .replace("ULL", "")
                .replace("U", "")
                .replace("~", "!")
                .replace("f", "")
                .replace("F", "")
                .as_code(),
        }
    }
}

impl krs_quote::ToTokens for ConstValue2 {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let value = self.value();

        match (self.context, &self.value) {
            (ConstantContext::Enum, ValueKind::Enumref(..)) => {
                krs_quote_with!(tokens <- Self::{@value} )
            }
            (ConstantContext::Enum, _) => krs_quote_with!(tokens <- Self({@value}) ),
            (ConstantContext::GlobalConstant, _) => {
                <TokenWrapper as krs_quote::ToTokens>::to_tokens(&value, tokens)
            }
        }
    }
}

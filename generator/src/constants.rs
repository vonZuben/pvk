use quote::{quote, ToTokens};

use vkxml::*;

use proc_macro2::TokenStream;

use crate::utils::*;

use crate::ctype;
use crate::vkxml_visitor;
use crate::vk_parse_visitor;

use std::fmt;

pub enum OptionMofifierFn<'a> {
    Some(Box<dyn for<'s> Fn(&'s str) -> String + 'a>),
    None,
}

impl<'a> OptionMofifierFn<'a> {
    fn new<M: for<'s> Fn(&'s str) -> String + 'a>(modifier: M) -> Self {
        Self::Some(Box::new(modifier))
    }
}

impl PartialEq for OptionMofifierFn<'_> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            OptionMofifierFn::Some(_) => false,
            OptionMofifierFn::None => true,
        }
    }
}

impl Eq for OptionMofifierFn<'_> {}

impl Clone for OptionMofifierFn<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Some(ref f) => panic!("shouldn't clone OptionMofifierFn with modifer"),
            Self::None => Self::None,
        }
    }
}

impl fmt::Debug for OptionMofifierFn<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionMofifierFn::Some(_) => write!(f, "with modifier"),
            OptionMofifierFn::None => write!(f, "no modifier"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Constant2<'a> {
    pub name: &'a str,
    val: TypeValueExpresion<'a>,
    name_modifier: OptionMofifierFn<'a>,
}

impl<'a> Constant2<'a> {
    pub fn new(name: &'a str, val: TypeValueExpresion<'a>) -> Self {
        Self { name, val, name_modifier: OptionMofifierFn::None }
    }
    pub fn with_name_modifier<M: for<'s> Fn(&'s str) -> String + Copy + 'a>(&self, modifier: M) -> Self {
        Self {
            name: self.name,
            val: self.val.with_name_modifier(modifier),
            name_modifier: OptionMofifierFn::new(modifier),
        }
    }
}

impl ToTokens for Constant2<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;
        let name = match self.name_modifier {
            OptionMofifierFn::Some(ref modifier) => modifier(self.name).as_code(),
            OptionMofifierFn::None => self.name.as_code(),
        };
        let ty = self.val.value_ctype();
        let val = &self.val;
        quote!(
            pub const #name: #ty = #val;
        )
        .to_tokens(tokens);
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Negate {
    True,
    False,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ConstValue<'a> {
    Offset(i64, Negate),
    Text(&'a str),
    Enumref(&'a str, OptionMofifierFn<'a>),
    Number(i32),
    Hex(&'a str),
    Bitpos(u32),
    Cexpr(&'a str),
}

impl<'a> ConstValue<'a> {
    fn value_ctype(&self) -> ctype::Ctype<'static> {
        use ctype::Ctype;
        use ConstValue::*;
        match self {
            Offset(_, _) => Ctype::new("usize"),
            Text(_) => Ctype::new("&'static str"),
            Enumref(enumref, _) => Ctype::new("usize"),
            Number(_) => Ctype::new("usize"),
            Hex(_) => Ctype::new("usize"),
            Bitpos(bitpos) => Ctype::new("Flags"),
            Cexpr(cexpr) => match cexpr {
                e if e.contains("ULL") => Ctype::new("u64"),
                e if e.contains("U") => Ctype::new("u32"),
                e if e.contains("f") || e.contains("F") => Ctype::new("f32"),
                _ => Ctype::new("u32"),
            },
        }
    }

    fn add_modified<M: for<'s> Fn(&'s str) -> String + 'a>(&mut self, modifier: M) {
        match self {
            Self::Enumref(_, ref mut opt_modifier) => *opt_modifier = OptionMofifierFn::new(modifier),
            _ => {}
        }
    }
}

impl ToTokens for ConstValue<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use ConstValue::*;
        match self {
            Offset(calcualted, negate) => match negate {
                Negate::False => calcualted.to_string().as_code().to_tokens(tokens),
                Negate::True => format!("-{}", calcualted).as_code().to_tokens(tokens),
            },
            Text(text) => quote!(#text).to_tokens(tokens),
            Enumref(enumref, modifier) => {
                match modifier {
                    OptionMofifierFn::Some(modifier) => modifier(enumref).as_code().to_tokens(tokens),
                    OptionMofifierFn::None => enumref.as_code().to_tokens(tokens),
                }
            }
            Number(num) => num.to_string().as_code().to_tokens(tokens),
            Hex(hex) => format!("0x{:0>8}", hex).as_code().to_tokens(tokens),
            Bitpos(bitpos) => format!("0x{:0>8X}", (1u64 << bitpos)).as_code().to_tokens(tokens),
            Cexpr(cexpr) => cexpr
                .replace("ULL", "")
                .replace("U", "")
                .replace("~", "!")
                .replace("f", "")
                .replace("F", "")
                .as_code().to_tokens(tokens),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum TypeValueExpresionKind {
    Literal,
    SimpleSelf, // simple Self(val) expression for use in associated const (vulkan enum emulation)
    SelfRef, // Self::...
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TypeValueExpresion<'a> {
    val: ConstValue<'a>,
    kind: TypeValueExpresionKind,
}

impl<'a> TypeValueExpresion<'a> {
    pub fn literal(val: impl Into<ConstValue<'a>>) -> Self {
        Self {
            val: val.into(),
            kind: TypeValueExpresionKind::Literal,
        }
    }
    pub fn simple_self(val: impl Into<ConstValue<'a>>) -> Self {
        Self {
            val: val.into(),
            kind: TypeValueExpresionKind::SimpleSelf,
        }
    }
    pub fn self_ref(val: impl Into<ConstValue<'a>>) -> Self {
        Self {
            val: val.into(),
            kind: TypeValueExpresionKind::SelfRef,
        }
    }

    fn value_ctype(&self) -> ctype::Ctype {
        use TypeValueExpresionKind::*;
        match self.kind {
            Literal => self.val.value_ctype(),
            SimpleSelf | SelfRef => ctype::Ctype::new("Self"),
        }
    }

    fn with_name_modifier<M: for<'s> Fn(&'s str) -> String + 'a>(&self, modifier: M) -> Self {
        let mut new = self.clone();
        new.val.add_modified(modifier);
        new
    }
}

impl ToTokens for TypeValueExpresion<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.kind {
            TypeValueExpresionKind::Literal => self.val.to_tokens(tokens),
            TypeValueExpresionKind::SimpleSelf => {
                let val = &self.val;
                quote!(Self(#val)).to_tokens(tokens);
            }
            TypeValueExpresionKind::SelfRef => {
                let val = &self.val;
                quote!(Self::#val).to_tokens(tokens);
            }
        }
    }
}

impl<'a> From<&'a vkxml::Constant> for ConstValue<'a> {
    fn from(vkxml_constant: &'a vkxml::Constant) -> Self {
        if let Some(number) = vkxml_constant.number {
            return ConstValue::Number(number);
        }

        if let Some(ref hex) = vkxml_constant.hex {
            return ConstValue::Hex(hex);
        }

        if let Some(bitpos) = vkxml_constant.bitpos {
            return ConstValue::Bitpos(bitpos);
        }

        if let Some(ref expr) = vkxml_constant.c_expression {
            return ConstValue::Cexpr(expr);
        }

        panic!("improper vkxml_constant does not have a value");
    }
}

impl<'a> From<&'a vkxml::ExtensionConstant> for ConstValue<'a> {
    fn from(vkxml_ex_constant: &'a vkxml::ExtensionConstant) -> Self {
        if let Some(ref text) = vkxml_ex_constant.text {
            return ConstValue::Text(text);
        }

        if let Some(ref enumref) = vkxml_ex_constant.enumref {
            return ConstValue::Enumref(enumref, OptionMofifierFn::None);
        }

        if let Some(num) = vkxml_ex_constant.number {
            return ConstValue::Number(num);
        }

        if let Some(ref hex_str) = vkxml_ex_constant.hex {
            return ConstValue::Hex(hex_str);
        }

        if let Some(bitpos) = vkxml_ex_constant.bitpos {
            return ConstValue::Bitpos(bitpos);
        }

        if let Some(ref expr) = vkxml_ex_constant.c_expression {
            return ConstValue::Cexpr(expr);
        }

        panic!("improper vkxml_ex_constant does not have a value");
    }
}

impl<'a> From<vk_parse_visitor::VkParseEnumConstant<'a>> for ConstValue<'a> {
    fn from(ex: vk_parse_visitor::VkParseEnumConstant<'a>) -> Self {
        let enm = ex.enm;
        use vk_parse::EnumSpec::*;
        match enm.spec {
            Alias { ref alias, .. } => {
                ConstValue::Enumref(alias, OptionMofifierFn::None)
            }
            Offset { offset, extnumber, dir, .. } => {
                let number = extnumber.unwrap_or_else(||ex.number.expect("error: enum extension must have a number"));
                let val = 1000000000 + (number - 1) * 1000 + offset;
                let negate = match dir {
                    true => Negate::False,
                    false => Negate::True,
                };
                return ConstValue::Offset(
                    val,
                    negate,
                );
            }
            Bitpos { bitpos, .. } => {
                use std::convert::TryInto;
                ConstValue::Bitpos(bitpos.try_into().expect("error: expecting 32 bit number for Flags (is this not a Flags type?)"))
            }
            Value { ref value, .. } => {
                if let Ok(val) = i32::from_str_radix(value, 10) {
                    ConstValue::Number(val)
                } else if value.starts_with('"') && value.ends_with('"') { // TODO: in future, if I remove vkxml entierly, then this can just keep the quotes as part of the value rather than removing them
                    ConstValue::Text(&value[1..value.len()-1])
                } else if value.starts_with("0x") { // probably a hex value
                    ConstValue::Hex(&value[2..])
                } else { // assume Cexpr
                    ConstValue::Cexpr(value)
                }
            }
            None => panic!("error: enum has no value, is this somhow an enumref?"),
            _ => panic!("unexpecxted unknown value"),
        }
    }
}

impl<'a> From<vkxml_visitor::VkxmlExtensionEnum<'a>> for ConstValue<'a> {
    fn from(vkxml_ex_enum: vkxml_visitor::VkxmlExtensionEnum<'a>) -> Self {
        if let Some(offset) = vkxml_ex_enum.enum_extension.offset {
            use std::convert::TryInto;
            let offset: i32 = offset.try_into().expect("error: offset cannot be i32");
            let val = 1000000000 + (vkxml_ex_enum.number - 1) * 1000 + offset;
            let negate = match vkxml_ex_enum.enum_extension.negate {
                true => Negate::True,
                false => Negate::False,
            };
            return ConstValue::Offset(
                val.try_into().expect("error: i32 to usize cannot fail"),
                negate,
            );
        }

        if let Some(num) = vkxml_ex_enum.enum_extension.number {
            return ConstValue::Number(num);
        }

        if let Some(ref hex_str) = vkxml_ex_enum.enum_extension.hex {
            return ConstValue::Hex(hex_str);
        }

        if let Some(bitpos) = vkxml_ex_enum.enum_extension.bitpos {
            return ConstValue::Bitpos(bitpos);
        }

        if let Some(ref expr) = vkxml_ex_enum.enum_extension.c_expression {
            return ConstValue::Cexpr(expr);
        }

        panic!("improper vkxml_ex_constant does not have a value");
    }
}
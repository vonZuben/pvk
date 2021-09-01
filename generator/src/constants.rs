
use quote::{quote, ToTokens};

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;

pub enum Expresion<'a> {
    Literal(String),
    CallExpresion(&'a str, String), // simplified call expression
}

impl ToTokens for Expresion<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Expresion::*;
        match self {
            Literal(lit) => quote!(#lit).to_tokens(tokens),
            // only need to support simple 1 param call
            // this is for initilizing single element tuple structs
            CallExpresion(callee, param1) => quote!( #callee(#param1) ).to_tokens(tokens),
        }
    }
}

pub struct Constant2<'a> {
    name: &'a str,
    ty: crate::ctype::Ctype<'a>,
    val: Expresion<'a>,
}

impl<'a> Constant2<'a> {
    pub fn new(name: &'a str, ty: crate::ctype::Ctype<'a>, val: Expresion<'a>) -> Self {
        Self {
            name,
            ty,
            val,
        }
    }
}

impl ToTokens for Constant2<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::utils::StrAsCode;
        let name = self.name.as_code();
        let ty = &self.ty;
        let val = &self.val;
        quote!(
            pub const #name: #ty = #val;
        ).to_tokens(tokens);
    }
}

pub fn make_vulkan_const_from_vkxml<'a>(vkxml_constant: &'a vkxml::Constant) -> Constant2<'a> {
    let name = &vkxml_constant.name;
    let ty = vkxml_constant_type(vkxml_constant);
    let val = vkxml_constant_value_expresion(vkxml_constant);
    Constant2::new(name, ty, val)
}

pub fn vkxml_constant_type(vkxml_constant: &vkxml::Constant) -> crate::ctype::Ctype<'static> {
    use crate::ctype::Ctype;

    if let Some(_) = vkxml_constant.number {
        return Ctype::new("usize");
    }

    if let Some(_) = vkxml_constant.hex {
        return Ctype::new("usize");
    }

    if let Some(_) = vkxml_constant.bitpos {
        // This only shows up for FlagBits (I think) which as a Flags type alias defined by Vulkan
        return Ctype::new("Flags");
    }

    if let Some(ref expr) = vkxml_constant.c_expression {
        return match expr {
            e if e.contains("ULL") => Ctype::new("u64"),
            e if e.contains("U") => Ctype::new("u32"),
            e if e.contains("f") => Ctype::new("f32"),
            _ => Ctype::new("usize"),
        };
    }

    panic!("improper vkxml_constant does not have a value");
}

pub fn vkxml_constant_value_string(vkxml_constant: &vkxml::Constant) -> String {
    use crate::ctype::Ctype;

    if let Some(num) = vkxml_constant.number {
        return num.to_string();
    }

    if let Some(ref hex_str) = vkxml_constant.hex {
        return format!("0x{:0>8}", hex_str);
    }

    if let Some(bitpos) = vkxml_constant.bitpos {
        // This only shows up for FlagBits (I think) which as a Flags type alias defined by Vulkan
        return format!("0x{:0>8X}", (1u32 << bitpos));
    }

    if let Some(ref expr) = vkxml_constant.c_expression {
        return expr
            .replace("ULL", "")
            .replace("U", "")
            .replace("~", "!")
            .replace("f", "");
    }

    panic!("improper vkxml_constant does not have a value");
}

fn vkxml_constant_value_expresion(vkxml_constant: &vkxml::Constant) -> Expresion<'static> {
    Expresion::Literal(vkxml_constant_value_string(vkxml_constant))
}


pub fn handle_constants(constants: &Constants) -> TokenStream {

    let q = constants.elements.iter().map(|constant| {
        let name = constant.name();
        let ty = constant.ty();
        let val = constant.val();
        quote!( pub const #name: #ty = #val; )
    });

    quote!( #( #q )* )

}

trait ConstExt {
    fn ty(&self) -> TokenStream;
    fn val(&self) -> TokenStream;
    fn name(&self) -> TokenStream;
}

impl ConstExt for vkxml::Constant {

    fn ty(&self) -> TokenStream {
        if self.name.contains("TRUE") || self.name.contains("FALSE") {
            return quote!(Bool32);
        }
        else { one_option!(

                &self.number , |_| quote!(usize) ;

                &self.hex , |_| quote!(usize) ;

                &self.bitpos , |_| panic!("error: trying to get bitpos type not implemented -> {}", self.name) ;

                &self.c_expression , |expr: &str| {
                    match &expr {
                        e if e.contains("ULL") => quote!(u64),
                        e if e.contains("U") => quote!(u32),
                        e if e.contains("f") => quote!(f32),
                        _ => quote!(usize),
                    }
                }
                )
        }

    }
    fn val(&self) -> TokenStream {
        one_option!(

            &self.number , |num: &i32| { num.to_string().as_code() } ;

            &self.hex , |hex_str| format!("0x{:0>8}", hex_str).as_code() ;

            &self.bitpos , |_| panic!("error: trying to take bit pos for constant not implemented") ;

            &self.c_expression , |expr: &String| {
                let v = expr
                    .replace("ULL", "")
                    .replace("U", "")
                    .replace("~", "!")
                    .replace("f", "")
                    .as_code();
                quote!(#v)

            } ;
        )
    }
    fn name(&self) -> TokenStream {
        self.name.as_code()
    }
}

use super::*;

pub struct Delegate;

impl<'a, I: Variants<'a>> ToTokensDelegate<I> for Delegate {
    fn delegate_to_tokens(params: &Properties<I>, tokens: &mut TokenStream) {
        let target = params.target;
        let variants_names = params
            .variants
            .clone()
            .filter(|v| !v.is_alias())
            .map(|v| v.name());

        let is_compressed = variants_names.clone().map(|v| v.contains("_BLOCK"));
        let is_multi_planar = variants_names.clone().map(|v| v.contains("PLANE_"));
        let has_depth_stencil: Vec<_> = variants_names
            .clone()
            .map(|v| has_depth_stencil(v.as_str()))
            .collect();
        let has_depth = has_depth_stencil.iter().map(|t| t.0);
        let has_stencil = has_depth_stencil.iter().map(|t| t.1);

        krs_quote_with!(tokens <-
            impl {@target} {
                pub const fn is_compressed_format(self) -> bool {
                    match self {
                        {@* Self::{@variants_names} => {@is_compressed}, }
                        _ => panic!("invalid Format"),
                    }
                }
                pub const fn is_multi_planar_format(self) -> bool {
                    match self {
                        {@* Self::{@variants_names} => {@is_multi_planar}, }
                        _ => panic!("invalid Format"),
                    }
                }
                pub const fn has_depth_component(self) -> bool {
                    match self {
                        {@* Self::{@variants_names} => {@has_depth}, }
                        _ => panic!("invalid Format"),
                    }
                }
                pub const fn has_stencil_component(self) -> bool {
                    match self {
                        {@* Self::{@variants_names} => {@has_stencil}, }
                        _ => panic!("invalid Format"),
                    }
                }
            }
        );
    }
}

// return (has_depth, has_stencil)
fn has_depth_stencil(name: &str) -> (bool, bool) {
    let mut has_depth = false;
    let mut has_stencil = false;

    let mut chars = name.chars();

    while let Some(c) = chars.next() {
        // look for D##_
        if c == 'D' {
            let mut has_num = false;
            while let Some(c) = chars.next() {
                if c.is_numeric() {
                    has_num = true;
                }
                if c == '_' {
                    if has_num {
                        has_depth = true;
                    }
                    break;
                }
            }
        }

        // look for S##_
        if c == 'S' {
            let mut has_num = false;
            while let Some(c) = chars.next() {
                if c.is_numeric() {
                    has_num = true;
                }
                if c == '_' {
                    if has_num {
                        has_stencil = true;
                    }
                    break;
                }
            }
        }
    }

    (has_depth, has_stencil)
}

use super::*;

pub struct FormatPropertiesDef;

impl EnumProperties for FormatPropertiesDef {
    fn name(&self, name: VkTyName) -> DynToTokens {
        Box::new(ToTokensClosure(move |tokens: &mut TokenStream| {
            let p_name = trait_name(name);
            tokens.push(p_name)
        }))
    }

    fn def(&self, name: VkTyName) -> DynToTokens {
        Box::new(ToTokensClosure(move |tokens: &mut TokenStream| {
            let p_name = trait_name(name);
            krs_quote_with!(tokens <-
                pub trait {@p_name} {
                    const COMPRESSED_FORMAT: bool;
                    const MULTI_PLANAR: bool;
                    const HAS_DEPTH_COMPONENT: bool;
                    const HAS_STENCIL_COMPONENT: bool;
                }
            )
        }))
    }

    fn variant(&self, name: VkTyName, target: VkTyName) -> DynToTokens {
        Box::new(ToTokensClosure(move |tokens: &mut TokenStream| {
            let p_name = trait_name(target);

            let compressed = name.contains("_BLOCK");
            let multi_planar = name.contains("PLANE_");
            let (has_depth, has_stencil) = has_depth_stencil(&name);

            krs_quote_with!(tokens <-
                impl {@p_name} for {@name} {
                    const COMPRESSED_FORMAT: bool = {@compressed};
                    const MULTI_PLANAR: bool = {@multi_planar};
                    const HAS_DEPTH_COMPONENT: bool = {@has_depth};
                    const HAS_STENCIL_COMPONENT: bool = {@has_stencil};
                }
            )
        }))
    }
}

fn trait_name(name: VkTyName) -> Token {
    Token::from(format!("{name}EnumProperties"))
}

// return (has_depth, has_stencil)
fn has_depth_stencil(name: &str) -> (bool, bool) {
    let mut has_depth = false;
    let mut has_stencil = false;

    let mut chars = name.chars().peekable();

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
                    break
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
                    break
                }
            }
        }
    }

    (has_depth, has_stencil)
}
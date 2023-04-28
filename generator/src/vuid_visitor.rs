mod vuid_parser;

use vuid_parser::VuidParser;

pub trait VuidVisitor<'a> {
    fn visit_vuid(&mut self, vuid: Vuid<'a>);
}

pub fn visit_vuids<'a>(vuids: impl VuidParser<'a>, visitor: &mut impl VuidVisitor<'a>) {
    for vuid in vuids.iter() {
        println!("{vuid:?}");
    }
}

pub struct Vuid<'a> {
    _p: std::marker::PhantomData<&'a ()>
}
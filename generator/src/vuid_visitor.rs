mod vuid_parser;

use vuid_parser::VuidParser;

pub use vuid_parser::{json::VuidJsonStrParser, VuidPair};

pub trait VuidVisitor<'a> {
    fn visit_vuid(&mut self, vuid: VuidPair<'a>);
    // fn visit_vuid_version(&mut self, version)
}

pub fn visit_vuids<'a>(vuids: impl VuidParser<'a>, visitor: &mut impl VuidVisitor<'a>) {
    for vuid in vuids.iter() {
        visitor.visit_vuid(vuid);
    }
}
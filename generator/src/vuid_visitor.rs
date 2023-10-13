mod vuid_parser;

use vuid_parser::VuidParser;

pub use vuid_parser::{json::VuidJsonStrParser, VuidPair};

pub trait VuidVisitor<'a> {
    fn visit_vuid(&mut self, vuid: VuidPair<'a>);
    fn visit_vuid_version(&mut self, version: (u32, u32, u32));
}

pub fn visit_vuids<'a>(parser: impl VuidParser<'a>, visitor: &mut impl VuidVisitor<'a>) {
    parser.parse_with(visitor)
}

pub mod json;

use std::borrow::Cow;

/// A VUID by name and description
///
/// #internal detail
/// the fields use Cow to allow for borrowed and owned strings for flexibility with possible parer implementations
/// there is a json format where the descriptions have html, and I would want to remove the html in my code, so this
/// will mean creating new Strings with the html removed.
///
/// there is also a version of the vuids available from lunarG that does not include the html, and could be used directly as borrowed data
/// However, it is also a little harder to obtain the vuids from lunarG
///
/// I will experiment with the json first (and probably stick with it, but who knows for the future)
#[derive(Debug)]
pub struct VuidPair<'a> {
    /// the name of the VUID (e.g. VUID-vkGetInstanceProcAddr-instance-parameter)
    name: Cow<'a, str>,
    /// the description of the VUID
    description: Cow<'a, str>,
}

impl VuidPair<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> &str {
        &self.description
    }
}

pub trait VuidParser<'a> {
    fn parse_with(self, visitor: &mut impl super::VuidVisitor<'a>);
}
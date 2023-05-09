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
    fn next_vuid(&mut self) -> Option<VuidPair<'a>>;
    fn iter(self) -> VuidParserIter<'a, Self> where Self: Sized {
        VuidParserIter(self, std::marker::PhantomData)
    }
}

pub struct VuidParserIter<'a, V: VuidParser<'a>>(V, std::marker::PhantomData<&'a str>);

impl<'a, V: VuidParser<'a>> Iterator for VuidParserIter<'a, V> {
    type Item = VuidPair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_vuid()
    }
}
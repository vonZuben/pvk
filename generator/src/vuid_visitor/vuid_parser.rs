use std::borrow::Cow;
use std::io::Read;

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

pub struct VuidJsonStrParser<'a> {
    json: &'a str,
}

impl<'a> VuidJsonStrParser<'a> {
    pub fn new(json: &'a str) -> Self {
        Self { json }
    }
}

impl<'a> VuidParser<'a> for VuidJsonStrParser<'a> {
    fn next_vuid(&mut self) -> Option<VuidPair<'a>> {
        let mut line = &self.json[..0];
        for (i, c) in self.json.chars().enumerate() {
            if c == '\n' {
                line = &self.json[..i];
            }
        }

        todo!()
    }
}
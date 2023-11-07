use super::{VuidPair, VuidParser};

/// This is a simple parser for the validusage.json file provided in https://github.com/KhronosGroup/Vulkan-Headers
///
/// #IMPORTANT NOTE
/// the expected file includes "vuid"/"text" pairs in adjacent lines
/// rather than full json parsing, this just takes advantage of such line based layout
pub struct VuidJsonStrParser<'a> {
    json: &'a str,
}

impl<'a> VuidJsonStrParser<'a> {
    pub fn new(json: &'a str) -> Self {
        Self { json }
    }
}

impl<'a> VuidParser<'a> for VuidJsonStrParser<'a> {
    fn parse_with(mut self, visitor: &mut impl crate::vuid_visitor::VuidVisitor<'a>) {
        let mut get_line = || {
            for (i, c) in self.json.bytes().enumerate() {
                if c == b'\n' {
                    let line = &self.json[..i];
                    self.json = &self.json[i + 1..];
                    return Some(line);
                }
            }
            None
        };

        let mut supported_schema = false;

        // parse all lines
        while let Some(line) = get_line() {
            let line = line.trim();

            const VUID_TAG: &'static str = "\"vuid\": ";
            const TEXT_TAG: &'static str = "\"text\": ";
            const SCHEMA_TAG: &'static str = "\"schema version\": ";
            const API_VER_TAG: &'static str = "\"api version\": ";

            // when find vuid line
            if line.starts_with(VUID_TAG) {
                // get "vuid" value which is the name of the vuid
                let vuid_name = &line[VUID_TAG.len() + 1..line.len() - 2]; // this takes the value without the quotation marks and ending comma

                // then get next line and assert that it is the "text" value, which is the description
                let line = get_line()
                    .expect("error: no line after vuid line")
                    .trim()
                    .strip_prefix(TEXT_TAG)
                    .expect("error: line after 'vuid' is not 'text'");

                let start = line
                    .find("\"")
                    .expect("ERROR: vuid description does not start with a '\"'")
                    + 1;
                let end = line
                    .rfind("\"")
                    .expect("ERROR: vuid description does not end with a '\"'");
                assert!(start != end);

                let vuid_description = line[start..end].trim(); // remove quotation marks and whitespace leading/trailing

                // remove HTML from the description
                let filtered_description: String = HtmlFilter(vuid_description.chars()).collect();

                let vuid_pair = VuidPair {
                    name: vuid_name.into(),
                    description: filtered_description.into(),
                };

                visitor.visit_vuid(vuid_pair);
            } else if line.starts_with(SCHEMA_TAG) {
                let schema_version: u32 = line[SCHEMA_TAG.len()..line.len() - 1]
                    .parse()
                    .expect("error: can't parse json schema version");
                if schema_version == 2 {
                    supported_schema = true;
                } else {
                    panic!("unsupported validusage.json schema version");
                }
            } else if line.starts_with(API_VER_TAG) {
                let api_version_str = &line[API_VER_TAG.len() + 1..line.len() - 2];

                let mut version_parts = api_version_str.split(".");

                let major = version_parts
                    .next()
                    .expect("error: api version - no major")
                    .parse()
                    .expect("error: can't parse major");
                let minor = version_parts
                    .next()
                    .expect("error: api version - no minor")
                    .parse()
                    .expect("error: can't parse minor");
                let patch = version_parts
                    .next()
                    .expect("error: api version - no patch")
                    .parse()
                    .expect("error: can't parse patch");

                visitor.visit_vuid_version((major, minor, patch));
            }
        }

        if !supported_schema {
            panic!("error: never found supported schema version");
        }
    }
}

/// This is simple html tag filter
///
/// this is very naive implementation that does not account for special escape characters in tags (if thats even possible)
/// just assume everything between '<' and '>' is a tag and filter it out
struct HtmlFilter<I>(I);

impl<I: Iterator<Item = char>> Iterator for HtmlFilter<I> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.0.next()?;
        if c == '<' {
            while let Some(c) = self.0.next() {
                if c == '>' {
                    return self.next(); // this intentionally recursively call Self::next since we need to continue to check for '<'
                }
            }
            panic!("error: HtmlFilter did not find end og html tag")
        } else {
            Some(c)
        }
    }
}

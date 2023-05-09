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
    fn next_vuid(&mut self) -> Option<VuidPair<'a>> {
        if self.json.is_empty() {
            return None;
        }

        let mut get_line = || {
            for (i, c) in self.json.chars().enumerate() {
                if c == '\n' {
                    let line = &self.json[..i];
                    self.json = &self.json[i+1..];
                    return Some(line)
                }
            }
            None
        };

        // parse all lines
        while let Some(line) = get_line() {
            let line = line.trim();

            const VUID_TAG: &'static str = "\"vuid\": ";
            const TEXT_TAG: &'static str = "\"text\": ";

            // when find vuid line
            if line.starts_with(VUID_TAG) {
                // get "vuid" value which is the name of the vuid
                let vuid_name = &line[VUID_TAG.len()+1..line.len()-2]; // this takes the value without the quotation marks and ending comma

                // then get next line and assert that it is the "text" value, which is the description
                let line = get_line()
                    .expect("error: no line after vuid line")
                    .trim()
                    .strip_prefix(TEXT_TAG)
                    .expect("error: line after 'vuid' is not 'text'");

                let vuid_description = line[1..line.len()-1].trim(); // remove quotation marks

                let vuid_pair = VuidPair {
                    name: vuid_name.into(),
                    description: vuid_description.into(),
                };

                return Some(vuid_pair);
            }
        }

        None
    }
}
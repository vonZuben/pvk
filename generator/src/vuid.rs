use crate::utils::VkTyName;
use crate::vuid_visitor::VuidPair;
use krs_quote::{krs_quote_with, ToTokens};

use crate::utils::{StrAsCode, VecMap};

type ApiVersion = (u32, u32, u32);

#[derive(Default)]
pub struct Vuids<'a> {
    collections: VecMap<VkTyName, TargetVuids<'a>>,
    api_version: Option<ApiVersion>,
}

impl<'a> Vuids<'a> {
    pub fn api_version(&mut self, api_version: ApiVersion) {
        self.api_version = Some(api_version);
    }
    pub fn insert_vuid(&mut self, target: VkTyName, pair: VuidPair<'a>) {
        self.collections
            .get_mut_or_default_with(target, || TargetVuids::new(target))
            .vuid_pairs
            .push(pair);
    }
}

impl ToTokens for Vuids<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let (major, minor, patch) = self.api_version.expect("error: vuid api version never set");
        let collections = self.collections.iter();

        krs_quote_with!(tokens <-
            use std::collections::HashMap;

            const API_VERSION: (u32, u32, u32) = ({@major}, {@minor}, {@patch});

            pub type Target = &'static str;
            pub type Vuid = &'static str;
            pub type Description = &'static str;
            pub type VuidGroup = (Target, &'static [(Vuid, Description)]);

            static VUID_GROUPS: &'static [VuidGroup] = [{@,* {@collections}}].as_slice();

            pub fn get_vuids() -> &'static [VuidGroup] {
                VUID_GROUPS
            }

        );
    }
}

pub struct TargetVuids<'a> {
    target: VkTyName,
    vuid_pairs: Vec<VuidPair<'a>>,
}

impl<'a> TargetVuids<'a> {
    pub fn new(target: VkTyName) -> Self {
        Self {
            target,
            vuid_pairs: Vec::new(),
        }
    }
}

impl ToTokens for TargetVuids<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let target = self.target.as_str();
        let vuid_names = self
            .vuid_pairs
            .iter()
            .map(|p| p.name().replace("-", "_").replace("::", "_"));
        let descriptions = self.vuid_pairs.iter().map(|p| p.description());
        // let docs = descriptions
        //     .clone()
        //     .map(|desc| DocFormatFilter::new(desc).into_iter().collect::<String>());

        krs_quote_with!(tokens <-

            ({@target}, [{@,* ({@vuid_names}, {@descriptions}) }].as_slice())

        );
    }
}

/// Need to ensure proper format in docs
///
/// for now, this just replaces [text] with [[text]()] to render properly and not be a broken link
struct DocFormatFilter<'a> {
    text: &'a str,
    brackets: Brackets,
}

impl<'a> DocFormatFilter<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            brackets: Brackets::No,
        }
    }
}

enum Brackets {
    No,
    Start,
    In(usize),
    End,
}

impl<'a> Iterator for DocFormatFilter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.len() == 0 {
            None
        } else {
            match self.brackets {
                Brackets::No => match self.text.find('[') {
                    Some(until) => {
                        self.brackets = Brackets::Start;
                        let ret = &self.text[..until];
                        self.text = &self.text[until..];
                        Some(ret)
                    }
                    None => {
                        let ret = self.text;
                        self.text = &"";
                        Some(ret)
                    }
                },
                Brackets::Start => {
                    // check for an end ']' in advance because there have been docs with typos. If typo is found, just maintain it
                    match self.text.find("]") {
                        Some(until) => {
                            self.text = &self.text[1..]; // represent consuming '['
                            self.brackets = Brackets::In(until - 1); // subtract 1 from until since we advanced the text by 1
                            Some("[[")
                        }
                        None => {
                            self.brackets = Brackets::No;
                            let ret = self.text;
                            self.text = &"";
                            Some(ret)
                        }
                    }
                }
                Brackets::In(until) => {
                    self.brackets = Brackets::End;
                    let ret = &self.text[..until];
                    self.text = &self.text[until..];
                    Some(ret)
                }
                Brackets::End => {
                    self.text = &self.text[1..]; // represent consuming ']'
                    self.brackets = Brackets::No;
                    Some("]()]")
                }
            }
        }
    }
}

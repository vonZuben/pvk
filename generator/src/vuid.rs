use krs_quote::{ToTokens, krs_quote_with};
use crate::utils::VkTyName;
use crate::vuid_visitor::VuidPair;

use crate::utils::{VecMap, StrAsCode};

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
        self.collections.get_mut_or_default_with(target, ||TargetVuids::new(target)).vuid_pairs.push(pair);
    }
}

impl ToTokens for Vuids<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let (major, minor, patch) = self.api_version.expect("error: vuid api version never set");
        let collections = self.collections.iter();

        krs_quote_with!(tokens <-
            pub mod validation {
                /// vuid api version
                const API_VERSION: (u32, u32, u32) = ({@major}, {@minor}, {@patch});
                {@* {@collections}}
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
        Self { target, vuid_pairs: Vec::new() }
    }
}

impl ToTokens for TargetVuids<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let target = self.target;
        let vuid_names = self.vuid_pairs.iter().map(|p| p.name().replace("-", "_").as_code());
        let descriptions = self.vuid_pairs.iter().map(|p| p.description());
        let docs = descriptions.clone().map(|desc| DocFormatFilter::new(desc).into_iter().collect::<String>());

        krs_quote_with!(tokens <-

            #[allow(non_upper_case_globals)]
            pub mod {@target} {
                // output trait that should be implemented for vuid checks
                pub trait Vuids {
                    {@*
                        #[doc = {@docs}]
                        const {@vuid_names}: ();
                    }
                }
                pub fn validate<V: Vuids>(_: V) {
                    {@*
                        let _ = V::{@vuid_names};
                    }
                }
                {@*
                    pub const {@vuid_names}: &'static [u8] = {@descriptions}.as_bytes();
                }
            }

        );
    }
}

/// Need to ensure proper format in docs
///
/// for now, this just replaces [text] with [[text]()] to render properly and not be a broken link
struct DocFormatFilter<'a>{
    text: &'a str,
    brackets: Brackets,
}

impl<'a> DocFormatFilter<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, brackets: Brackets::No }
    }
}

enum Brackets {
    No,
    Start,
    In,
    End,
}

impl<'a> Iterator for DocFormatFilter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.len() == 0 {
            None
        }
        else {
            match self.brackets {
                Brackets::No => {
                    match self.text.find('[') {
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
                    }
                }
                Brackets::Start => {
                    self.text = &self.text[1..]; // represent consuming '['
                    self.brackets = Brackets::In;
                    Some("[[")
                }
                Brackets::In => {
                    match self.text.find(']') {
                        Some(until) => {
                            self.brackets = Brackets::End;
                            let ret = &self.text[..until];
                            self.text = &self.text[until..];
                            Some(ret)
                        }
                        None => {
                            panic!("error: DocFormatFilter can't find ']'")
                        }
                    }
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
use crate::utils::VkTyName;
use krs_quote::{krs_quote_with, to_tokens_closure, ToTokens};

pub(crate) fn dependencies_to_tokens<'a>(
    features: &'a crate::features::FeatureCollection,
    extensions: &'a crate::extensions::ExtensionCollection,
) -> impl ToTokens + use<'a> {
    let feature_deps = krs_quote::to_tokens_closure!(tokens {
        let mut previous_features: Vec<(VkTyName, crate::features::FeatureVersion)> = Vec::new();

        for feature in features.features().rev() {
            let number = crate::features::parse_version(feature.as_str());
            match previous_features.last() {
                Some(prev) => {
                    if (*prev).1 > number {
                        previous_features.push((feature, number));
                    }
                    else {
                        panic!("relation between version {:?} and {:?} is not clear at this time", prev.1, number);
                    }
                }
                None => previous_features.push((feature, number)),
            }

            let bounds = previous_features.iter().map(|p|p.0);
            krs_quote_with!(tokens <-
                #[marker]
                pub trait {@feature} {}
                {@*
                    impl<T> {@feature} for T where T: crate::version::{@bounds} {}
                }
            )
        }
    });

    let extension_deps = krs_quote::to_tokens_closure!(tokens {
        for extension in extensions.extensions() {
            let name = extension.name();
            let promoted = extension.promoted_to().into_iter().map(|promoted| {
                krs_quote::to_tokens_closure!(tokens {
                    if extensions.find(promoted).is_some() {
                        krs_quote_with!(tokens <-
                            crate::extension::{@promoted}
                        )
                    }
                    else {
                        krs_quote_with!(tokens <-
                            crate::dependency::{@promoted}
                        )
                    }
                })
            });

            krs_quote_with!(tokens <-
                #[marker]
                pub trait {@name} {}
                impl<T> {@name} for T where T: crate::extension::{@name} {}
                {@*
                    impl<T> {@name} for T where T: {@promoted} {}
                }
            )
        }
    });

    to_tokens_closure!(tokens {
        krs_quote_with!(tokens <-
            pub mod dependency {
                #![allow(non_camel_case_types)]
                {@feature_deps}
                {@extension_deps}
            }
        )
    })
}

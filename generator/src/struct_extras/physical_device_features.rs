use crate::utils::case;
use crate::utils::StrAsCode;
use crate::utils::VkTyName;

#[derive(Default)]
pub struct PhysicalDeviceFeatures {
    features: Vec<PhysicalDeviceFeaturesInner>,
}

struct PhysicalDeviceFeaturesInner {
    struct_name: VkTyName,
    features: Vec<VkTyName>,
}

impl super::ExtrasDelegate for PhysicalDeviceFeatures {
    fn delegate_to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let s_names = self.features.iter().map(|f| f.struct_name);
        let s_features = self.features.iter().map(|f| {
            krs_quote::ToTokensClosure(|tokens: &mut krs_quote::TokenStream| {
                let s_name = f.struct_name;
                let features = f
                    .features
                    .iter()
                    .copied()
                    .filter(is_feature_field)
                    .map(|f| case::camel_to_snake(f.as_str()).as_code());

                krs_quote::krs_quote_with!(tokens <-
                    {@*
                        fn {@features}(&self) -> bool {
                            self.{@s_name}.{@features}
                        }
                    }
                )
            })
        });

        let mut all_feature_names: Vec<_> = self
            .features
            .iter()
            .flat_map(|f| f.features.iter().copied().filter(is_feature_field))
            .collect();

        all_feature_names.sort();
        all_feature_names.dedup();

        let all_feature_names = all_feature_names
            .iter()
            .map(|n| case::camel_to_snake(&n).as_code());

        krs_quote::krs_quote_with!(tokens <-

            {@*
                #[macro_export]
                macro_rules! {@s_names} {
                    () => {
                        {@s_features}
                    }
                }
            }

            pub trait CheckPhysicalDeviceFeatures {
                {@*
                    fn {@all_feature_names}(&self) -> bool {
                        false
                    }
                }
            }
        )
    }

    fn delegate_check_bespoke(&mut self, s: &crate::types::Struct2) {
        if is_physical_device_features(s) {
            let inner = PhysicalDeviceFeaturesInner {
                struct_name: s.name(),
                features: s.fields().map(|f| f.name).collect(),
            };

            self.features.push(inner);
        }
    }
}

fn is_physical_device_features(s: &super::Struct2) -> bool {
    let extends_features = s
        .extends()
        .find(|e| e.as_str() == "VkPhysicalDeviceFeatures2")
        .is_some();
    let is_features = s.name().as_str() == "VkPhysicalDeviceFeatures";

    is_features || extends_features
}

fn is_feature_field(name: &VkTyName) -> bool {
    let is_stype = name.as_str() == "sType";
    let is_pnext = name.as_str() == "pNext";

    !is_stype && !is_pnext
}

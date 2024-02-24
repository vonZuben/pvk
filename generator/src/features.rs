use std::collections::HashMap;

use krs_quote::krs_quote_with;

use crate::utils::{StrAsCode, VecMap, VkTyName};

// Feature Collection is for keeping track of different feature Versions
// Since the Vulkan spec defines each Feature as additions/deletions (requires/remove)
// on the previous version, then it is necessary to keep track of all feature versions
// previous to a specific version

#[derive(Default)]
pub struct FeatureCollection {
    versions: VecMap<VkTyName, Feature>,
}

impl FeatureCollection {
    // needs to be called in order of versions
    // will automatically make a new Feature collection based on the previous version
    // when a new version is passed
    pub fn modify_with(&mut self, version: impl Into<VkTyName>, f: impl FnOnce(&mut Feature)) {
        let version = version.into();
        match self.versions.get_mut(version) {
            Some(fc) => f(fc),
            None => {
                let mut fc = match self.versions.last() {
                    Some(previous_feature) => previous_feature.as_new_version(version),
                    None => Feature::new(version),
                };
                f(&mut fc);
                self.versions.push(version, fc);
            }
        }
    }
}

impl krs_quote::ToTokens for FeatureCollection {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let versions = self.versions.iter();

        // structs
        let instance_structs = versions.clone().map(|v| VersionStruct {
            name: v.version,
            commands: &v.instance_command_names,
        });
        let device_structs = versions.clone().map(|v| VersionStruct {
            name: v.version,
            commands: &v.device_command_names,
        });
        let entry_structs = versions.clone().map(|v| VersionStruct {
            name: v.version,
            commands: &v.entry_command_names,
        });

        // traits
        let instance_traits = versions.clone().map(|v| VersionTrait {
            name: v.version,
            commands: &v.instance_command_names,
        });
        let device_traits = versions.clone().map(|v| VersionTrait {
            name: v.version,
            commands: &v.device_command_names,
        });
        let entry_traits = versions.clone().map(|v| VersionTrait {
            name: v.version,
            commands: &v.entry_command_names,
        });

        // macros
        let instance_macros = versions.clone().map(|v| VersionMacros {
            name: v.version,
            mod_name: "instance",
            commands: &v.instance_command_names,
        });
        let device_macros = versions.clone().map(|v| VersionMacros {
            name: v.version,
            mod_name: "device",
            commands: &v.device_command_names,
        });
        let entry_macros = versions.clone().map(|v| VersionMacros {
            name: v.version,
            mod_name: "entry",
            commands: &v.entry_command_names,
        });

        let version_values = versions.clone().map(|v| VersionValues { feature: v });

        krs_quote_with!(tokens <-
            #[doc(hidden)]
            pub mod version {
                pub mod numbers {
                    {@* {@version_values}}
                }
                pub mod instance {
                    use super::super::has_command::*;
                    {@* {@instance_traits}}
                    {@* {@instance_macros}}
                    pub mod structs {
                        use super::super::super::*;
                        {@* {@instance_structs}}
                    }
                }

                pub mod device {
                    use super::super::has_command::*;
                    {@* {@device_traits}}
                    {@* {@device_macros}}
                    pub mod structs {
                        use super::super::super::*;
                        {@* {@device_structs}}
                    }
                }

                pub mod entry {
                    use super::super::has_command::*;
                    {@* {@entry_traits}}
                    {@* {@entry_macros}}
                    pub mod structs {
                        use super::super::super::*;
                        {@* {@entry_structs}}
                    }
                }
            }
        )
    }
}

struct VersionStruct<'a> {
    name: VkTyName,
    commands: &'a [RequireRemove],
}

impl krs_quote::ToTokens for VersionStruct<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let command = self.commands.iter().filter(|r| r.is_require());

        krs_quote_with!(tokens <-
            pub struct {@name} {
                {@* pub {@command}: {@command}, }
            }

            impl {@name} {
                pub fn load(loader: impl FunctionLoader) -> std::result::Result<Self, CommandLoadError> {
                    Ok(Self {
                        {@* {@command} : {@command}::load(loader)?, }
                    })
                }
            }
        );
    }
}

struct VersionTrait<'a> {
    name: VkTyName,
    commands: &'a [RequireRemove],
}

impl krs_quote::ToTokens for VersionTrait<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let commands = self.commands.iter().filter(|c| c.is_require());

        krs_quote_with!(tokens <-
            pub trait {@name} : {@+* {@commands}} {}
            impl<T> {@name} for T where T: {@+* {@commands}} {}
        );
    }
}

struct VersionMacros<'a> {
    name: VkTyName,
    mod_name: &'a str,
    commands: &'a [RequireRemove],
}

impl krs_quote::ToTokens for VersionMacros<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let commands = self.commands.iter().filter(|c| c.is_require());
        let macro_name = format!("{}_{}", name, self.mod_name).as_code();
        krs_quote_with!(tokens <-
            #[doc(hidden)]
            #[macro_export]
            macro_rules! {@macro_name} {
                ( $target:ident ) => {
                    {@* $crate::{@commands}!($target {@name}); }
                }
            }
            pub use {@macro_name} as {@name};
        );
    }
}

struct VersionValues<'a> {
    feature: &'a Feature,
}

impl krs_quote::ToTokens for VersionValues<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.feature.version;
        let version = parse_version(self.feature.version.as_str());

        krs_quote_with!(tokens <-
            const {@name}: (u32, u32, u32) = {@version};
        )
    }
}

// =================================================================
#[derive(Copy, Clone)]
enum RequireRemove {
    Require(VkTyName),
    Remove(VkTyName),
}

impl RequireRemove {
    fn require(name: VkTyName) -> Self {
        RequireRemove::Require(name)
    }
    fn remove(&mut self) {
        use RequireRemove::*;
        match self {
            Require(name) => *self = RequireRemove::Remove(*name),
            Remove(_) => {}
        }
    }
    fn is_require(&self) -> bool {
        matches!(self, RequireRemove::Require(_))
    }
}

impl krs_quote::ToTokens for RequireRemove {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        use RequireRemove::*;
        match self {
            Require(name) => {
                krs_quote_with!(tokens <- {@name}  );
            }
            Remove(_) => panic!("should not turn Remove into code"),
        }
    }
}

// =================================================================
// for keeping track of which list the command is in
#[derive(Copy, Clone)]
enum List {
    Instance(usize),
    Device(usize),
    Entry(usize),
}

// =================================================================
/// Command Names for a given version
/// intended to generate code within a instance/device command_names module
#[derive(Clone)]
pub struct Feature {
    version: VkTyName,
    instance_command_names: Vec<RequireRemove>,
    device_command_names: Vec<RequireRemove>,
    entry_command_names: Vec<RequireRemove>,
    // internal for quickly converting Require commands into Remove Commands
    vec_map: HashMap<VkTyName, List>,
}

impl Feature {
    pub fn new(version: impl Into<VkTyName>) -> Self {
        let version = version.into();
        Self {
            version,
            instance_command_names: Default::default(),
            device_command_names: Default::default(),
            entry_command_names: Default::default(),
            vec_map: Default::default(),
        }
    }
    pub fn as_new_version(&self, version: impl Into<VkTyName>) -> Self {
        let mut new_version = self.clone();
        new_version.version = version.into();
        new_version
    }
    pub fn push_instance_command(&mut self, command: impl Into<VkTyName>) {
        // insert index of to-be-inserted instance command and ensure not already there
        let command = command.into();
        assert!(self
            .vec_map
            .insert(command, List::Instance(self.instance_command_names.len()))
            .is_none());
        self.instance_command_names
            .push(RequireRemove::require(command));
    }
    pub fn push_device_command(&mut self, command: impl Into<VkTyName>) {
        // insert index of to-be-inserted instance command and ensure not already there
        let command = command.into();
        assert!(self
            .vec_map
            .insert(command, List::Device(self.device_command_names.len()))
            .is_none());
        self.device_command_names
            .push(RequireRemove::require(command));
    }
    pub fn push_entry_command(&mut self, command: impl Into<VkTyName>) {
        // insert index of to-be-inserted instance command and ensure not already there
        let command = command.into();
        assert!(self
            .vec_map
            .insert(command, List::Entry(self.entry_command_names.len()))
            .is_none());
        self.entry_command_names
            .push(RequireRemove::require(command));
    }
    pub fn remove_command(&mut self, command: impl Into<VkTyName>) {
        let command = command.into();
        match self.vec_map.get(&command) {
            Some(List::Instance(index)) => self.instance_command_names[*index].remove(),
            Some(List::Device(index)) => self.device_command_names[*index].remove(),
            Some(List::Entry(index)) => self.entry_command_names[*index].remove(),
            None => panic!("should not be trying to remove command that was never required"),
        }
    }
}

fn parse_version(ver: &str) -> FeatureVersion {
    let mut tokens = ver.split('_');

    // assert that first text is equal to VK and VERSION
    tokens
        .next()
        .map(|version| assert_eq!(version, "VK"))
        .expect("Error parsing version, no 'VK' ...");
    tokens
        .next()
        .map(|version| assert_eq!(version, "VERSION"))
        .expect("Error parsing version, no 'VERSION' ...");
    let major = tokens
        .next()
        .expect("error: parsing version can't get major number");
    let minor = tokens
        .next()
        .expect("error: parsing version can't get minor number");

    // Note: I am assuming that the major and minor that are parsed are integers

    FeatureVersion {
        major: major.parse().expect("error: major not number"),
        minor: minor.parse().expect("error: minor not number"),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FeatureVersion {
    major: usize,
    minor: usize,
}

impl krs_quote::ToTokens for FeatureVersion {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let major = self.major;
        let minor = self.minor;
        krs_quote_with!(tokens <- ({@major}, {@minor}, 0) );
    }
}

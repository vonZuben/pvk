use std::collections::HashMap;

use krs_quote::krs_quote_with;

use crate::utils::{VecMap, VkTyName};

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

        let v_name = versions.clone().map(|v| v.version);
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

        krs_quote_with!(tokens <-
            {@* {@versions}}

            #[doc(hidden)]
            pub mod version {
                pub mod instance {
                    use super::super::{LoadCommands, CommandLoadError, FunctionLoader};
                    {@* {@instance_structs}}
                    pub mod provider {
                        {@*
                            pub trait {@v_name} {
                                fn {@v_name}(&self) -> &super::{@v_name};
                            }
                        }
                    }
                }

                pub mod device {
                    use super::super::{LoadCommands, CommandLoadError, FunctionLoader};
                    {@* {@device_structs}}
                    pub mod provider {
                        {@*
                            pub trait {@v_name} {
                                fn {@v_name}(&self) -> &super::{@v_name};
                            }
                        }
                    }
                }

                pub mod entry {
                    use super::super::{LoadCommands, CommandLoadError, FunctionLoader};
                    {@* {@entry_structs}}
                    pub mod provider {
                        {@*
                            pub trait {@v_name} {
                                fn {@v_name}(&self) -> &super::{@v_name};
                            }
                        }
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
                {@* {@command}: super::super::{@command}, }
            }

            impl {@name} {
                pub fn load(loader: impl FunctionLoader) -> std::result::Result<Self, CommandLoadError> {
                    Ok(Self {
                        {@* {@command} : super::super::{@command}::load(loader)?, }
                    })
                }
            }

            {@*
                impl<T: provider::{@name}> super::super::command::{@command}<super::super::{@name}> for T {
                    fn {@command}(&self) -> super::super::{@command} {
                        self.{@name}().{@command}
                    }
                }
            }
        );
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

impl krs_quote::ToTokens for Feature {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let version = self.version.as_code();

        let version_triple = parse_version(&self.version);

        krs_quote_with!( tokens <-
            #[derive(Debug)]
            pub struct {@version};
            impl VulkanVersion for {@version} {
                const VersionTriple: (u32, u32, u32) = {@version_triple};
                type InstanceCommands = version::instance::{@version};
                type DeviceCommands = version::device::{@version};
                type EntryCommands = version::entry::{@version};
            }
        );
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

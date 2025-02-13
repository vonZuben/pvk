use std::collections::HashMap;

use krs_quote::krs_quote_with;

use crate::utils::{StrAsCode, VecMap, VkTyName};

// Feature Collection is for keeping track of different feature Versions
// Since the Vulkan spec defines each Feature as additions/deletions (requires/remove)
// on the previous version, then it is necessary to keep track of all feature versions
// previous to a specific version

#[derive(Default)]
pub(crate) struct FeatureCollection {
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

    pub fn feature_names_iter(&self) -> impl Iterator<Item = &str> + Clone {
        self.versions.iter().map(|f| f.version.as_str())
    }

    pub fn features(&self) -> impl DoubleEndedIterator<Item = VkTyName> + Clone + use<'_> {
        self.versions.iter().map(|f| f.version)
    }
}

impl krs_quote::ToTokens for FeatureCollection {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let versions = self.versions.iter();

        // trait
        let traits = versions.clone().map(|v| VersionTrait { name: v.version });

        // structs
        let instance_command_structs = versions.clone().map(|v| VersionStruct {
            name: v.version,
            commands: &v.instance_command_names,
            label_trait: "InstanceLabel",
            command_method: "instance_commands",
        });
        let device_command_structs = versions.clone().map(|v| VersionStruct {
            name: v.version,
            commands: &v.device_command_names,
            label_trait: "DeviceLabel",
            command_method: "device_commands",
        });

        let version_values = versions.clone().map(|v| VersionValues { feature: v });

        krs_quote_with!(tokens <-
            #[doc(hidden)]
            pub mod version {
                pub mod numbers {
                    {@* {@version_values}}
                }

                use crate::Version;
                {@* {@traits}}

                pub mod instance_command_structs {
                    use crate::LoadCommands;
                    {@* {@instance_command_structs}}
                }

                pub mod device_command_structs {
                    use crate::LoadCommands;
                    {@* {@device_command_structs}}
                }
            }
        )
    }
}

struct VersionStruct<'a> {
    name: VkTyName,
    commands: &'a [RequireRemove],
    label_trait: &'a str,
    command_method: &'a str,
}

impl krs_quote::ToTokens for VersionStruct<'_> {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;
        let command = self.commands.iter().filter(|r| r.is_require());
        let label_trait = self.label_trait.as_code();
        let command_method = self.command_method.as_code();

        krs_quote_with!(tokens <-
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            pub struct {@name} {
                {@*
                    pub {@command}: crate::{@command},
                }
            }

            impl {@name} {
                pub fn load(loader: impl crate::FunctionLoader) -> std::result::Result<Self, crate::CommandLoadError> {
                    Ok(Self {
                        {@* {@command} : crate::{@command}::load(loader)?, }
                    })
                }
            }

            {@*
                impl<T> crate::has_command::{@command}<{@name}> for T
                    where T: super::{@name} + crate::{@label_trait}
                {
                    fn {@command}(&self) -> crate::{@command} {
                        self.{@command_method}().{@command}
                    }
                }
            }
        );
    }
}

struct VersionTrait {
    name: VkTyName,
}

impl krs_quote::ToTokens for VersionTrait {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let name = self.name;

        krs_quote_with!(tokens <-
            #[allow(non_camel_case_types)]
            pub unsafe trait {@name} : Version {
                fn instance_commands(&self) -> &instance_command_structs::{@name} where Self: crate::InstanceLabel {
                    unreachable!();
                }

                fn device_commands(&self) -> &device_command_structs::{@name} where Self: crate::DeviceLabel {
                    unreachable!();
                }
            }
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
            pub const {@name}: (u32, u32, u32) = {@version};
        )
    }
}

// =================================================================
#[derive(Copy, Clone)]
enum RequireRemove {
    Require(VkTyName),
    #[allow(dead_code)]
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

pub(crate) fn parse_version(ver: &str) -> FeatureVersion {
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct FeatureVersion {
    major: usize,
    minor: usize,
}

impl std::fmt::Debug for FeatureVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.x", self.major, self.minor)
    }
}

impl krs_quote::ToTokens for FeatureVersion {
    fn to_tokens(&self, tokens: &mut krs_quote::TokenStream) {
        let major = self.major;
        let minor = self.minor;
        krs_quote_with!(tokens <- ({@major}, {@minor}, 0) );
    }
}

/// Compare feature versions form a functionality perspective
///
/// X > Y means X has all functionality of Y, and more
/// X = Y means X and Y have the same functionality
/// X < Y means X has less functionality than Y
impl std::cmp::PartialOrd for FeatureVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.major.partial_cmp(&other.major) {
            Some(core::cmp::Ordering::Equal) => {}
            _ => panic!(
                "It is not currently clear how functionality of different major versions compare"
            ),
        }
        self.minor.partial_cmp(&other.minor)
    }
}

/// Compare feature versions form a functionality perspective
///
/// X > Y means X has all functionality of Y, and more
/// X = Y means X and Y have the same functionality
/// X < Y means X has less functionality than Y
impl std::cmp::Ord for FeatureVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            core::cmp::Ordering::Equal => {}
            _ => panic!(
                "It is not currently clear how functionality of different major versions compare"
            ),
        }
        self.minor.cmp(&other.minor)
    }
}

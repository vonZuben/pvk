use once_cell::sync::OnceCell;
use std::collections::HashMap;

use crate::commands;

use crate::utils;

use vkxml::*;

// TODO: GlobalData data members don't need to be public
// want to encourage use of the helper methods

pub struct VkResultMember<'a> {
    pub name: &'a str,
    pub is_err: bool,
}

pub struct VKSt<'a> {
    pub name: &'a str,
    pub st_name: &'a str,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct TypeLifetime {
    pub public: bool,
    pub private: bool,
}

impl TypeLifetime {
    fn merge(&mut self, other: TypeLifetime) {
        self.public |= other.public;
        self.private |= other.private;
    }
}

type TypeLifetimes<'a> = HashMap<&'a str, TypeLifetime>;

type Dictionary<'a> = HashMap<&'a str, ()>;
type VkResultMembers<'a> = Vec<VkResultMember<'a>>;
type GenericDictionary<T> = HashMap<T, ()>;

type ExtensionMap<'a> = (String, &'a str); // extension_name (const value), and extension name for code gen

type EnumDictionary<'a> = HashMap<String, Vec<String>>;

#[derive(Default)]
pub struct GlobalData<'a> {
    // extern_sync_params holds a command/struct_name and a list of params in a string that are
    // externally synced
    pub extern_sync_params: HashMap<&'a str, String>,
    pub handles: Dictionary<'a>,
    pub command_types: HashMap<&'a str, commands::CommandCategory>,
    pub not_sync_handles: Dictionary<'a>,
    pub not_sync_and_send_handles: Dictionary<'a>,
    pub extension_tags: Dictionary<'a>,
    pub result_members: VkResultMembers<'a>,
    pub extension_maps: Vec<ExtensionMap<'a>>,
    pub all_enums: EnumDictionary<'a>,
    pub is_freeable_handle: Dictionary<'a>,
    pub type_lifetimes: TypeLifetimes<'a>,
    pub extendable: Dictionary<'a>,
    pub is_base: Dictionary<'a>,
    pub all_structure_types: Vec<VKSt<'a>>,
    pub size_fields: GenericDictionary<(&'a str, &'a str)>,
    pub optional_size: GenericDictionary<(&'a str, &'a str)>,
    pub noautovalid: GenericDictionary<(&'a str, &'a str)>,
    pub versions: Vec<&'a vkxml::Features>,
}

pub static GLOBAL_DATA: OnceCell<GlobalData<'static>> = OnceCell::new();

pub static REGISTRY: OnceCell<Registry> = OnceCell::new();
pub static REGISTRY2: OnceCell<vk_parse::Registry> = OnceCell::new();

fn expect_gd() -> &'static GlobalData<'static> {
    GLOBAL_DATA.get().expect("error: GLOBAL_DATA not set")
}

pub fn all_enums() -> &'static EnumDictionary<'static> {
    &expect_gd().all_enums
}

pub fn extension_maps() -> &'static Vec<ExtensionMap<'static>> {
    &expect_gd().extension_maps
}

#[allow(unused)]
pub fn result_members() -> &'static VkResultMembers<'static> {
    &expect_gd().result_members
}

pub fn is_freeable_handle(handle_basetype: &str) -> bool {
    expect_gd().is_freeable_handle.contains_key(handle_basetype)
}

pub fn command_type(cmd_name: &str) -> commands::CommandCategory {
    *expect_gd().command_types.get(cmd_name).expect("error: command, {}, has no command type")
}

// context: struct or command name
// field: struct or command member field
pub fn is_externsync(context: &str, field: &Field) -> bool {
    expect_gd()
        .extern_sync_params.get(context)
        .map(|sync_params| sync_params.contains(utils::field_name_expected(field)))
        .unwrap_or(false)
}

pub fn is_handle(field_type: &str) -> bool {
    expect_gd().handles.contains_key(field_type)
}

pub fn is_handle_not_sync(handle_name: &str) -> bool {
    expect_gd().not_sync_handles.contains_key(handle_name)
}

pub fn is_handle_not_sync_and_send(handle_name: &str) -> bool {
    expect_gd().not_sync_and_send_handles.contains_key(handle_name)
}

pub fn extension_tags() -> &'static Dictionary<'static> {
    &expect_gd().extension_tags
}

pub fn type_lifetime(name: &str) -> Option<TypeLifetime> {
    expect_gd().type_lifetimes.get(name).copied()
}

pub fn is_extendable(name: &str) -> bool {
    expect_gd().extendable.contains_key(name)
}

pub fn is_base(name: &str) -> bool {
    expect_gd().is_base.contains_key(name)
}

pub fn structure_types() -> &'static Vec<VKSt<'static>> {
    &expect_gd().all_structure_types
}

pub fn is_size_field(context: &str, field: &Field) -> bool {
    expect_gd().size_fields.contains_key(&(context, utils::field_name_expected(field)))
}

pub fn is_optional_size(context: &str, size: &str) -> bool {
    expect_gd().optional_size.contains_key(&(context, size))
}

pub fn is_noautovalid(context: &str, field: &Field) -> bool {
    expect_gd().noautovalid.contains_key(&(context, utils::field_name_expected(field)))
}

#[allow(unused)]
pub fn versions() -> impl Iterator<Item=&'static vkxml::Feature> {
    expect_gd().versions.iter().map(|f| f.elements.iter()).flatten()
}

// the first pass of the registry is for collecting information about the kinds of basetypes
// and other things that can be collected and used to make other decisions in a second pass
//
// e.g. we want to know if a basetype is a Handle in a struct member, but we only have the name of
// the base type when parsing. This first pass will let us collect information such as what types
// are Handles, and that can be used in a second pass
//
// we should be able to collect all information we need from only 2 passes

pub fn generate(registry: &'static vkxml::Registry, registry2: &'static vk_parse::Registry) {

    let mut global_data = GlobalData::default();

    let mut structs = HashMap::new();
    type Structs<'a> = HashMap<&'a str, &'a Struct>;
    //let mut func_ptrs = HashMap::new();
    let mut unions = HashMap::new();

    // ================ STATICS ==========================

    // ================ Special cases for SEND and SYNC ==========================
    // all handles are send and sync by defauly, but some shouldn't be
    // handles which should only be send but not sync
    // these are not sync because we will allow shared references for externsync methods, they are
    // still send
    let not_sync_handles = [
        "VkCommandPool",
        "VkDescriptorPool",
        "VkDescriptorSet",
        "VkDisplayKHR",
        "VkDisplayModeKHR",
        "VkSwapchainKHR",
        "VkSurfaceKHR",

        // ---------- From here is handles that are also not sync and are included in the below map
        "VkCommandBuffer",
    ];

    // these handles should not be send or sync
    // this makes it so these handles cannot be sent between threads
    let not_sync_and_send_handles = [
        "VkCommandBuffer",
    ];

    for handle_name in not_sync_handles.iter() {
        global_data.not_sync_handles.insert(handle_name, ());
    }

    for handle_name in not_sync_and_send_handles.iter() {
        global_data.not_sync_and_send_handles.insert(handle_name, ());
    }

    // these are exceptions, where we should still take &mut even if the type is not sync
    // (Command name, param name)
    let still_take_mut = [
        ("vkResetDescriptorPool", "descriptorPool"),
        ("vkResetCommandPool", "commandPool"),
    ];

    let should_be_extern_sync = |context, member| -> bool {
        if still_take_mut.contains( &(context, utils::field_name_expected(member)) ) {
            // some handle types which normally arn't sync will still need MutHandle in some
            // contexts
            true
        }
        else if not_sync_handles.contains(&member.basetype.as_str()) {
            false
        }
        else {
            true
        }
    };

    // ================ FIRST PASS ==========================

    for reg_elem in registry.elements.iter() {
        match reg_elem {
            RegistryElement::Definitions(definitions) => {
                for def in definitions.elements.iter() {
                    match def {
                        // NOTE: we are assuming that Handles are parsed before other definitions
                        DefinitionsElement::Handle(handle) => {
                            global_data.handles.insert(handle.name.as_str(), ());
                            global_data.type_lifetimes.insert(handle.name.as_str(), TypeLifetime { public: true, private: false });
                        }
                        DefinitionsElement::Struct(ref stct) => {
                            structs.insert(stct.name.as_str(), stct);

                            if let Some(extends) = stct.extends.as_ref() {
                                for extends in extends.split(',') {
                                    global_data.extendable.entry(extends)
                                        .or_insert(());
                                }
                            }
                            let mut optional_fields = Vec::new();
                            for field in stct.elements.iter().filter_map(variant!(StructElement::Member)) {
                                if stct.name.as_str() == "VkBaseOutStructure" || stct.name.as_str() == "VkBaseInStructure" {
                                    continue;
                                }

                                let name = stct.name.as_str();

                                if field.basetype.as_str() == "VkStructureType" {
                                    let st_name = utils::structure_type_name(field);
                                    global_data.all_structure_types.push(VKSt{name, st_name});
                                    global_data.is_base.entry(name).or_insert(());
                                }

                                if let Some(size) = field.size.as_ref() {
                                    global_data.size_fields.insert((name, size.as_str()), ());
                                    if optional_fields.contains(&size.as_str()) {
                                        global_data.optional_size.insert((name, size.as_str()), ());
                                    }
                                }

                                if field.optional.as_ref().map(|opt|opt.split(',').next() == Some("true")).unwrap_or(false) {
                                    optional_fields.push(utils::field_name_expected(field));
                                }
                            }
                        }
                        DefinitionsElement::Bitmask(bitmask) => {
                            let enm_name = utils::normalize_flag_names(&bitmask.name);
                            assert!(global_data.all_enums.insert(enm_name, Vec::new()).is_none(),
                                    "unextepxted value already in all_enums");
                        }
                        DefinitionsElement::FuncPtr(_fptr) => {
                        }
                        DefinitionsElement::Union(uni) => {
                            unions.insert(uni.name.as_str(), ());
                        }
                        _ => {}
                    }
                }
            }
            RegistryElement::Constants(_cnts) => {
            }
            RegistryElement::Enums(enums) => {
                for element in enums.elements.iter() {
                    match element {
                        EnumsElement::Enumeration(enm) => {
                            if enm.name.as_str() == "VkResult" {
                                for member in enm.elements.iter() {
                                    match member {
                                        EnumerationElement::Enum(enum_constant) => {
                                            let name = &enum_constant.name.as_str()[3..]; // cut off Vk_
                                            let is_err = name.starts_with("ERROR_");

                                            let result_member = VkResultMember {
                                                name,
                                                is_err,
                                            };
                                            global_data.result_members.push(result_member);
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            // add new enumeration if not already there
                            let enm_name = utils::normalize_flag_names(&enm.name);
                            global_data.all_enums.entry(enm_name.clone())
                                .or_default();

                            // add core variants to enumeration
                            for elem in enm.elements.iter()
                                .filter_map(variant!(EnumerationElement::Enum))
                                {
                                    global_data.all_enums.get_mut(&enm_name)
                                        .expect("error: enum not in all_enums")
                                        .push(elem.name.clone());
                                }
                        }
                        EnumsElement::Notation(_) => {}
                    }
                }
            }
            RegistryElement::Commands(cmds) => {
                for cmd in cmds.elements.iter() {
                    global_data.command_types.insert(cmd.name.as_str(), commands::command_category(&cmd));

                    // assume last param of an allocate command is the handles which is allocated
                    // all allocated handles should have a command for freeing
                    // thus assume said handle is freeable
                    if cmd.name.starts_with("vkAllocate") {
                        let freeable_param = cmd.param.last().expect("error: allocate command with no last param");
                        global_data.is_freeable_handle.insert(freeable_param.basetype.as_str(), ());
                    }
                }
            }
            RegistryElement::Features(features) => {
                global_data.versions.push(features);
            }
            RegistryElement::Extensions(extensions) => {
                for extension in extensions.elements.iter() {
                    // some extensions are just placeholders and do not have a type
                    // thus, we should not generate any code for them since they have no function
                    if extension.ty.is_none() {
                        continue;
                    }

                    // TODO: check vkxml::DefinitionReference variant

                    // Enum defeined by extensions
                    let enum_extensions = extension.elements.iter()
                        .filter_map(variant!(ExtensionElement::Require))
                        .map(|extension_spec| extension_spec.elements.iter()
                             .filter_map(variant!(ExtensionSpecificationElement::Enum))
                            )
                        .flatten();

                    for enum_extension in enum_extensions {
                        let extends = enum_extension.extends.as_str();
                        if extends == "VkResult" {
                            let name = &enum_extension.name.as_str()[3..]; // cut off Vk_
                            let is_err = name.starts_with("ERROR_");

                            let result_member = VkResultMember {
                                name,
                                is_err,
                            };
                            global_data.result_members.push(result_member);
                        }

                        // add extension defined vairaints to enums
                        let enm_name = utils::normalize_flag_names(&enum_extension.extends);
                        global_data.all_enums.get_mut(&enm_name)
                            .expect("error: extension enum not in all_enums")
                            .push(enum_extension.name.clone());
                    }

                    // Const defeined by extensions
                    let constant_extensions = extension.elements.iter()
                        .filter_map(variant!(ExtensionElement::Require))
                        .map(|extension_spec| extension_spec.elements.iter()
                            .filter_map(variant!(ExtensionSpecificationElement::Constant))
                            )
                        .flatten();

                    for const_extension in constant_extensions {
                        // want to add extension names to map so that we can generate a giant map from ExtensionProperties to &dyn VkExtensionLoader
                        if utils::is_extension_name(&const_extension.name) {
                            let name_def: &str = const_extension.text.as_ref().expect("error: extension name without text value");
                            let name = extension.name.as_str();

                            let c_name = format!("b\"{}\"", name_def);

                            // note, c_name and name are probably always the same, but I am doing this anyway just to be sure
                            // in anyevent, I need a vec with all of the possible extensions so I can map ExtensionProperties (which have a c string name)
                            // to a &dyn VkExtensionLoader which implements the loading methods for that extension
                            global_data.extension_maps.push((c_name, name));
                        }
                    }
                }
            }
            RegistryElement::Tags(tags) => {
                for tag in tags.elements.iter() {
                    global_data.extension_tags.insert(tag.name.as_str(), ());
                }
            }
            _ => {}
        }
    }

    // ================ SECOND PASS ==========================

    for reg_elem in registry.elements.iter() {
        match reg_elem {
            RegistryElement::Definitions(definitions) => {
                for def in definitions.elements.iter() {
                    match def {
                        DefinitionsElement::Struct(stct) => {
                            // check if struct identifies param as extern sync
                            for field in stct.elements.iter().filter_map(variant!(StructElement::Member)) {
                                if let Some(synced_thing) = field.sync.as_ref() {
                                    assert_eq!(synced_thing, "true");
                                    let synced_thing = utils::field_name_expected(&field);
                                    if should_be_extern_sync(stct.name.as_str(), field) {
                                        global_data.extern_sync_params
                                            .entry(stct.name.as_str())
                                            .and_modify(|list| list.push_str(format!(",{}", synced_thing).as_str()))
                                            .or_insert(synced_thing.to_string());
                                    }
                                }
                            }
                        }
                        DefinitionsElement::FuncPtr(_fptr) => {
                        }
                        DefinitionsElement::Union(_uni) => {
                        }
                        _ => {}
                    }
                }
            }
            RegistryElement::Constants(_cnts) => {
            }
            RegistryElement::Enums(_enums) => {
            }
            RegistryElement::Commands(cmds) => {
                for cmd in cmds.elements.iter() {
                    for param in cmd.param.iter() {
                        if param.sync.is_some() {
                            if let Some(stct) = structs.get(param.basetype.as_str()) {
                                // now, if a command takes a struct, and the struct has a parameter
                                // that should be externally synced, then we keep track of it for use
                                // later

                                let synced_thing = param.sync.as_ref().expect("already confimed sync is_some");

                                for field in stct.elements.iter().filter_map(variant!(StructElement::Member))
                                    .filter(|field| synced_thing.contains(utils::field_name_expected(field)) )
                                    {
                                        // TODO consider if context for should_be_extern_sync
                                        // should be the struct name or the command name
                                        if should_be_extern_sync(stct.name.as_str(), field) {
                                            global_data.extern_sync_params
                                                .entry(param.basetype.as_str())
                                                .and_modify(|list| list.push_str(format!(",{}", synced_thing).as_str()))
                                                .or_insert(synced_thing.to_string());
                                        }
                                    }
                            }
                            else {
                                // otherwise, we check for normal extern sync param
                                assert_eq!(param.sync.as_ref().expect("already confimed is_some"), "true");

                                let synced_thing = utils::field_name_expected(&param);

                                let cmd_name = cmd.name.as_str();

                                if should_be_extern_sync(cmd_name, &param) {
                                    global_data.extern_sync_params
                                        .entry(cmd_name)
                                        .and_modify(|list| list.push_str(format!(",{}", synced_thing).as_str()))
                                        .or_insert(synced_thing.to_string());
                                }
                            }
                        }
                    }
                }
            }
            RegistryElement::Features(_features) => {
            }
            RegistryElement::Extensions(_extensions) => {
            }
            _ => {}
        }
    }

    // ================ vk_parse REG PASS ==========================

    for item in registry2.0.iter() {
        match item {
            vk_parse::RegistryChild::Feature(feature) => get_feature_enums_from_vk_parse_reg(feature, &mut global_data),
            vk_parse::RegistryChild::Types(ty) => get_no_auto_valid(ty, &mut global_data),
            _ => {},
        }
    };

    fn get_struct_lifetime<'a>(name: &'a str, structs: &Structs<'a>, handles: &Dictionary, s_lifetime: &mut TypeLifetimes<'a>) -> TypeLifetime {
        let mut lifetime = TypeLifetime::default();
        let stct = structs.get(name).unwrap();

        if s_lifetime.contains_key(name) {
            return *s_lifetime.get(name).unwrap();
        }

        for field in stct.elements.iter().filter_map(variant!(StructElement::Member)) {
            let field_type = field.basetype.as_str();

            if field.name.as_ref().map(String::as_str) == Some("pNext") {
                lifetime.public = true;
                lifetime.private = true;
                break;
            }

            if field_type == name {
                continue;
            }

            if structs.contains_key(field_type) {
                if s_lifetime.contains_key(field_type) {
                    let field_lifetime = *s_lifetime.get(field_type).unwrap();
                    lifetime.merge(field_lifetime);
                }
                else {
                    let field_lifetime = get_struct_lifetime(field_type, structs, handles, s_lifetime);
                    s_lifetime.insert(field_type, field_lifetime);
                    lifetime.merge(field_lifetime);
                }
             }

            if handles.contains_key(field_type) {
                lifetime.public = true;
            }

            if field.reference.is_some() {
                lifetime.private = true;
            }

            // break early since we alreay know this struct has both types of lifetimes
            if lifetime.public && lifetime.private {
                break;
            }
        }
        lifetime
    }

    for stct in structs.iter() {
        let name = stct.0;
        if !global_data.type_lifetimes.contains_key(name) {
            let lifetime = get_struct_lifetime(name, &structs, &global_data.handles, &mut global_data.type_lifetimes);
            global_data.type_lifetimes.insert(name, lifetime);
        }
    }

    assert!(GLOBAL_DATA.set(global_data).is_ok());
}

pub fn get_feature_enums_from_vk_parse_reg(feature: &vk_parse::Feature, global_data: &mut GlobalData) {
    for interface_item in feature.children.iter()
        .filter_map(
            |feature| {
                match feature {
                    vk_parse::ExtensionChild::Require{items, ..} => {
                        Some( items.iter() )
                    }
                    _ => None,
                }
            }
        )
        .flatten() {
                match interface_item {
                    vk_parse::InterfaceItem::Enum(enm) => {
                        match &enm.spec {
                            vk_parse::EnumSpec::Alias{alias: _, extends: _} => {
                                unimplemented!("not yet dealing with Aliases for enums defined by features");
                            }
                            vk_parse::EnumSpec::Offset{offset: _, extends, extnumber: _, dir: _} => {
                                let enm_name = utils::normalize_flag_names(extends);
                                global_data.all_enums.get_mut(&enm_name)
                                    .expect("error: extension enum not in all_enums")
                                    .push(enm.name.clone());
                            }
                            vk_parse::EnumSpec::Bitpos{bitpos: _, extends} => {
                                match extends {
                                    Some(extends) => {
                                        let enm_name = utils::normalize_flag_names(extends);
                                        global_data.all_enums.get_mut(&enm_name)
                                            .expect("error: extension enum not in all_enums")
                                            .push(enm.name.clone());
                                    }
                                    None => unimplemented!("not yet handle Feature defined enum with Bitset that dosn't extend another enum"),
                                }
                            }
                            vk_parse::EnumSpec::Value{value: _, extends: _} => {
                                unimplemented!("not yet handle Feature defined enum with Value")
                            }
                            vk_parse::EnumSpec::None => {}
                        }
                    }
                    _ => {},
                }
            }
}

fn get_no_auto_valid(types_child: &'static vk_parse::CommentedChildren<vk_parse::TypesChild>, global_data: &mut GlobalData) {
    'child: for ty in types_child.children.iter() {
        match ty {
            vk_parse::TypesChild::Type(ty) => {
                if ty.category.as_ref().map(String::as_str) != Some("struct") {
                    continue 'child;
                }
                match &ty.spec {
                    vk_parse::TypeSpec::Members(members) => {
                        for member in members.iter() {
                            match member {
                                vk_parse::TypeMember::Definition(member) => {
                                    if member.len.is_some() {
                                        if let Some(_av) = member.noautovalidity.as_ref(){
                                            let get_field_name = || {
                                                for m in member.markup.iter() {
                                                    match m {
                                                        vk_parse::TypeMemberMarkup::Name(name) => return name.as_str(),
                                                        _ => {}
                                                    }
                                                }
                                                panic!("no name for filed of struct")
                                            };
                                            let field_name = get_field_name();
                                            global_data.noautovalid.insert((ty.name.as_ref().unwrap().as_str(), field_name), ());
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
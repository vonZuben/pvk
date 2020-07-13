use once_cell::sync::OnceCell;
use std::collections::HashMap;

use quote::quote;

use crate::commands;
use crate::utils;

use vkxml::*;
use proc_macro2::{TokenStream};

// TODO: GlobalData data members don't need to be public
// want to encourage use of the helper methods

#[derive(Default)]
pub struct GlobalData<'a> {
    // extern_sync_params holds a command/struct_name and a list of params in a string that are
    // externally synced
    pub extern_sync_params: HashMap<&'a str, String>,
    pub needs_lifetime: HashMap<&'a str, ()>,
    pub handles: HashMap<&'a str, ()>,
    pub command_types: HashMap<&'a str, commands::CommandCategory>,
    pub not_sync_handles: HashMap<&'a str, ()>,
    pub not_sync_and_send_handles: HashMap<&'a str, ()>,
}

pub static GLOBAL_DATA: OnceCell<GlobalData<'static>> = OnceCell::new();

pub static REGISTRY: OnceCell<Registry> = OnceCell::new();

fn expect_gd() -> &'static GlobalData<'static> {
    GLOBAL_DATA.get().expect("error: GLOBAL_DATA not set")
}


type Identifier<'a> = &'a str;

pub fn lifetime(named_type: &str) -> Option<TokenStream> {
    if expect_gd().needs_lifetime.get(named_type).is_some() {
        Some( quote!(<'handle>) )
    }
    else {
        None
    }
}

pub fn uses_lifetime(named_type: &str) -> bool {
    if expect_gd().needs_lifetime.get(named_type).is_some() {
            true
    }
    else {
        false
    }
}

pub fn command_type(cmd_name: &str) -> commands::CommandCategory {
    *expect_gd().command_types.get(cmd_name).expect("error: command, {}, has no command type")
}

// context: struct or command name
// field: struct or command member field
pub fn is_externsync(context: &str, field: &Field) -> bool {
    if is_handle_not_sync(field.basetype.as_str()) {
        false
    }
    else {
        expect_gd()
            .extern_sync_params.get(context)
            .map(|sync_params| sync_params.contains(utils::field_name_expected(field)))
            .unwrap_or(false)
    }
}

pub fn is_handle(field_type: &str) -> bool {
    expect_gd().handles.get(field_type).is_some()
}

pub fn is_handle_not_sync(handle_name: &str) -> bool {
    expect_gd().not_sync_handles.get(handle_name).is_some()
}

pub fn is_handle_not_sync_and_send(handle_name: &str) -> bool {
    expect_gd().not_sync_and_send_handles.get(handle_name).is_some()
}

// the first pass of the registry is for collecting information about the kinds of basetypes
// and other things that can be collected and used to make other decisions in a second pass
//
// e.g. we want to know if a basetype is a Handle in a struct member, but we only have the name of
// the base type when parsing. This first pass will let us collect information such as what types
// are Handles, and that can be used in a second pass
//
// we should be able to collect all information we need from only 2 passes

macro_rules! filter_varient {
    ( $varient:path ) => {
        |elem| {
            match elem {
                $varient(varient) => Some(varient),
                _ => None,
            }
        }
    }
}

pub fn generate(registry: &'static vkxml::Registry) {

    let mut global_data = GlobalData::default();

    let mut structs = HashMap::new();
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

    // ================ FIRST PASS ==========================

    for reg_elem in registry.elements.iter() {
        match reg_elem {
            RegistryElement::Definitions(definitions) => {
                for def in definitions.elements.iter() {
                    match def {
                        // NOTE: we are assuming that Handles are parsed before other definitions
                        DefinitionsElement::Handle(handle) => {
                            global_data.handles.insert(handle.name.as_str(), ());
                            global_data.needs_lifetime.insert(handle.name.as_str(), ());
                        }
                        DefinitionsElement::Struct(stct) => {
                            structs.insert(stct.name.as_str(), ());
                            global_data.needs_lifetime.insert(stct.name.as_str(), ());
                            //for field in stct.elements.iter().filter_map(filter_varient!(StructElement::Member)) {
                            //    if global_data.needs_lifetime.get(field.basetype.as_str()).is_some() {
                            //        global_data.needs_lifetime.insert(stct.name.as_str(), ());
                            //        break;
                            //    }
                            //}
                        }
                        DefinitionsElement::FuncPtr(fptr) => {
                            //func_ptrs.insert(fptr.name.as_str(), ());
                            //for field in fptr.param.iter() {
                            //    if global_data.needs_lifetime.get(field.basetype.as_str()).is_some() {
                            //        global_data.needs_lifetime.insert(fptr.name.as_str(), ());
                            //        break;
                            //    }
                            //}
                        }
                        DefinitionsElement::Union(uni) => {
                            unions.insert(uni.name.as_str(), ());
                            for field in uni.elements.iter() {
                                if global_data.needs_lifetime.get(field.basetype.as_str()).is_some() {
                                    global_data.needs_lifetime.insert(uni.name.as_str(), ());
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            RegistryElement::Constants(cnts) => {
            }
            RegistryElement::Enums(enums) => {
            }
            RegistryElement::Commands(cmds) => {
                for cmd in cmds.elements.iter() {
                    global_data.command_types.insert(cmd.name.as_str(), commands::command_category(&cmd));
                }
            }
            RegistryElement::Features(features) => {
            }
            RegistryElement::Extensions(extensions) => {
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
                            if global_data.needs_lifetime.get(stct.name.as_str()).is_none() {
                                for field in stct.elements.iter().filter_map(filter_varient!(StructElement::Member)) {
                                    if global_data.needs_lifetime.get(field.basetype.as_str()).is_some() {
                                        global_data.needs_lifetime.insert(stct.name.as_str(), ());
                                        break;
                                    }
                                }
                            }

                            // check if struct identifies param as extern sync
                            for field in stct.elements.iter().filter_map(filter_varient!(StructElement::Member)) {
                                if let Some(synced_thing) = field.sync.as_ref() {
                                    assert_eq!(synced_thing, "true");
                                    let synced_thing = utils::field_name_expected(&field);
                                    global_data.extern_sync_params
                                        .entry(stct.name.as_str())
                                        .and_modify(|list| list.push_str(format!(",{}", synced_thing).as_str()))
                                        .or_insert(synced_thing.to_string());
                                }
                            }
                        }
                        DefinitionsElement::FuncPtr(fptr) => {
                            //if global_data.needs_lifetime.get(fptr.name.as_str()).is_none() {
                            //    for field in fptr.param.iter() {
                            //        if global_data.needs_lifetime.get(field.basetype.as_str()).is_some() {
                            //            global_data.needs_lifetime.insert(fptr.name.as_str(), ());
                            //            break;
                            //        }
                            //    }
                            //}
                        }
                        DefinitionsElement::Union(uni) => {
                            if global_data.needs_lifetime.get(uni.name.as_str()).is_none() {
                                for field in uni.elements.iter() {
                                    if global_data.needs_lifetime.get(field.basetype.as_str()).is_some() {
                                        global_data.needs_lifetime.insert(uni.name.as_str(), ());
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            RegistryElement::Constants(cnts) => {
            }
            RegistryElement::Enums(enums) => {
            }
            RegistryElement::Commands(cmds) => {
                for cmd in cmds.elements.iter() {
                    for param in cmd.param.iter() {
                        if structs.get(param.basetype.as_str()).is_some() && param.sync.is_some() {
                            // now, if a command takes a struct, and the struct has a parameter
                            // that should be externally synced, then we keep track of it for use
                            // later

                            let synced_thing = param.sync.as_ref().expect("already confimed sync is_some");

                            global_data.extern_sync_params
                                .entry(param.basetype.as_str())
                                .and_modify(|list| list.push_str(format!(",{}", synced_thing).as_str()))
                                .or_insert(synced_thing.to_string());
                        }
                        else if param.sync.as_ref().map(|sync_kind|sync_kind=="true").unwrap_or(false) {
                            // otherwise, we check for normal extern sync param

                            let synced_thing = utils::field_name_expected(&param);

                            global_data.extern_sync_params
                                .entry(cmd.name.as_str())
                                .and_modify(|list| list.push_str(format!(",{}", synced_thing).as_str()))
                                .or_insert(synced_thing.to_string());
                        }
                    }
                }
            }
            RegistryElement::Features(features) => {
            }
            RegistryElement::Extensions(extensions) => {
            }
            _ => {}
        }
    }

    GLOBAL_DATA.set(global_data);
}

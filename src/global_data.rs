use once_cell::sync::OnceCell;
use std::collections::HashMap;

use quote::quote;

use vkxml::*;
use proc_macro2::{TokenStream};

pub struct GlobalData<'a> {
    pub struct_with_sync_member: HashMap<&'a str, &'a str>,
    pub needs_lifetime: HashMap<&'a str, ()>,
    pub handles: HashMap<&'a str, ()>,
}

pub static GLOBAL_DATA: OnceCell<GlobalData<'static>> = OnceCell::new();

pub static REGISTRY: OnceCell<Registry> = OnceCell::new();


type Identifier<'a> = &'a str;

pub fn lifetime(named_type: &str) -> Option<TokenStream> {
    if GLOBAL_DATA.get().expect("error: global_data not set")
        .needs_lifetime.get(named_type).is_some() {
        Some( quote!(<'handle>) )
    }
    else {
        None
    }
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

    let mut global_data = GlobalData {
        struct_with_sync_member: HashMap::new(),
        needs_lifetime: HashMap::new(),
        handles: HashMap::new(),
    };

    let mut structs = HashMap::new();
    //let mut func_ptrs = HashMap::new();
    let mut unions = HashMap::new();

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
                            global_data.struct_with_sync_member
                                .insert(param.basetype.as_str(), param.sync.as_ref().unwrap());
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

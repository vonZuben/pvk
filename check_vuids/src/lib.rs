#![warn(missing_docs)]

/*!
This module includes macros that are used by the check_vuids tool

the macros can be inserted into rust source code files to provide information about what VUIDs are being checked
the check_vuids tool will parse the rust code files and check to see that the VUIDs are up to date

the macros generate no actual code and only provide information to the reader and check_vuids tool
 */

/// indicates start of a list of VUIDs to check
///
/// must be placed within a block
/// all VUIDs to check must follow the macro call within the same block
#[macro_export]
macro_rules! check_vuids {
    ($name:ident) => {};
}

/// indicate the version of the VUID to check
/// the check_vuids tool will check if this is up to date with the latest version that check_vuids was compiled with
#[macro_export]
macro_rules! version {
    ($ver:literal) => {};
}

/// indicate the current description of the VUID to check
/// the check_vuids tool will check if this is up to date with the latest version that check_vuids was compiled with
#[macro_export]
macro_rules! cur_description {
    ($desc:literal) => {};
}

/// provide the old description of the VUID
/// when the check_vuids toll updates the description of a VUID, it will save the old description with this
/// this is simply informative to the user to allow them to easily compare and see what is new
#[macro_export]
macro_rules! old_description {
    ($desc:literal) => {};
}

/* EXAMPLE
const fn tst() {
    #![allow(unused_labels)]
    check_vuids!(CreateInstance);

    'VUID_INFO_003245: {
        version!("1.3.24");
        cur_description!("Info must be valid");

        compile_error!("new VUID")
    }

    'VUID_INFO_001111: {
        version!("1.3.24");
        cur_description!("Info must be valid and ...");
        old_description!("Info must be valid");

        compile_error!("updated VUID")
    }
}
 */

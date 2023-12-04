
#[macro_export]
macro_rules! check_vuids {
    ($name:ident) => {};
}

#[macro_export]
macro_rules! version {
    ($ver:literal) => {};
}

#[macro_export]
macro_rules! description {
    ($desc:literal) => {};
}

/* EXAMPLE
#[allow(unused_labels)]
const fn tst() {
    check_vuids!(CreateInstance);

    'VUID_INFO_003245: {
        version!("1.3.24");
        description!("Info must be valid");

        compile_error!("check new VUID_INFO_003245")
    }
}
 */


use quote::quote;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;

type VkCommandParts<'a> = (Option<&'a str>, Option<&'a str>, Option<&'a str>);

pub fn handle_commands(commands: &Commands) -> TokenStream {

    let q = commands.elements.iter().map(|cmd| {

        let name = cmd.name.as_code();
        let pfn_name = format!("PFN_{}", cmd.name).as_code();
        let raw_name = &cmd.name;

        let return_type = make_field_type(&cmd.return_type);
        let params1 = cmd.param.iter().map(handle_field);
        let params2 = params1.clone();

        //println!("{} ({})", cmd.name, &cmd.param[0].basetype);

        quote!{
            #[allow(non_camel_case_types)]
            pub type #pfn_name = extern "system" fn(
                #( #params1 ),*
            ) -> #return_type;

            struct #name (#pfn_name);
            impl #name {
                pub fn load<F>(mut _f: F) -> Self
                    where F: FnMut(&::std::ffi::CStr) -> *const c_void
                    {
                        extern "system" fn load_error ( #( #params2 ),* ) -> #return_type {
                            panic!(concat!("Unable to load ", #raw_name))
                        }
                        let cname = ::std::ffi::CString::new(#raw_name).unwrap();
                        let val = _f(&cname);
                        if val.is_null(){
                            Self(load_error)
                        }
                        else{
                            Self(unsafe { ::std::mem::transmute(val) } )
                        }
                    }
            }
        }

    });

    quote!( #( #q )* )

}

//fn get_command_parts(cmd: &Command) -> VkCommandParts {
//
//    let name = &cmd.name;
//
//    let prefix = match name {
//       x if x.starts_with("VkQueue") =>
//
//    (Some("efe"), Some("efef"), Some("fef"))
//
//}

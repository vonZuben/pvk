
use quote::quote;
use quote::ToTokens;

use vkxml::*;

use proc_macro2::{TokenStream};

use crate::utils::*;
use crate::utils;
use crate::commands::*;
use crate::global_data;

pub fn handle_extensions<'a>(extensions: &'a Extensions, parse_state: &mut crate::ParseState<'a>) -> TokenStream {

    let q = extensions.elements.iter().map(|extension| {

        if extension.ty.is_none() { return quote!(); }

        let extension_name = extension.name.as_code();

        // NOTE the current code does not handle 'Removed' functionality
        // i.e. at the time of writing this, the vulkan spec does not remove
        // any functions in any features or extensions. Thus, we ignore the
        // Remove case for now

        let enum_constants_name_cache = &mut parse_state.enum_constants_name_cache;
        let command_alias_cache = &parse_state.command_alias_cache;

        let enum_extensions = extension.elements.iter()
            .filter_map(variant!(ExtensionElement::Require))
            .map(|extension_spec| extension_spec.elements.iter()
                 .filter_map(variant!(ExtensionSpecificationElement::Enum))
                 )
            .flatten()
            .map(|enum_extension| {

                // add enum constant names to cache and if the name already exists, then do not
                // generate duplicate
                match enum_constants_name_cache.insert(enum_extension.name.as_str(), ()) {
                    Some(_) => return quote!() ,
                    None => {}
                }

                let name = enum_extension.extends.as_code();
                let const_name = crate::enumerations
                    ::make_variant_name(&enum_extension.extends, enum_extension.name.as_str()).as_code();

                let val = enum_extension.val(extension.number);

                quote!{
                    impl #name {
                        pub const #const_name: Self = #name(#val);
                    }
                }

            });

        let constant_extensions = extension.elements.iter()
            .filter_map(variant!(ExtensionElement::Require))
            .map(|extension_spec| extension_spec.elements.iter()
                 .filter_map(variant!(ExtensionSpecificationElement::Constant))
                 )
            .flatten()
            .map(|const_extension| {
                // every extension should define an extension name
                // we will add a method for easily obtianing a C string
                // of the extension name
                let extension_name_impl = if utils::is_extension_name(&const_extension.name) {
                    let name = const_extension.text.as_ref().expect("error: extension name without text value");
                    let c_name = name.to_string() + "\0";

                    Some(
                        quote!{
                            pub const #extension_name: extensions::#extension_name = unsafe {
                                extensions::#extension_name::new(transmute(#c_name.as_ptr()))
                            };
                        }
                    )
                }
                else {
                    None
                };
                let name = const_extension.name();
                let ty = const_extension.ty();
                let val = const_extension.val();
                quote!{
                    pub const #name: #ty = #val;
                    #extension_name_impl
                }
            });

        let commands_to_load: Vec<_> = extension.elements.iter()
            .filter_map(variant!(ExtensionElement::Require))
            .map(|extension_spec| extension_spec.elements.iter()
                 .filter_map(variant!(ExtensionSpecificationElement::CommandReference))
                 )
            .flatten()
            .map(|command_ref| {
                // check the command_alias_cache to see if the extension identifies an alias
                let command_ref_name = command_ref.name.as_str();
                let name = command_alias_cache.get(command_ref_name)
                    .map_or(command_ref_name, |alias| *alias);
                (name, command_ref_name)
            })
            .collect();

        let instance_commands: Vec<_> = commands_to_load.iter()
            .filter_map( |(name, command_ref_name)| {
                let name_code = name.as_code();
                match global_data::command_type(name) {
                    CommandCategory::Instance => {
                        match global_data::extension_feature_level(&extension.name, name) {
                            Some(feature) => {
                                let requiered_feature = feature.as_code();
                                Some(
                                    quote! {
                                        if api.version() >= #requiered_feature.version() {
                                            commands.#name_code.load(loader);
                                        }
                                    }
                                )
                            }
                            None => {
                                Some( quote!( commands.#name_code.load(loader); ) )
                            }
                        }
                    }
                    CommandCategory::Device => {
                        None
                    }
                    CommandCategory::Static => panic!("error: extension command is for static command: {}", command_ref_name),
                    CommandCategory::Entry => panic!("error: extension command is for Entry command: {}", command_ref_name),
                    CommandCategory::DoNotGenerate => panic!("error: extension command is for DoNotGenerate command: {}", command_ref_name),
                }
            }).collect();

        let device_commands: Vec<_> = commands_to_load.iter()
            .filter_map( |(name, command_ref_name)| {
                let name_code = name.as_code();
                match global_data::command_type(name) {
                    CommandCategory::Instance => {
                        None
                    }
                    CommandCategory::Device => {
                        match global_data::extension_feature_level(&extension.name, name) {
                            Some(feature) => {
                                let requiered_feature = feature.as_code();
                                Some(
                                    quote! {
                                        if api.version() >= #requiered_feature.version() {
                                            commands.#name_code.load(loader);
                                        }
                                    }
                                )
                            }
                            None => {
                                Some( quote!( commands.#name_code.load(loader); ) )
                            }
                        }
                    }
                    CommandCategory::Static => panic!("error: extension command is for static command: {}", command_ref_name),
                    CommandCategory::Entry => panic!("error: extension command is for Entry command: {}", command_ref_name),
                    CommandCategory::DoNotGenerate => panic!("error: extension command is for DoNotGenerate command: {}", command_ref_name),
                }
            }).collect();

        let instance_loader_fn = if instance_commands.len() == 0 {
            None
        }
        else {
            Some ( quote!{
                fn load_instance_commands(&self, instance: Instance, commands: &mut InstanceCommands, api: &dyn Feature) {
                    let loader = |raw_cmd_name: &CStr| unsafe { GetInstanceProcAddr(instance, raw_cmd_name.to_c()) };
                    #( #instance_commands )*
                }
            })
        };

        let device_loader_fn = if device_commands.len() == 0 {
            None
        }
        else {
            Some( quote!{
                fn load_device_commands(&self, device: Device, commands: &mut DeviceCommands, api: &dyn Feature) {
                    let loader = |raw_cmd_name: &CStr| unsafe { GetDeviceProcAddr(device, raw_cmd_name.to_c()) };
                    #( #device_commands )*
                }
            })
        };

        let extension_kind = match extension.ty {
            Some(ExtensionType::Instance) => {
                // Instance extensions can only load instance commands
                quote!(InstanceEx)
            }
            Some(ExtensionType::Device) => {
                // some device extensions can also load instance level commands
                if instance_loader_fn.is_some() {
                    quote!(MultiEx)
                }
                else {
                    quote!(DeviceEx)
                }
            }
            None => unreachable!("extension.ty must be Some(..) here"),
        };

        let requiered_extensions = extension.requires.iter().map(|reqs|{
            reqs.split(",")
        }).flatten();

        let make_verify_params = |requiered_extensions: Vec<&str>| -> TokenStream {
            let indecies = (0..requiered_extensions.len()).into_iter()
            .map(|i| format!("Index{}", i + 1));
            let contains = indecies.clone().zip(requiered_extensions.iter())
            .map(|(i_name, req)| format!("Contains<extensions::{}, {}>", req.as_code(), i_name).as_code());

            let generics: Vec<_> = if matches!(extension.ty, Some(ExtensionType::Instance)) {
                indecies.map(|i_name|i_name.as_code()).chain(Some(quote!(Purity))).collect()
            }
            else {
                indecies.map(|i_name|i_name.as_code()).collect()
            };

            let requierments: Vec<_> = if matches!(extension.ty, Some(ExtensionType::Instance)) {
                contains.chain(Some(quote!(OnlyInstance<Purity>))).collect()
            }
            else {
                contains.collect()
            };
            quote!( (#extension_name => #(#generics),* ; #( + #requierments)* ) )
        };

        let verify_instance_params = make_verify_params(requiered_extensions.clone()
            .filter(|name| matches!(global_data::extention_type(name), vkxml::ExtensionType::Instance)).collect());

        let verify_device_params = make_verify_params(requiered_extensions.clone()
            .filter(|name| matches!(global_data::extention_type(name), vkxml::ExtensionType::Device)).collect());

        quote!{
            #( #enum_extensions )*
            #( #constant_extensions )*
            unsafe impl #extension_kind for extensions::#extension_name {}
            unsafe impl ExPtr for extensions::#extension_name {}
            impl_verify_instance! #verify_instance_params;
            impl_verify_device! #verify_device_params;
            impl VkExtension for extensions::#extension_name {
                #instance_loader_fn
                #device_loader_fn
            }
        }

    });

    quote!( #( #q )* )

}

// similar to ConstExt in constant.rs for Constant from vkxml
// except it is modified for ExtensionConstant from vkxml
trait ConstExtExt {
    fn ty(&self) -> TokenStream;
    fn val(&self) -> TokenStream;
    fn name(&self) -> TokenStream;
}

impl ConstExtExt for vkxml::ExtensionConstant {

    fn ty(&self) -> TokenStream {
        one_option!{

            &self.text , |_| quote!(&'static str) ;

            &self.enumref , |_| quote!(usize) ; // TODO: is this correct?

            &self.number , |_| quote!(usize) ;

            &self.hex , |_| panic!("error: trying to get hex type not implemented for ConstExtExt -> {}", self.name);

            &self.bitpos , |_| panic!("error: trying to get bitpos type not implemented for ConstExtExt -> {}", self.name);

            &self.c_expression , |_expr: &str| panic!("error: trying to get c_expression type not implemented for ConstExtExt -> {}", self.name);

        }

    }
    fn val(&self) -> TokenStream {
        one_option!{

            &self.text , |sval| quote!(#sval) ;

            &self.enumref , |r: &str| r.as_code() ;

            &self.number , |num: &i32| { num.to_string().as_code() } ;

            &self.hex , |_| panic!("error: trying to get hex type not implemented for ConstExtExt -> {}", self.name);

            &self.bitpos , |_| panic!("error: trying to get bitpos type not implemented for ConstExtExt -> {}", self.name);

            &self.c_expression , |_expr: &str| panic!("error: trying to get c_expression type not implemented for ConstExtExt -> {}", self.name);

        }
    }
    fn name(&self) -> TokenStream {
        self.name.as_code()
    }
}

pub struct EnumVal {
    bitpos: bool,
    val: i32,
}

impl std::ops::Deref for EnumVal {
    type Target = i32;
    fn deref(&self) -> &i32 {
        &self.val
    }
}

impl ToTokens for EnumVal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.bitpos {
            format!("0x{:0>8X}", self.val).as_code().to_tokens(tokens);
        }
        else {
            format!("{}", self.val).as_code().to_tokens(tokens);
        }
    }
}

pub trait EnumExtExt {
    fn val(&self, extension_number: i32) -> EnumVal; // this is s String representation of a value of any type for converting into code
}

impl EnumExtExt for ExtensionEnum {
    fn val(&self, extension_number: i32) -> EnumVal {
        one_option!{

            &self.offset , |offset: &usize|
            {
                // see vulkan spec style guide regarding this equation
                let val = 1000000000 + (extension_number - 1) * 1000 + *offset as i32;
                if self.negate {
                    EnumVal{ val: -val, bitpos: false }
                }
                else {
                    EnumVal{ val: val, bitpos: false }
                }
            };

            &self.number , |num: &i32|
                if self.negate {
                    EnumVal{ val: -*num, bitpos: false }
                }
                else {
                    EnumVal{ val: *num, bitpos: false }
                } ;

            &self.hex , |_hex| panic!("not expecting hex in enum extension definition: {}", self.name);

            // shouldn't have negative bit positions
            &self.bitpos , |bitpos: &u32| EnumVal{ val: 1i32 << bitpos, bitpos: true } ;

        }
    }
}

pub fn generate_feature_enums_from_vk_parse_reg<'a>(feature: &'a vk_parse::Feature) -> TokenStream {
    let q = feature.children.iter()
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
        .flatten()
        .filter_map(
            |interface_item| {
                match interface_item {
                    vk_parse::InterfaceItem::Enum(enm) => {
                        match &enm.spec {
                            vk_parse::EnumSpec::Alias{alias: _, extends: _} => {
                                unimplemented!("not yet dealing with Aliases for enums defined by features");
                            }
                            vk_parse::EnumSpec::Offset{offset, extends, extnumber, dir} => {
                                let extension_number = extnumber.expect("Feature defined enum with Offset value and no extnumber");
                                let mut val = 1000000000 + (extension_number - 1) * 1000 + *offset;
                                if !dir {
                                    val = -val;
                                }
                                let val = format!("{}", val).as_code();
                                let const_name = crate::enumerations
                                    ::make_variant_name(extends, enm.name.as_str()).as_code();
                                let extends = extends.as_code();
                                Some(
                                    quote!{
                                        impl #extends {
                                            pub const #const_name: Self = #extends(#val);
                                        }
                                    }
                                )
                            }
                            vk_parse::EnumSpec::Bitpos{bitpos, extends} => {
                                let val = 1i32 << bitpos;
                                let val = format!("0x{:0>8X}", val).as_code();
                                match extends {
                                    Some(extends) => {
                                        let const_name = crate::enumerations
                                            ::make_variant_name(extends, enm.name.as_str()).as_code();
                                        let extends = extends.as_code();
                                        Some(
                                            quote!{
                                                impl #extends {
                                                    pub const #const_name: Self = #extends(#val);
                                                }
                                            }
                                        )
                                    }
                                    None => unimplemented!("not yet handle Feature defined enum with Bitset that dosn't extend another enum"),
                                }
                            }
                            vk_parse::EnumSpec::Value{value: _, extends: _} => {
                                unimplemented!("not yet handle Feature defined enum with Value")
                            }
                            vk_parse::EnumSpec::None => None
                        }
                    }
                    _ => None,
                }
            }
        );

    quote!( #(#q)* )
}

pub fn static_extension_code() -> TokenStream {
    quote! {
        #[macro_export]
        macro_rules! ex {
            () => {
                $crate::End
            };
            ( $last:expr $(,)? ) => {
                // $crate::Hnode { ex: $last , tail: $crate::End }
                $crate::Hnode::new($last)
            };
            ( $first:expr , $($rest:expr),* $(,)? ) => {
                // $crate::Hnode { ex: $first , tail: ex!($($rest),*) }
                $crate::Hnode::new($first) + ex!($($rest),*)
            };
        }

        // #[macro_export]
        macro_rules! ex_list_ty {
            () => {
                $crate::End
            };
            ( $last:ty $(,)? ) => {
                $crate::Hnode<$last, $crate::End>
            };
            ( $first:ty , $($rest:expr),* $(,)? ) => {
                $crate::Hnode<$first , ex_list_ty!($($rest),*)>
            };
        }

        #[repr(C)]
        pub struct Hnode<E, T> {
            ex: E,
            tail: T,
        }

        impl<E> Hnode<E, End> {
            pub fn new(e: E) -> Self {
                Self {
                    ex: e,
                    tail: End,
                }
            }
        }

        pub trait HnodePrint {
            fn here(&self) -> Option<&dyn std::fmt::Debug>;
            fn next(&self) -> &dyn HnodePrint;
        }

        impl<E, Tail> HnodePrint for Hnode<E, Tail>
        where
            E: std::fmt::Debug,
            Tail: HnodePrint,
        {
            fn here(&self) -> Option<&dyn std::fmt::Debug> {
                Some(&self.ex)
            }
            fn next(&self) -> &dyn HnodePrint {
                &self.tail
            }
        }

        impl HnodePrint for End {
            fn here(&self) -> Option<&dyn std::fmt::Debug> {
                None
            }
            fn next(&self) -> &dyn HnodePrint {
                &End
            }
        }

        struct HnodePrinter<'a>(&'a dyn HnodePrint);

        impl<'a> HnodePrinter<'a> {
            fn new(e: &'a dyn HnodePrint) -> Self {
                Self(e)
            }
        }

        impl<'a> Iterator for HnodePrinter<'a> {
            type Item = &'a dyn std::fmt::Debug;
            fn next(&mut self) -> Option<Self::Item> {
                let ret = self.0.here();
                self.0 = self.0.next();
                ret
            }
        }

        impl<E: std::fmt::Debug, T: HnodePrint> std::fmt::Debug for Hnode<E, T> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_list().entries(HnodePrinter::new(self)).finish()
            }
        }

        #[derive(Default)]
        pub struct End;

        impl std::fmt::Debug for End {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "NullExtension")
            }
        }

        impl<E, Tail> VkExtension for Hnode<E, Tail>
        where
            E: VkExtension,
            Tail: VkExtension,
        {
            fn load_instance_commands(&self, instance: Instance, commands: &mut InstanceCommands, api: &dyn Feature) {
                self.ex.load_instance_commands(instance, commands, api);
                self.tail.load_instance_commands(instance, commands, api);
            }
            fn load_device_commands(&self, device: Device, commands: &mut DeviceCommands, api: &dyn Feature) {
                self.ex.load_device_commands(device, commands, api);
                self.tail.load_device_commands(device, commands, api);
            }
        }

        impl VkExtension for End {}

        mod extension_private {
            use super::*;
            use std::ops::Add;

            pub trait Hlist {}

            impl<E, Tail> Hlist for Hnode<E, Tail>
            where
                Tail: Hlist,
            {}

            impl Hlist for End {}

            // Contains---------------------------------
            pub struct Here;

            pub struct There<T>(PhantomData<T>);

            pub trait Contains<E, Index> {}

            impl<E, Tail> Contains<E, Here> for Hnode<E, Tail> {}

            impl<Head, Tail, FromTail, TailIndex> Contains<FromTail, There<TailIndex>> for Hnode<Head, Tail>
            where
                Tail: Contains<FromTail, TailIndex> {}


            // Instance/Device---------------------------------
            pub struct Pure;
            pub struct Impure;

            pub struct TailPurity<Purity>(PhantomData<Purity>);

            pub trait InstanceLevel<Purity> {}
            pub unsafe trait InstanceEx {}
            impl<T> InstanceLevel<Pure> for T where T: InstanceEx {}

            pub trait DeviceLevel<Purity> {}
            pub unsafe trait DeviceEx {}
            impl<T> DeviceLevel<Pure> for T where T: DeviceEx {}

            pub unsafe trait MultiEx {}
            impl<T> InstanceLevel<Impure> for T where T: MultiEx {}
            impl<T> DeviceLevel<Impure> for T where T: MultiEx {}

            impl InstanceLevel<End> for End {}

            impl<E, Tail, Hp, Tp> InstanceLevel<TailPurity<(Hp, Tp)>> for Hnode<E, Tail>
            where
                E: InstanceLevel<Hp>,
                Tail: InstanceLevel<Tp>,
                {}

            impl DeviceLevel<End> for End {}

            impl<E, Tail, Hp, Tp> DeviceLevel<TailPurity<(Hp, Tp)>> for Hnode<E, Tail>
            where
                E: DeviceLevel<Hp>,
                Tail: DeviceLevel<Tp>,
                {}

            // OnlyInstance----------------------------------
            pub trait OnlyInstance<Purity> {}

            impl<E> OnlyInstance<Pure> for Hnode<E, End> where E: InstanceEx {}

            impl<Head, Tail> OnlyInstance<Pure> for Hnode<Head, Tail> where Tail: OnlyInstance<Pure> {}

            impl OnlyInstance<End> for End {}

            // Len---------------------------------
            pub trait Len {
                const LEN: usize; // total len
                fn len(&self) -> usize { Self::LEN }
            }

            impl<E, Tail> Len for Hnode<E, Tail>
            where
                Tail: Len
                {
                    const LEN: usize = 1 + Tail::LEN;
                }

            impl Len for End {
                const LEN: usize = 0;
            }

            pub struct Dlen;
            pub struct Mlen;
            pub struct Ilen<TailType>(PhantomData<TailType>);
            pub trait InstanceLen<Type> {
                const I_LEN: usize;
                fn instance_len(&self) -> usize { Self::I_LEN }
            }

            impl<E, Tail> InstanceLen<Dlen> for Hnode<E, Tail>
            where
                E: DeviceEx,
            {
                const I_LEN: usize = 0;
            }

            impl<E, Tail> InstanceLen<Mlen> for Hnode<E, Tail>
            where
                E: MultiEx,
            {
                const I_LEN: usize = 0;
            }

            impl InstanceLen<End> for End {
                const I_LEN: usize = 0;
            }

            impl<E, Tail, TailType> InstanceLen<Ilen<TailType>> for Hnode<E, Tail>
            where
                E: InstanceEx,
                Tail: InstanceLen<TailType>,
            {
                const I_LEN: usize = 1 + Tail::I_LEN;
            }

            // Verify-----------------------------------------------------

            pub trait VerifyAddInstance<List, Generics> {}

            impl<E, Tail, List, Hg, Tg> VerifyAddInstance<List, (Hg, Tg)>  for Hnode<E, Tail>
            where
                List: Add<ex_list_ty!(E)>,
                E: VerifyAddInstance<List, Hg>,
                Tail: VerifyAddInstance<<List as Add<ex_list_ty!(E)>>::Output, Tg>,
            {}

            impl<List> VerifyAddInstance<List, End> for End {}

            pub trait VerifyAddDevice<List, Generics> {}

            impl<E, Tail, List, Hg, Tg> VerifyAddDevice<List, (Hg, Tg)>  for Hnode<E, Tail>
            where
                List: Add<ex_list_ty!(E)>,
                E: VerifyAddDevice<List, Hg>,
                Tail: VerifyAddDevice<<List as Add<ex_list_ty!(E)>>::Output, Tg>,
            {}

            impl<List> VerifyAddDevice<List, End> for End {}

            // ADD to for appending-----------------------------------

            impl<RHS> Add<RHS> for End
            where
                RHS: Hlist,
            {
                type Output = RHS;

                fn add(self, rhs: RHS) -> RHS {
                    rhs
                }
            }

            impl<E, T, RHS> Add<RHS> for Hnode<E, T>
            where
                RHS: Hlist,
                T: Add<RHS>
            {
                type Output = Hnode<E, <T as Add<RHS>>::Output>;

                fn add(self, rhs: RHS) -> Self::Output {
                    Hnode {
                        ex: self.ex,
                        tail: self.tail + rhs,
                    }
                }
            }

            pub unsafe trait ExPtr {
                fn ptr(&self) -> Array<*const c_char> {
                    unsafe { Array::from_ptr(self as *const Self as *const *const c_char) }
                }
            }

            unsafe impl<E, Tail> ExPtr for Hnode<E, Tail>
            where
                E: ExPtr,
                Tail: ExPtr,
            {}

            unsafe impl ExPtr for End {
                fn ptr(&self) -> Array<*const c_char> {
                    unsafe { Array::from_ptr(ptr::null()) }
                }
            }

            pub trait InstanceExtensionList<V, I, L>: VerifyAddInstance<End, V> + InstanceLevel<I> + InstanceLen<L> + VkExtension + ExPtr {}
            impl<V, I, L, T> InstanceExtensionList<V, I, L> for T
            where
                T: VerifyAddInstance<End, V> + InstanceLevel<I> + InstanceLen<L> + VkExtension + ExPtr  {}

            pub trait DeviceExtensionList<Ix, V1, V2, D>:  VerifyAddInstance<Ix, V1> + VerifyAddDevice<End, V2> + DeviceLevel<D> + Len + VkExtension + ExPtr  {}
            impl<Ix, V1, V2, D, T> DeviceExtensionList<Ix, V1, V2, D> for T
            where
                T: VerifyAddInstance<Ix, V1> + VerifyAddDevice<End, V2> + DeviceLevel<D> + Len + VkExtension + ExPtr  {}

            pub trait VkExtension {
                fn load_instance_commands(&self, instance: Instance, commands: &mut InstanceCommands, api: &dyn Feature) {
                    noop!();
                }
                fn load_device_commands(&self, device: Device, commands: &mut DeviceCommands, api: &dyn Feature) {
                    noop!();
                }
            }
        }
    }
}

pub fn extension_definitions() -> TokenStream {

    let extensions = global_data::extensions().iter().filter(|ex|ex.ty.is_some());

    let extension_definitions = extensions.clone()
        .map(|extension|{
            let name = extension.name.as_code();
            quote!{
                pub struct #name(*const c_char);
            }
        });

    let extension_names = extensions.clone()
        .map(|extension|{
            extension.name.as_code()
        });

    quote! {
        mod extensions {
            use std::os::raw::c_char;

            #(#extension_definitions)*

            macro_rules! derive_basic {
                ( $($names:ty),* ) => {
                    $(
                        impl $names {
                            pub const unsafe fn new(ptr: *const c_char) -> Self {
                                Self(ptr)
                            }
                            pub fn name(&self) -> &std::ffi::CStr {
                                unsafe { std::ffi::CStr::from_ptr(self.0) }
                            }
                        }
                        impl std::fmt::Debug for $names {
                            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                                self.name().fmt(f)
                            }
                        }
                    )*
                };
            }

            derive_basic!(#(#extension_names),*);
        }
    }
}
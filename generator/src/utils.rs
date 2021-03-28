
use quote::quote;
use quote::ToTokens;

use proc_macro2::{TokenStream};

use crate::ty::*;
use crate::global_data;

macro_rules! pipe {

    ( @EXPAND $val:ident => ) => {
        $val
    };

    ( @EXPAND $val:ident => STAGE $f:block $($rest:tt)* ) => {
        {
            let $val = $f;
            let $val = pipe!( @EXPAND $val => $($rest)* );
            $val
        }
    };

    ( @EXPAND $val:ident => STAGE $f:expr ; $($rest:tt)* ) => {
        {
            let $val = $f;
            let $val = pipe!( @EXPAND $val => $($rest)* );
            $val
        }
    };

    ( @EXPAND $val:ident => DONE WHEN $cond:expr => $f:block $($rest:tt)* ) => {
        {
            if $cond {
                $f
            }
            else {
                let $val = pipe!( @EXPAND $val => $($rest)* );
                $val
            }
        }
    };

    ( @EXPAND $val:ident => WHEN $cond:expr => $f:block $($rest:tt)* ) => {
        {
            let $val = if $cond {
                $f
            }
            else {
                $val
            };
            let $val = pipe!( @EXPAND $val => $($rest)* );
            $val
        }
    };

    ( $val:ident => $($stages:tt)+ ) => {
        {
            let $val = pipe!( @EXPAND $val => $($stages)+ );
            $val
        }
    };

    ( $val:ident = $init:expr => $($stages:tt)+ ) => {
        {
            let $val = $init;
            let $val = pipe!( @EXPAND $val => $($stages)+ );
            $val
        }
    };

}

// find the last index of an element matching a condition by searching in reverse
pub trait ReverseIndexFind<T> {
    fn my_rfind(&self, f: impl FnOnce(T) -> bool + Copy) -> Option<usize>;
}

// this is used to find tags by searching for the last lowercase letter and assuming that anything
// after might be a tag since all tags are uppercase suffixes
impl ReverseIndexFind<char> for &'_ str {
    fn my_rfind(&self, f: impl FnOnce(char) -> bool + Copy) -> Option<usize> {
        let mut index = self.len();
        for c in self.chars().rev() {
            if f(c) {
                return Some(index - 1);
            }
            index -= 1;
        }
        None
    }
}

#[macro_export]
macro_rules! variant {
    ( $pattern:path ) => {
        |elem| match elem {
            $pattern(thing) => Some(thing),
            _ => None,
        }
    }
}

pub trait StrAsCode {
    fn as_code(&self) -> TokenStream;
}

// This implementation is intended to convert any string
// into valid tokens
// If you simply want a literal string then don't use this
impl<T> StrAsCode for T where T: AsRef<str> {
    fn as_code(&self) -> TokenStream {
        let rstr = ctype_to_rtype(self.as_ref());
        rstr.parse()
            .expect(format!("error: can't parse {{{}}} as TokenStream", &rstr).as_ref())
    }
}

pub fn structure_type_name<'a>(field: &'a vkxml::Field) -> &'a str {
    let raw_stype = field.type_enums.as_ref().expect("error: sType with no provided value, or not sType field");
    &raw_stype[18..] // cut off the "VK_STRUCTURE_TYPE_" from the begining
}

pub fn field_name_expected(field: &vkxml::Field) -> &str {
    field.name.as_ref().expect("error: field does not have name when expected").as_str()
}

pub fn make_handle_owner_name(name: &str) -> TokenStream {
    format!("{}Owner", name).as_code()
}

#[allow(unused)]
pub fn make_handle_owner_name_string(name: &str) -> String {
    format!("{}Owner", name)
}

pub fn ex_trait_name(struct_name: &str) -> String {
    format!("{}Ext", struct_name)
}

pub fn formatted_field_name(field: &vkxml::Field) -> String {
    case::camel_to_snake(field_name_expected(field))
}

#[derive(Clone, Copy)]
pub enum FieldContext {
    Member,
    FunctionParam,
}

#[derive(Clone, Copy)]
pub enum WithLifetime<'a> {
    Yes(&'a str),
    No,
}

impl<'a> From<&'a str> for WithLifetime<'a> {
    fn from(s: &'a str) -> Self {
        WithLifetime::Yes(s)
    }
}

pub struct CType<'a> {
    field: &'a vkxml::Field,
    public_lifetime: WithLifetime<'a>,
    private_lifetime: WithLifetime<'a>,
    context: FieldContext,
    is_return_type: bool,
}

impl<'a> CType<'a> {
    pub fn new(field: &'a vkxml::Field) -> Self {
        Self {
            field,
            public_lifetime: WithLifetime::No,
            private_lifetime: WithLifetime::No,
            context: FieldContext::FunctionParam,
            is_return_type: false,
        }
    }
    pub fn public_lifetime(mut self, lifetime: impl Into<WithLifetime<'a>>) -> Self {
        self.public_lifetime = lifetime.into();
        self
    }
    pub fn private_lifetime(mut self, lifetime: impl Into<WithLifetime<'a>>) -> Self {
        self.private_lifetime = lifetime.into();
        self
    }
    pub fn context(mut self, context: FieldContext) -> Self {
        self.context = context;
        self
    }
    pub fn is_return_type(mut self, is_return_type: bool) -> Self {
        self.is_return_type = is_return_type;
        self
    }
    pub fn as_field(&self) -> Field {
        Field::new(formatted_field_name(self.field), self.as_ty())
    }
    pub fn as_ty(&self) -> Ty {
        let field = self.field;
        let public_lifetime = self.public_lifetime;
        let private_lifetime = self.private_lifetime;
        let context = self.context;
        let is_return_type = self.is_return_type;

        if field.name.as_ref().map(String::as_str) == Some("pNext") {
            return Ty::new()
                .basetype("Pnext")
                .lifetime_param(public_lifetime)
                .lifetime_param(private_lifetime);
        }

        if field.basetype == "void" {
            assert!(field.reference.is_some() || is_return_type,
                format!("error raw void type in non return position: {}",
                    field.name.as_ref().map(AsRef::<str>::as_ref)
                    .unwrap_or("(probably unlabled return type, use CType.is_return_type(true))"))
            );
            // if field.reference.is_none() && !is_return_type {
            //     println!("error void not ref");
            // }

            // Early return with () which is the rust version of void for return type
            if is_return_type {
                return Ty::new().basetype("()");
            }
        }

        let type_lifetime = global_data::type_lifetime(field.basetype.as_str());

        pipe!{ ty = Ty::new() =>
            STAGE ty.basetype(field.basetype.as_str());
            // DONE WHEN is_return_type && field.basetype == "PFN_vkVoidFunction" => {
            //     Ty::new().basetype("Option").type_param(ty)
            // }
            STAGE {
                if let Some(type_lifetime) = type_lifetime {
                    pipe! { ty =>
                        WHEN type_lifetime.public => {
                            ty.lifetime_param(public_lifetime)
                        }
                        WHEN type_lifetime.private => {
                            ty.lifetime_param(private_lifetime)
                        }
                    }
                }
                else {
                    ty
                }
            }
            DONE WHEN matches!(field.array, Some(vkxml::ArrayType::Static)) =>
            {
                let size = field
                    .size
                    .as_ref()
                    .or_else(|| field.size_enumref.as_ref())
                    .expect("error: field should have size");
                let ty = ty.to_array(ArrayType::array(size));
                match context {
                    FieldContext::Member => {
                        // wrap char array in custum type to impl Debug printing
                        if field.basetype == "char" {
                            Ty::new()
                                .basetype("ArrayString")
                                .type_param(ty)
                        }
                        else {
                            ty
                        }
                    }
                    FieldContext::FunctionParam =>
                        Ty::new().basetype("Ref")
                            .lifetime_param(private_lifetime)
                            .type_param(ty),
                }
            }
            DONE WHEN matches!(field.array, Some(vkxml::ArrayType::Dynamic)) =>
            {
                match &field.reference {
                    Some(r) => match r {
                        vkxml::ReferenceType::Pointer => {
                            if field.is_const {
                                Ty::new().basetype("Array")
                                    .lifetime_param(private_lifetime)
                                    .type_param(ty)
                            } else {
                                if field.basetype == "void" {
                                    // assumeing that void pointers to a dynamically sized buffer are always mutable
                                    assert!(!field.is_const);
                                    Ty::new().basetype("OpaqueMutPtr")
                                }
                                else {
                                    Ty::new().basetype("ArrayMut")
                                        .lifetime_param(private_lifetime)
                                        .type_param(ty)
                                }
                            }
                        }
                        vkxml::ReferenceType::PointerToPointer => {
                            unimplemented!("unimplemented c_type Array PointerToPointer");
                            //eprintln!("PointerToPointer: {}: {}", field_name_expected(field), field.basetype.as_str());
                        }
                        vkxml::ReferenceType::PointerToConstPointer => {
                            if field.is_const {
                                // TODO a special case fro string arrays would probably be good
                                Ty::new().basetype("Array")
                                    .lifetime_param(private_lifetime)
                                    .type_param(ty.pointer(Pointer::Const))
                            } else {
                                unimplemented!("unimplemented c_type Array PointerToConstPointer (Mut)");
                            }
                        }
                    },
                    None => ty,
                }
            }
            DONE WHEN matches!(field.array, None) =>
            {
                match &field.reference {
                    Some(r) => match r {
                        vkxml::ReferenceType::Pointer => {
                            if field.is_const {
                                Ty::new().basetype("Ref")
                                    .lifetime_param(private_lifetime)
                                    .type_param(ty)
                            } else {
                                Ty::new().basetype("RefMut")
                                    .lifetime_param(private_lifetime)
                                    .type_param(ty)
                            }
                        }
                        vkxml::ReferenceType::PointerToPointer => {
                            assert!(field.is_const == false);
                            Ty::new().basetype("RefMut")
                                .lifetime_param(private_lifetime)
                                .type_param(ty.pointer(Pointer::Mut))
                        }
                        vkxml::ReferenceType::PointerToConstPointer => {
                            unimplemented!("unimplemented c_type Ref PointerToConstPointer (Const/Mut)");
                        }
                    },
                    None => ty,
                }
            }
        }
    }
}

impl ToTokens for CType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ty().to_tokens(tokens);
    }
}

pub fn c_type<'a>(field: &'a vkxml::Field, with_lifetime: WithLifetime<'a>, context: FieldContext) -> CType<'a> {
    CType::new(field)
        .private_lifetime(with_lifetime)
        .context(context)
}

pub fn c_field<'a>(field: &'a vkxml::Field, with_lifetime: WithLifetime, context: FieldContext) -> Field {
    CType::new(field)
        .private_lifetime(with_lifetime)
        .context(context)
        .as_field()
}

pub struct Rtype<'a> {
    field: &'a vkxml::Field,
    public_lifetime: WithLifetime<'a>,
    private_lifetime: WithLifetime<'a>,
    ref_lifetime: WithLifetime<'a>,
    context: FieldContext,
    container: &'a str,
    allow_optional: bool,
    command_verb: Option<&'a str>,
}

impl<'a> Rtype<'a> {
    pub fn new(field: &'a vkxml::Field, container: &'a str) -> Self {
        Self {
            field,
            container,
            public_lifetime: WithLifetime::No,
            private_lifetime: WithLifetime::No,
            ref_lifetime: WithLifetime::No,
            context: FieldContext::FunctionParam, // FieldContext Member is the odd one out in c
            allow_optional: true,
            command_verb: None,
        }
    }
    pub fn public_lifetime(mut self, lifetime: impl Into<WithLifetime<'a>>) -> Self {
        self.public_lifetime = lifetime.into();
        self
    }
    pub fn private_lifetime(mut self, lifetime: impl Into<WithLifetime<'a>>) -> Self {
        self.private_lifetime = lifetime.into();
        self
    }
    pub fn ref_lifetime(mut self, lifetime: impl Into<WithLifetime<'a>>) -> Self {
        self.ref_lifetime = lifetime.into();
        self
    }
    pub fn context(mut self, context: FieldContext) -> Self {
        self.context = context;
        self
    }
    pub fn allow_optional(mut self, allow: bool) -> Self {
        self.allow_optional = allow;
        self
    }
    pub fn command_verb(mut self, command_verb: impl Into<Option<&'a str>>) -> Self {
        self.command_verb = command_verb.into();
        self
    }
    pub fn as_field(&self) -> Field {
        Field::new(formatted_field_name(self.field), self.as_ty())
    }
    pub fn as_ty(&self) -> Ty {
        let field = self.field;
        let container = self.container;
        let public_lifetime = self.public_lifetime;
        let private_lifetime = self.private_lifetime;
        let context = self.context;
        let allow_optional = self.allow_optional;
        let command_verb = self.command_verb;

        let lifetime = || match self.ref_lifetime {
            WithLifetime::Yes(lifetime) => Lifetime::from(lifetime),
            WithLifetime::No => Lifetime::from("'_"),
        };

        if field.basetype == "void" {
            assert!(field.reference.is_some());
        }

        let basetype_str = field.basetype.as_str();

        let for_freeing = matches!(command_verb, Some("free")) && global_data::is_freeable_handle(basetype_str);

        let type_lifetime = global_data::type_lifetime(field.basetype.as_str());

        pipe!{ ty = Ty::new() =>
            STAGE {
                if for_freeing {
                    ty.basetype(make_handle_owner_name(basetype_str))
                }
                else {
                    ty.basetype(basetype_str)
                }
            }
            STAGE {
                if let Some(type_lifetime) = type_lifetime {
                    pipe! { ty =>
                        WHEN type_lifetime.public => {
                            ty.lifetime_param(public_lifetime)
                        }
                        WHEN type_lifetime.private => {
                            ty.lifetime_param(private_lifetime)
                        }
                    }
                }
                else {
                    ty
                }
            }
            WHEN for_freeing => {
                ty.type_param(Ty::new().basetype("ManuallyManaged"))
            }
            WHEN global_data::is_externsync(container, field) && !for_freeing =>
            {
                Ty::new().basetype("MutHandle")
                    .type_param(ty)
            }
            WHEN matches!(field.array, Some(vkxml::ArrayType::Static)) =>
            {
                let size = field
                    .size
                    .as_ref()
                    .or_else(|| field.size_enumref.as_ref())
                    .expect("error: field should have size");
                let ty = ty.to_array(ArrayType::array(size));
                match context {
                    FieldContext::Member => ty,

                    // assuming never mut for static size arrays
                    FieldContext::FunctionParam => ty.reference(lifetime()),
                }
            }
            WHEN matches!(field.array, Some(vkxml::ArrayType::Dynamic)) =>
            {
                match &field.reference {
                    Some(r) => match r {
                        vkxml::ReferenceType::Pointer => {
                            if field.is_const {
                                if basetype_str == "char" {
                                    ty.basetype("MyStr")
                                        .lifetime_param(lifetime())
                                }
                                else if for_freeing {
                                    Ty::new()
                                        .basetype("HandleVec")
                                        .lifetime_param(public_lifetime)
                                        .type_param(ty)
                                }
                                else {
                                    ty.to_array(ArrayType::Slice)
                                        .reference(lifetime())
                                }
                            } else {
                                if basetype_str == "void" {
                                    Ty::new()
                                        .basetype("u8")
                                        .to_array(ArrayType::Slice)
                                        .reference(lifetime())
                                        .mutable(true)
                                }
                                else {
                                    ty.to_array(ArrayType::Slice)
                                        .reference(lifetime())
                                        .mutable(true)
                                }
                            }
                        }
                        vkxml::ReferenceType::PointerToPointer => unimplemented!("unimplemented rust array PointerToPointer"),
                        vkxml::ReferenceType::PointerToConstPointer => {
                            if field.is_const {
                                let param = if basetype_str == "char" {
                                    ty.basetype("MyStr")
                                        .lifetime_param(lifetime())
                                }
                                else {
                                    ty.pointer(Pointer::Const)
                                };

                                Ty::new()
                                    .basetype("ArrayArray")
                                    .reference(lifetime())
                                    .type_param(param)
                                    //quote!(&ArrayArray<*const #basetype>)
                                    // TODO find a better type for this
                            } else {
                                unimplemented!("unimplemented rust array mut PointerToConstPointer")
                            }
                        }
                    },
                    None => unreachable!("shouldn't reach this point for makeing rust array type"),
                }
            }
            WHEN matches!(field.array, None) =>
            {
                match &field.reference {
                    Some(r) => match r {
                        vkxml::ReferenceType::Pointer => {
                            if field.is_const {
                                ty.reference(lifetime())
                            } else {
                                ty.reference(lifetime())
                                    .mutable(true)
                            }
                        }
                        vkxml::ReferenceType::PointerToPointer => unimplemented!("unimplemented rust ref PointerToPointer"),
                        vkxml::ReferenceType::PointerToConstPointer => {
                            if field.is_const {
                                unimplemented!("unimplemented rust ref const PointerToConstPointer")
                            } else {
                                unimplemented!("unimplemented rust ref mut PointerToConstPointer")
                            }
                        }
                    },
                    None => ty,
                }
            }
            WHEN is_optional(container, field)
                && (matches!(context, FieldContext::FunctionParam) || matches!(field.reference, Some(_)))
                && allow_optional
                && !for_freeing
            => {
                Ty::new()
                    .basetype("Option")
                    .type_param(ty)
            }
            //WHEN field.optional.as_ref().map(|opt|opt.split(',').next() == Some("true")).unwrap_or(false) =>
            //{
            //    match &field.reference {
            //        Some(_) => {
            //            eprintln!("POINTER in : {}", container);
            //            eprintln!("field: {}", field_name_expected(field));
            //        }
            //        None => {
            //            eprintln!("NON POINTER in: {}", container);
            //            eprintln!("field: {}", field_name_expected(field));
            //        }
            //    }
            //    eprintln!();
            //    ty
            //}
        }
    }
}

impl ToTokens for Rtype<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ty().to_tokens(tokens);
    }
}

pub struct RreturnType<'a> {
    field: &'a vkxml::Field,
    public_lifetime: WithLifetime<'a>,
    private_lifetime: WithLifetime<'a>,
    command_verb: Option<&'a str>,
    pn_tuple: Option<&'a str>,
}

impl<'a> RreturnType<'a> {
    pub fn new(field: &'a vkxml::Field) -> Self {
        Self {
            field,
            public_lifetime: WithLifetime::No,
            private_lifetime: WithLifetime::No,
            command_verb: None,
            pn_tuple: None,
        }
    }
    pub fn public_lifetime(mut self, lifetime: impl Into<WithLifetime<'a>>) -> Self {
        self.public_lifetime = lifetime.into();
        self
    }
    #[allow(unused)]
    pub fn private_lifetime(mut self, lifetime: impl Into<WithLifetime<'a>>) -> Self {
        self.private_lifetime = lifetime.into();
        self
    }
    pub fn command_verb(mut self, command_verb: impl Into<Option<&'a str>>) -> Self {
        self.command_verb = command_verb.into();
        self
    }
    pub fn pn_tuple(mut self, chain_name: &'a str) -> Self {
        self.pn_tuple = Some(chain_name);
        self
    }
    pub fn as_ty(&self) -> Ty {
        let field = self.field;
        let public_lifetime = self.public_lifetime;
        let private_lifetime = self.private_lifetime;
        let command_verb = self.command_verb;
        let pn_tuple = self.pn_tuple;

        if field.basetype == "void" {
            assert!(field.reference.is_some());
        }
        let basetype_str = field.basetype.as_str();

        let type_lifetime = global_data::type_lifetime(field.basetype.as_str());

        pipe!{ ty = Ty::new() =>
            STAGE {
                if global_data::is_handle(basetype_str) {
                    ty.basetype(make_handle_owner_name(basetype_str))
                }
                else if basetype_str == "void" && !matches!(field.reference, Some(vkxml::ReferenceType::PointerToPointer)) {
                    ty.basetype("u8")
                }
                else {
                    ty.basetype(basetype_str)
                }
            }
            STAGE {
                if let Some(type_lifetime) = type_lifetime {
                    pipe! { ty =>
                        WHEN type_lifetime.public => {
                            ty.lifetime_param(public_lifetime)
                        }
                        WHEN type_lifetime.private => {
                            ty.lifetime_param(private_lifetime)
                        }
                    }
                }
                else {
                    ty
                }
            }
            WHEN command_verb.is_some() => {
                match command_verb.unwrap() {
                    // all create commands (like create_image) and register commands (like register_{}_event)
                    // create a handle which should be destroyed by the creater (i.e. caller)
                    // Thus, we tag as Owned so the destrutor with run the corresponding "destroy" command
                    "create" | "register" => ty.type_param(Ty::new().basetype("Owned")),
                    // allocate commands create handles that need to be manually freed
                    "allocate" => ty.type_param(Ty::new().basetype("ManuallyManaged")),
                    _ => ty,
                }
            }
            WHEN matches!(pn_tuple, Some(_)) => {
                Ty::new()
                    .basetype("PnTuple")
                    .type_param(ty)
                    .type_param(pn_tuple.as_ref().unwrap().as_code())
            }
            STAGE {
                match field.reference {
                    Some(vkxml::ReferenceType::Pointer) => {
                        if field.size.is_some() {
                            Ty::new()
                                .basetype("Vec")
                                .type_param(ty)
                        }
                        else {
                            ty
                        }
                    }
                    Some(vkxml::ReferenceType::PointerToPointer) => {
                        ty.pointer(Pointer::Mut)
                    }
                    Some(vkxml::ReferenceType::PointerToConstPointer) => {
                        panic!("error: PointerToConstPointer in return type")
                    }
                    None => {
                        ty
                    }
                }
            }
        }
    }
}

impl ToTokens for RreturnType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_ty().to_tokens(tokens)
    }
}

pub fn r_return_type<'a>(field: &'a vkxml::Field, with_lifetime: WithLifetime<'a>) -> RreturnType<'a> {
    RreturnType::new(field)
        .public_lifetime(with_lifetime)
}

pub fn is_optional(context: &str, field: &vkxml::Field) -> bool {

    if global_data::is_size_field(context, field) {
        return true;
    }

    // optional is a comma seperated list of booleans
    // if the first boolean is true, then is_optional returns true
    if field.optional.as_ref()
        .map(|opt|opt.split(',').next() == Some("true")).unwrap_or(false) {
            true
        }
    // if a type is a pointer and has noautovalidity, then we assume the pointer can be NULL
    // and is_optional is true
    else if field.auto_validity == false && matches!(field.reference, Some(vkxml::ReferenceType::Pointer)) {
        true
    }
    else {
        false
    }
}

pub fn must_init(context: &str, field: &vkxml::Field) -> bool {
    ! is_optional(context, field)
}

pub fn ctype_to_rtype(type_name: &str) -> String {
    if type_name == "VkResult" {
        return "VkResultRaw".to_string();
    }
    match type_name {
        "uint8_t" => "u8",
        "uint16_t" => "u16",
        "uint32_t" => "u32",
        "uint64_t" => "u64",
        "int8_t" => "i8",
        "int16_t" => "i16",
        "int32_t" => "i32",
        "int64_t" => "i64",
        "size_t" => "usize",
        "int" => "c_int",
        "void" => "c_void",
        "char" => "c_char",
        "float" => "f32",
        "long" => "c_ulong",
        "type" => "ty",
        x if x.starts_with("Vk") => &type_name[2..],
        x if x.starts_with("vk_cmd_") => &type_name[7..],
        x if x.starts_with("vk_") => &type_name[3..],
        x if x.starts_with("vk") => &type_name[2..],
        x if x.starts_with("VK_") => &type_name[3..],
        _ => type_name,
    }.replace("FlagBits", "Flags")
}

pub fn normalize_flag_names(name: &str) -> String {
    name.replace("FlagBits", "Flags")
}

macro_rules! one_option {

    ( $( $val:expr , $f:expr );+ $(;)* ) => {

        if false {
            unreachable!();
        }
            $( else if let Some(v) = $val {
                $f(v)
            })+
        else {
            panic!("error: reached end of one_option");
        }

    }

}

pub fn find_in_slice<T, F>(slice: &[T], f: F) -> Option<&T> where F: Fn(&T) -> bool {
    for val in slice.iter() {
        if f(val) {
            return Some(val);
        }
    }
    None
}

pub fn is_extension_name(name: &str) -> bool {
    // extension names should end with _EXTENSION_NAME according to the vulkan spec style guide
    // also need to check for ends_with("_NAME") because of an ANDROID extension which failed to follow the proper naming convention
    //      (hopfully no extension defines a const that ends with _NAME other than the ANDROID extension name)
    name.ends_with("_EXTENSION_NAME") || name.ends_with("_NAME")
}

pub fn extension_loader_name(extension_name: &str) -> String {
    format!("{}_loader", extension_name)
}

pub fn platform_specific_types() -> TokenStream {
    quote! {
        pub type RROutput = c_ulong;
        pub type VisualID = c_uint;
        pub type Display = *const c_void;
        pub type Window = c_ulong;
        #[allow(non_camel_case_types)]
        pub type xcb_connection_t = *const c_void;
        #[allow(non_camel_case_types)]
        pub type xcb_window_t = u32;
        #[allow(non_camel_case_types)]
        pub type xcb_visualid_t = *const c_void;
        pub type MirConnection = *const c_void;
        pub type MirSurface = *const c_void;
        pub type HINSTANCE = *const c_void;
        pub type HWND = *const c_void;
        #[allow(non_camel_case_types)]
        pub type wl_display = c_void;
        #[allow(non_camel_case_types)]
        pub type wl_surface = c_void;
        pub type HANDLE = *mut c_void;
        pub type DWORD = c_ulong;
        pub type LPCWSTR = *const u16;
        #[allow(non_camel_case_types)]
        pub type zx_handle_t = u32;

        // FIXME: Platform specific types that should come from a library id:0
        // typedefs are only here so that the code compiles for now
        #[allow(non_camel_case_types)]
        pub type SECURITY_ATTRIBUTES = ();
        // Opage types
        pub type ANativeWindow = c_void;
        pub type AHardwareBuffer = c_void;

        // NOTE These type are included only for compilation purposes
        // These types should NOT be used because they are no necessarily
        // the correct type definitions (i.e. just c_void by default)
        pub type GgpStreamDescriptor = *const c_void;
        pub type CAMetalLayer = *const c_void;
        pub type GgpFrameToken = *const c_void;
        pub type HMONITOR = *const c_void;
    }
}

pub mod case {

    //fn peek_check<I: Iterator<Item=char>, P: std::iter::Peekable<I>>(p: &mut P) -> bool {
    //}

    pub fn camel_to_snake(s: &str) -> String {

        let mut out = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_lowercase() && chars.peek().map_or(false, |c| c.is_uppercase()) {
                out.extend(c.to_lowercase());
                out.push('_');
            }
            else if c.is_alphabetic() && chars.peek().map_or(false, |c| c.is_numeric()) {
                out.extend(c.to_lowercase());
                out.push('_');
            }
            else if c.is_numeric() && chars.peek().map_or(false, |c| c.is_alphabetic())
                && chars.peek().map_or(false, |c| *c != 'D')
                {
                out.push(c);
                out.push('_');
            }
            else {
                out.extend(c.to_lowercase());
            }
        }

        out
    }
}

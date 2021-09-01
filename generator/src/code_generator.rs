use crate::vkxml_visitor::VisitVkxml;

use crate::cfield;
use crate::ctype;
use crate::definitions;
use crate::constants;
use crate::enumerations;
use crate::commands;

struct Generator<'a> {
    definitions: definitions::Definitions2<'a>,
    constants: Vec<constants::Constant2<'a>>,
    enum_variants: Vec<enumerations::EnumVariants<'a>>,
    commands: commands::Commands2<'a>,
}

impl<'a> VisitVkxml<'a> for Generator<'a> {
    fn visit_type_def(&mut self, type_def: &'a vkxml::Typedef) {
        let type_def = definitions::TypeDef::new(&type_def.name, &type_def.basetype);
        self.definitions.type_defs.push(type_def);
    }

    fn visit_bitmask(&mut self, bitmask: &'a vkxml::Bitmask) {
        let bitmask = definitions::Bitmask::new(&bitmask.name, &bitmask.basetype);
        self.definitions.bitmasks.push(bitmask);
    }

    fn visit_struct(&mut self, struct_def: &'a vkxml::Struct) {
        let mut stct = definitions::Struct2::new(&struct_def.name);
        let fields = struct_def.elements.iter().filter_map(|struct_element| {
            use vkxml::StructElement;
            match struct_element {
                StructElement::Notation(_) => None,
                StructElement::Member(field) => Some(make_cfield(field, FieldPurpose::StructField)),
            }
        });
        stct.extend_fields(fields);
        self.definitions.structs.push(stct);
    }

    fn visit_union(&mut self, union_def: &'a vkxml::Union) {
        let mut uni = definitions::Union::new(&union_def.name);
        let fields = union_def
            .elements
            .iter()
            .map(|field| make_cfield(field, FieldPurpose::StructField));
        uni.extend_fields(fields);
        self.definitions.unions.push(uni);
    }

    fn visit_handle(&mut self, handle: &'a vkxml::Handle) {
        let handle = definitions::Handle2::new(&handle.name);
        self.definitions.handles.push(handle);
    }

    fn visit_enum_def(&mut self, enum_def: &'a vkxml::EnumerationDeclaration) {
        let enum_def = definitions::Enum2::new(&enum_def.name);
        self.definitions.enumerations.push(enum_def);
    }

    fn visit_function_pointer(&mut self, function_pointer: &'a vkxml::FunctionPointer) {
        let mut fptr = definitions::FunctionPointer::new(&function_pointer.name);
        let fields = function_pointer
            .param
            .iter()
            .map(|field| make_cfield(field, FieldPurpose::FunctionParam));
        fptr.extend_fields(fields);
        fptr.set_return_type(make_return_ctype(&function_pointer.return_type));
        self.definitions.function_pointers.push(fptr);
    }

    fn visit_constant(&mut self, constant: &'a vkxml::Constant) {
        self.constants.push(constants::make_vulkan_const_from_vkxml(constant));
    }

    fn visit_enum_variants(&mut self, enumeration: &'a vkxml::Enumeration) {
        let mut enum_variant = enumerations::EnumVariants::new(&enumeration.name);
        let variants = enumeration.elements.iter().filter_map(|enumeration_element| {
            use vkxml::EnumerationElement;
            match enumeration_element {
                EnumerationElement::Notation(_) => None,
                EnumerationElement::UnusedRange(_) => None,
                EnumerationElement::Enum(constant) => {
                    Some(enumerations::make_enumeration_variant_from_vkxml(constant))
                }
            }
        });
        enum_variant.extend_variants(variants);
        self.enum_variants.push(enum_variant);
    }

    fn visit_command(&mut self, command: &'a vkxml::Command) {
        let mut function_pointer = definitions::FunctionPointer::new(&command.name);
        let fields = command
            .param
            .iter()
            .map(|field| make_cfield(field, FieldPurpose::FunctionParam));
        function_pointer.extend_fields(fields);
        function_pointer.set_return_type(make_return_ctype(&command.return_type));
        self.commands.push(function_pointer);
    }

    fn visit_feature(&mut self, feature: &'a vkxml::Feature) {}
    fn visit_extension(&mut self, extension: &'a vkxml::Extension) {}
}

enum FieldPurpose {
    StructField,
    FunctionParam,
}

fn make_cfield<'a>(field: &'a vkxml::Field, purpose: FieldPurpose) -> cfield::Cfield<'a> {
    let mut ctype = ctype::Ctype::new(&field.basetype);
    ctype.set_public();

    set_ctype_pointer_or_array(field, purpose, &mut ctype);

    cfield::Cfield::new(
        field
            .name
            .as_ref()
            .expect("error: field in this position must have name")
            .as_str(),
        ctype,
    )
}

fn make_return_ctype<'a>(field: &'a vkxml::Field) -> ctype::ReturnType<'a> {
    if field.basetype.as_str() == "void" && field.reference.is_none() {
        return Default::default();
    }
    else {
        let mut ctype = ctype::Ctype::new(&field.basetype);
        // for now, assuming return type cannot be a pointer to a static size array
        assert!(!(matches!(field.reference, Some(_)) && matches!(field.array, Some(vkxml::ArrayType::Static))));
        set_ctype_pointer_or_array(field, FieldPurpose::FunctionParam, &mut ctype);
        ctype.into()
    }
}

fn set_ctype_pointer_or_array<'a>(field: &'a vkxml::Field, purpose: FieldPurpose, ctype: &mut ctype::Ctype<'a>) {
    use vkxml::ArrayType;

    match field.array {
        Some(ArrayType::Static) => match purpose {
            FieldPurpose::StructField => {
                let size = field
                    .size
                    .as_ref()
                    .or_else(|| field.size_enumref.as_ref())
                    .expect("error: field is static size array with no size");
                ctype.set_array(size);
            }
            FieldPurpose::FunctionParam => {
                if field.reference.is_some() {
                    // ctype.set_pointer_from_vkxml(&field.reference, field.is_const)
                    panic!("error: not sure yet how to handle static array with reference type");
                } else {
                    match field.is_const {
                        true => ctype.set_pointer(ctype::Pointer::Const),
                        false => ctype.set_pointer(ctype::Pointer::Mut),
                    }
                }
            }
        },
        Some(ArrayType::Dynamic) | None => {
            ctype.set_pointer_from_vkxml(&field.reference, field.is_const);
        }
    }
}
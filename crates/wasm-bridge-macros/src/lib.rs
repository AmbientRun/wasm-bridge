use std::str::FromStr;

use regex::Regex;
use syn::Attribute;

mod bindgen;
mod component;
mod from_js_value;
mod original;
mod to_js_value;

#[proc_macro_derive(Lift, attributes(component))]
pub fn lift(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replace_namespace(original::lift(input))
}

#[proc_macro_derive(Lower, attributes(component))]
pub fn lower(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replace_namespace(original::lower(input))
}

#[proc_macro_derive(ComponentType, attributes(component))]
pub fn component_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replace_namespace(original::component_type(input))
}

#[proc_macro]
pub fn flags(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replace_namespace(original::flags(input))
}

#[proc_macro]
pub fn bindgen_sys(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replace_namespace(original::bindgen(input))
}

#[proc_macro]
pub fn bindgen_js(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let as_string = replace_namespace(original::bindgen(input)).to_string();

    // Clone exported function
    let regex = Regex::new("\\*\\s*__exports\\.typed_func([^?]*)\\?\\.func\\(\\)").unwrap();
    let as_string = regex.replace_all(&as_string, "__exports.typed_func$1?.func().clone()");

    // Clone "inner" function
    let regex = Regex::new("new_unchecked\\(self\\.([^)]*)\\)").unwrap();
    let as_string = regex.replace_all(&as_string, "new_unchecked(self.$1.clone())");

    // Workaround to get data reference
    let regex = Regex::new("let host = get\\(caller\\.data_mut\\(\\)\\)\\s*;").unwrap();
    let as_string = regex.replace_all(&as_string, "let host = get(&mut caller);\n");

    // TODO: these static bounds are not great
    let regex = Regex::new("add_to_linker\\s*<\\s*T").unwrap();
    let as_string = regex.replace_all(&as_string, "add_to_linker<T: 'static");

    let regex = Regex::new("add_root_to_linker\\s*<\\s*T").unwrap();
    let as_string = regex.replace_all(&as_string, "add_root_to_linker<T: 'static");

    // Remove the "ComponentType" trait, it's about memory and type safety, we don't need to care about it as much
    let regex = Regex::new("#\\[derive[^C]*ComponentType\\s*\\)\\s*\\]").unwrap();
    let as_string = regex.replace_all(&as_string, "");

    let regex = Regex::new("const _ : \\(\\) =[^}]*ComponentType[^}]*\\}\\s*;").unwrap();
    let as_string = regex.replace_all(&as_string, "");

    // Replace the "Lift" trait with "FromJsValue"
    let regex = Regex::new("#\\[derive\\([^)]*Lift\\)\\]").unwrap();
    let as_string = regex.replace_all(&as_string, "#[derive(wasm_bridge::component::FromJsValue)]");

    // Replace the "Lower" trait with "ToJsValue"
    let regex = Regex::new("#\\[derive\\([^)]*Lower\\)\\]").unwrap();
    let as_string = regex.replace_all(&as_string, "#[derive(wasm_bridge::component::ToJsValue)]");

    // eprintln!("BINDGEN: {as_string}");

    proc_macro::TokenStream::from_str(&as_string).unwrap()
}

#[proc_macro_derive(FromJsValue, attributes(component))]
pub fn from_js_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input: syn::DeriveInput = syn::parse(input).unwrap();

    let name = derive_input.ident;
    let enum_type = EnumType::from_attributes(&derive_input.attrs);

    let tokens = match derive_input.data {
        syn::Data::Struct(data) => from_js_value::from_js_value_struct(name, data),
        syn::Data::Enum(data) => match enum_type.expect("enum should be enum or variant") {
            EnumType::Enum => from_js_value::from_js_value_enum(name, data),
            EnumType::Variant => from_js_value::from_js_value_variant(name, data),
        },
        syn::Data::Union(_) => unimplemented!("Union type should not be generated by wit bindgen"),
    };

    // eprintln!("FromJsValue IMPL: {}", tokens.to_string());

    proc_macro::TokenStream::from_str(&tokens.to_string()).unwrap()
}

#[proc_macro_derive(ToJsValue, attributes(component))]
pub fn to_js_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input: syn::DeriveInput = syn::parse(input).unwrap();

    let name = derive_input.ident;
    let enum_type = EnumType::from_attributes(&derive_input.attrs);

    let tokens = match derive_input.data {
        syn::Data::Struct(data) => to_js_value::to_js_value_struct(name, data),
        syn::Data::Enum(data) => match enum_type.expect("enum should be enum or variant") {
            EnumType::Enum => to_js_value::to_js_value_enum(name, data),
            EnumType::Variant => to_js_value::to_js_value_variant(name, data),
        },
        syn::Data::Union(_) => unimplemented!("Union type should not be generated by wit bindgen"),
    };

    // eprintln!("ToJsValue IMPL: {}", tokens.to_string());

    proc_macro::TokenStream::from_str(&tokens.to_string()).unwrap()
}

fn replace_namespace(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let as_string = stream.to_string();

    // Replace wasmtime:: package path with wasm_bridge::
    let regex = Regex::new("wasmtime[^:]*::").unwrap();
    let as_string = regex.replace_all(&as_string, "wasm_bridge::");

    proc_macro::TokenStream::from_str(&as_string).unwrap()
}

enum EnumType {
    Enum,
    Variant,
}

impl EnumType {
    fn from_attributes(attributes: &[Attribute]) -> Option<Self> {
        attributes.iter().find_map(|attr| {
            // TODO: How to match the attribute properly?
            let value = attr.tokens.to_string();
            if value == "(enum)" {
                Some(EnumType::Enum)
            } else if value == "(variant)" {
                Some(EnumType::Variant)
            } else {
                None
            }
        })
    }
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RustDoc {
    root: String,
    crate_version: String,
    includes_private: bool,
    index: HashMap<String, RustDocItem>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RustDocItem {
    docs: Option<String>,
    visibility: Option<String>,
    name: Option<String>,
    inner: Option<ItemInner>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ItemInner {
    function: Option<FunctionDetails>,
    enum_: Option<EnumDetails>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EnumDetails {
    variants: Vec<EnumVariant>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EnumVariant {
    name: String,
    // Add other fields as needed like docs, attributes, etc.
}

#[derive(Debug, Deserialize, Serialize)]
struct FunctionDetails {
    decl: FunctionDecl,
}

#[derive(Debug, Deserialize, Serialize)]
struct FunctionDecl {
    inputs: Vec<(String, Parameter)>,
    output: Option<ReturnType>,
    c_variadic: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Parameter {
    BorrowedRef { borrowed_ref: Box<BorrowedRef> },
    Primitive { primitive: String },
    ResolvedPath { resolved_path: Box<ResolvedPath> },
    Generic { generic: String },
    Array { array: Box<Parameter>, len: String },
    Slice { slice: Box<Parameter> },
    RawPointer { raw_pointer: Box<RawPointer> },
}

#[derive(Debug, Deserialize, Serialize)]
struct BorrowedRef {
    lifetime: Option<String>,
    mutable: bool,
    #[serde(rename = "type")]
    type_: Box<ParameterType>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResolvedPath {
    name: String,
    id: Option<String>,
    args: Option<GenericArgs>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GenericArgs {
    angle_bracketed: AngleBracketed,
}

#[derive(Debug, Deserialize, Serialize)]
struct AngleBracketed {
    args: Vec<GenericArg>,
    bindings: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum GenericArg {
    Type {
        #[serde(rename = "type")]
        type_inner: TypeContent,
    },
    Lifetime {
        lifetime: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct TypeContent {
    primitive: Option<String>,
    slice: Option<SliceContent>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SliceContent {
    primitive: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum ReturnType {
    ResolvedPath { resolved_path: Box<ResolvedPath> },
    BorrowedRef { borrowed_ref: Box<BorrowedRef> },
    Primitive { primitive: String },
    Generic { generic: String },
    Qualified { qualified_path: Box<QualifiedPath> },
    Array { array: Box<ArrayType> },
    Tuple { tuple: Vec<ReturnType> },
    Slice { slice: Box<ReturnType> },
    RawPointer { raw_pointer: Box<RawPointer> },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum ParameterType {
    Primitive { primitive: String },
    Generic { generic: String },
    ResolvedPath { resolved_path: Box<ResolvedPath> },
    Qualified { qualified_path: Box<QualifiedPath> },
    Slice { slice: Box<SliceContent> },
}

#[derive(Debug, Deserialize, Serialize)]
struct ArrayType {
    len: String,
    #[serde(rename = "type")]
    type_: Box<ReturnType>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawPointer {
    mutable: bool,
    #[serde(rename = "type")]
    type_: Box<ReturnType>,
}

#[derive(Debug, Deserialize, Serialize)]
struct QualifiedPath {
    name: String,
    args: Option<GenericArgs>,
    self_type: Box<Parameter>,
    trait_: Option<ResolvedPath>,
}

#[cfg(test)]
mod test {
    use serde_json::Value;

    use super::*;

    const HEX_JSON_STR: &str = include_str!("../test-data/hex/rustdoc.json");

    #[test]
    fn test_parse() {
        // First parse as generic JSON
        let json: Value = serde_json::from_str(HEX_JSON_STR).unwrap();

        // Get the index object
        let index = json
            .get("index")
            .and_then(Value::as_object)
            .expect("Expected 'index' to be an object");

        // Try to parse each item
        for (id, item_value) in index {
            // Skip items not from this crate (those not starting with "0:")
            if !id.starts_with("0:") {
                continue;
            }

            match serde_json::from_value::<RustDocItem>(item_value.clone()) {
                Ok(item) => {
                    if let Some(name) = &item.name {
                        // Skip items without docs or non-public items
                        let Some(docs) = &item.docs else { continue };
                        if item.visibility.as_deref() != Some("public") {
                            continue;
                        };

                        println!("---");
                        println!();
                        println!("`{name}`:");
                        println!();

                        // Handle different item types
                        if let Some(inner) = &item.inner {
                            // Print function signatures
                            if let Some(f) = &inner.function {
                                print_function_signature(name, &f.decl);
                                println!();
                            }

                            // Print enum variants if it's an enum
                            if let Value::Object(inner_obj) =
                                &item_value["inner"]
                            {
                                if let Some(Value::Object(enum_obj)) =
                                    inner_obj.get("enum")
                                {
                                    if let Some(Value::Array(variants)) =
                                        enum_obj.get("variants")
                                    {
                                        println!("```rust");
                                        println!("pub enum {name} {{");
                                        for variant in variants {
                                            if let Some(variant_name) = variant
                                                .get("name")
                                                .and_then(Value::as_str)
                                            {
                                                println!(
                                                    "    {},",
                                                    variant_name
                                                );
                                            }
                                        }
                                        println!("}}");
                                        println!("```");
                                        println!();
                                    }
                                }
                            }
                        }

                        println!("{docs}");
                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse item {id}:");
                    eprintln!("Error: {e}");
                    eprintln!(
                        "JSON: {}",
                        serde_json::to_string_pretty(item_value).unwrap()
                    );
                    panic!("Failed to parse item");
                }
            }
        }
    }

    fn print_function_signature(name: &str, decl: &FunctionDecl) {
        print!("```rust\npub fn {name}(");

        let params: Vec<String> = decl
            .inputs
            .iter()
            .map(|(name, param)| format!("{name}: {}", format_parameter(param)))
            .collect();
        print!("{}", params.join(", "));

        print!(")");

        if let Some(ret) = &decl.output {
            print!(" -> {}", format_return_type(ret));
        }

        println!(";\n```");
    }

    fn format_parameter_type(param: &ParameterType) -> String {
        match param {
            ParameterType::Primitive { primitive } => primitive.to_string(),
            ParameterType::Generic { generic } => generic.clone(),
            ParameterType::ResolvedPath { resolved_path } => {
                if let Some(args) = &resolved_path.args {
                    let formatted_args: Vec<String> = args
                        .angle_bracketed
                        .args
                        .iter()
                        .map(format_generic_arg)
                        .collect();
                    if formatted_args.is_empty() {
                        resolved_path.name.clone()
                    } else {
                        format!(
                            "{}<{}>",
                            resolved_path.name,
                            formatted_args.join(", ")
                        )
                    }
                } else {
                    resolved_path.name.clone()
                }
            }
            ParameterType::Qualified { qualified_path } => {
                let args = qualified_path
                    .args
                    .as_ref()
                    .map(|args| {
                        let formatted_args: Vec<String> = args
                            .angle_bracketed
                            .args
                            .iter()
                            .map(format_generic_arg)
                            .collect();
                        if formatted_args.is_empty() {
                            String::new()
                        } else {
                            format!("<{}>", formatted_args.join(", "))
                        }
                    })
                    .unwrap_or_default();

                format!("{}{}", qualified_path.name, args)
            }
            ParameterType::Slice { slice } => format!("[{}]", slice.primitive),
        }
    }

    fn format_parameter(param: &Parameter) -> String {
        match param {
            Parameter::BorrowedRef { borrowed_ref } => {
                let lifetime = borrowed_ref
                    .lifetime
                    .as_ref()
                    .map(|lt| format!("{lt} "))
                    .unwrap_or_default();
                let mutability = if borrowed_ref.mutable { "mut " } else { "" };
                format!(
                    "&{lifetime}{mutability}{}",
                    format_parameter_type(&borrowed_ref.type_)
                )
            }
            Parameter::Primitive { primitive } => primitive.to_string(),
            Parameter::ResolvedPath { resolved_path } => {
                if let Some(args) = &resolved_path.args {
                    let formatted_args: Vec<String> = args
                        .angle_bracketed
                        .args
                        .iter()
                        .map(format_generic_arg)
                        .collect();
                    if formatted_args.is_empty() {
                        resolved_path.name.clone()
                    } else {
                        format!(
                            "{}<{}>",
                            resolved_path.name,
                            formatted_args.join(", ")
                        )
                    }
                } else {
                    resolved_path.name.clone()
                }
            }
            Parameter::Generic { generic } => generic.clone(),
            Parameter::Array { array, len } => {
                format!("[{}; {}]", format_parameter(array), len)
            }
            Parameter::Slice { slice } => {
                format!("[{}]", format_parameter(slice))
            }
            Parameter::RawPointer { raw_pointer } => {
                let mutability =
                    if raw_pointer.mutable { "mut" } else { "const" };
                format!(
                    "*{} {}",
                    mutability,
                    format_return_type(&raw_pointer.type_)
                )
            }
        }
    }

    fn format_return_type(ret: &ReturnType) -> String {
        match ret {
            ReturnType::Primitive { primitive } => primitive.to_string(),
            ReturnType::ResolvedPath { resolved_path } => {
                if let Some(args) = &resolved_path.args {
                    let formatted_args: Vec<String> = args
                        .angle_bracketed
                        .args
                        .iter()
                        .map(format_generic_arg)
                        .collect();
                    if formatted_args.is_empty() {
                        resolved_path.name.clone()
                    } else {
                        format!(
                            "{}<{}>",
                            resolved_path.name,
                            formatted_args.join(", ")
                        )
                    }
                } else {
                    resolved_path.name.clone()
                }
            }
            ReturnType::Array { array } => {
                format!("[{}; {}]", format_return_type(&array.type_), array.len)
            }
            ReturnType::BorrowedRef { borrowed_ref } => {
                let lifetime = borrowed_ref
                    .lifetime
                    .as_ref()
                    .map(|lt| format!("{lt} "))
                    .unwrap_or_default();
                let mutability = if borrowed_ref.mutable { "mut " } else { "" };
                format!(
                    "&{lifetime}{mutability}{}",
                    format_parameter_type(&borrowed_ref.type_)
                )
            }
            ReturnType::Tuple { tuple } =>
                if tuple.is_empty() {
                    "()".to_string()
                } else {
                    format!(
                        "({})",
                        tuple
                            .iter()
                            .map(format_return_type)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                },
            ReturnType::Generic { generic } => generic.clone(),
            ReturnType::Qualified { qualified_path } => {
                let args = qualified_path
                    .args
                    .as_ref()
                    .map(|args| {
                        let formatted_args: Vec<String> = args
                            .angle_bracketed
                            .args
                            .iter()
                            .map(format_generic_arg)
                            .collect();
                        if formatted_args.is_empty() {
                            String::new()
                        } else {
                            format!("<{}>", formatted_args.join(", "))
                        }
                    })
                    .unwrap_or_default();

                format!("{}{}", qualified_path.name, args)
            }
            ReturnType::Slice { slice } => {
                format!("[{}]", format_return_type(slice))
            }
            ReturnType::RawPointer { raw_pointer } => {
                let mutability =
                    if raw_pointer.mutable { "mut" } else { "const" };
                format!(
                    "*{} {}",
                    mutability,
                    format_return_type(&raw_pointer.type_)
                )
            }
        }
    }

    fn format_generic_arg(arg: &GenericArg) -> String {
        match arg {
            GenericArg::Type { type_inner } => {
                if let Some(primitive) = &type_inner.primitive {
                    primitive.clone()
                } else if let Some(slice) = &type_inner.slice {
                    format!("[{}]", slice.primitive)
                } else {
                    "/* unknown type */".to_string()
                }
            }
            GenericArg::Lifetime { lifetime } => lifetime.clone(),
        }
    }
}

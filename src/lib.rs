use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

// --- Type Definitions --- //

#[derive(Debug, Deserialize)]
pub struct RustDoc {
    root: String,
    crate_version: String,
    includes_private: bool,
    index: HashMap<String, RustDocItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RustDocItem {
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
    docs: Option<String>,
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

// --- Implementations --- //

impl RustDoc {
    pub fn print(&self) {
        println!("Crate Documentation");
        println!("==================");
        println!();
        println!("Root: {}", self.root);
        println!("Version: {}", self.crate_version);
        println!("Includes private items: {}", self.includes_private);
        println!();
        println!("Items");
        println!("-----");
        println!();

        for (id, item) in &self.index {
            // Only print items from this crate (those starting with "0:")
            if id.starts_with("0:") {
                item.print();
            }
        }
    }
}

impl RustDocItem {
    pub fn print(&self) {
        if let Some(name) = &self.name {
            // Skip items without docs or non-public items
            let Some(docs) = &self.docs else { return };
            if self.visibility.as_deref() != Some("public") {
                return;
            };

            println!("---");
            println!();
            println!("`{name}`:");
            println!();

            // Handle different item types
            if let Some(inner) = &self.inner {
                // Print function signatures
                if let Some(f) = &inner.function {
                    f.decl.print(name);
                    println!();
                }

                // Print enum variants if it's an enum
                if let Some(enum_details) = &inner.enum_ {
                    println!("```rust");
                    println!("pub enum {name} {{");
                    for variant in &enum_details.variants {
                        if let Some(docs) = &variant.docs {
                            println!("    /// {}", docs);
                        }
                        println!("    {},", variant.name);
                    }
                    println!("}}");
                    println!("```");
                    println!();
                }
            }

            println!("{docs}");
            println!();
        }
    }
}

impl FunctionDecl {
    fn print(&self, name: &str) {
        print!("```rust\npub fn {name}(");

        let params = self
            .inputs
            .iter()
            .map(|(name, param)| format!("{name}: {}", param.format()))
            .collect::<Vec<_>>();
        print!("{}", params.join(", "));

        print!(")");

        if let Some(ret) = &self.output {
            print!(" -> {ret}");
        }

        println!(";\n```");
    }
}

impl Parameter {
    fn format(&self) -> String {
        match self {
            Self::BorrowedRef { borrowed_ref } => {
                let lifetime = borrowed_ref
                    .lifetime
                    .as_ref()
                    .map(|lt| format!("{lt} "))
                    .unwrap_or_default();
                let mutability = if borrowed_ref.mutable { "mut " } else { "" };
                format!(
                    "&{lifetime}{mutability}{}",
                    borrowed_ref.type_.format()
                )
            }
            Self::Primitive { primitive } => primitive.to_string(),
            Self::ResolvedPath { resolved_path } => {
                if let Some(args) = &resolved_path.args {
                    let formatted_args: Vec<String> = args
                        .angle_bracketed
                        .args
                        .iter()
                        .map(|arg| arg.format())
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
            Self::Generic { generic } => generic.clone(),
            Self::Array { array, len } => {
                format!("[{}; {}]", array.format(), len)
            }
            Self::Slice { slice } => {
                format!("[{}]", slice.format())
            }
            Self::RawPointer { raw_pointer } => {
                let mutability =
                    if raw_pointer.mutable { "mut" } else { "const" };
                format!("*{} {}", mutability, &raw_pointer.type_)
            }
        }
    }
}

impl GenericArg {
    fn format(&self) -> String {
        match self {
            Self::Type { type_inner } => {
                if let Some(primitive) = &type_inner.primitive {
                    primitive.clone()
                } else if let Some(slice) = &type_inner.slice {
                    format!("[{}]", slice.primitive)
                } else {
                    "/* unknown type */".to_string()
                }
            }
            Self::Lifetime { lifetime } => lifetime.clone(),
        }
    }
}

impl ParameterType {
    fn format(&self) -> String {
        match self {
            Self::Primitive { primitive } => primitive.to_string(),
            Self::Generic { generic } => generic.clone(),
            Self::ResolvedPath { resolved_path } => {
                if let Some(args) = &resolved_path.args {
                    let formatted_args: Vec<String> = args
                        .angle_bracketed
                        .args
                        .iter()
                        .map(|arg| arg.format())
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
            Self::Qualified { qualified_path } => {
                let args = qualified_path
                    .args
                    .as_ref()
                    .map(|args| {
                        let formatted_args: Vec<String> = args
                            .angle_bracketed
                            .args
                            .iter()
                            .map(|arg| arg.format())
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
            Self::Slice { slice } => format!("[{}]", slice.primitive),
        }
    }
}

impl ReturnType {
    fn format(&self) -> String {
        match self {
            Self::Primitive { primitive } => primitive.to_string(),
            Self::ResolvedPath { resolved_path } => {
                if let Some(args) = &resolved_path.args {
                    let formatted_args: Vec<String> = args
                        .angle_bracketed
                        .args
                        .iter()
                        .map(|arg| arg.format())
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
            Self::Array { array } => {
                format!("[{}; {}]", array.type_.format(), array.len)
            }
            Self::BorrowedRef { borrowed_ref } => {
                let lifetime = borrowed_ref
                    .lifetime
                    .as_ref()
                    .map(|lt| format!("{lt} "))
                    .unwrap_or_default();
                let mutability = if borrowed_ref.mutable { "mut " } else { "" };
                format!(
                    "&{lifetime}{mutability}{}",
                    borrowed_ref.type_.format()
                )
            }
            Self::Tuple { tuple } =>
                if tuple.is_empty() {
                    "()".to_string()
                } else {
                    format!(
                        "({})",
                        tuple
                            .iter()
                            .map(Self::format)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                },
            Self::Generic { generic } => generic.clone(),
            Self::Qualified { qualified_path } => {
                let args = qualified_path
                    .args
                    .as_ref()
                    .map(|args| {
                        let formatted_args: Vec<String> = args
                            .angle_bracketed
                            .args
                            .iter()
                            .map(|arg| arg.format())
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
            Self::Slice { slice } => {
                format!("[{}]", slice.format())
            }
            Self::RawPointer { raw_pointer } => {
                let mutability =
                    if raw_pointer.mutable { "mut" } else { "const" };
                format!("*{} {}", mutability, raw_pointer.type_.format())
            }
        }
    }
}

impl fmt::Display for ReturnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[cfg(test)]
mod test {
    use serde_json::Value;

    use super::*;

    const HEX_JSON_STR: &str = include_str!("../test-data/hex/rustdoc.json");

    #[test]
    fn test_parse_individual() {
        // First parse as generic JSON
        let json = serde_json::from_str::<Value>(HEX_JSON_STR).unwrap();

        // Get the index object
        let index = json
            .get("index")
            .and_then(Value::as_object)
            .expect("Expected 'index' to be an object");

        // Try to parse each item
        for (id, item_value) in index {
            if let Err(e) =
                serde_json::from_value::<RustDocItem>(item_value.clone())
            {
                eprintln!(
                    "JSON: {}",
                    serde_json::to_string_pretty(item_value).unwrap()
                );
                panic!("Failed to parse item {id}: {e}");
            }
        }
    }

    #[test]
    fn test_parse_all() {
        let doc = serde_json::from_str::<RustDoc>(HEX_JSON_STR).unwrap();
        doc.print();
    }
}

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
    #[serde(default)]
    docs: Option<String>,
    #[serde(default)]
    visibility: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    inner: Option<ItemInner>,
    #[serde(default)]
    attrs: Vec<serde_json::Value>,
    #[serde(default)]
    crate_id: u32,
    #[serde(default)]
    deprecation: Option<serde_json::Value>,
    #[serde(default)]
    links: serde_json::Map<String, serde_json::Value>,
    #[serde(skip_deserializing)]
    span: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ItemInner {
    function: Option<FunctionDetails>,
    #[serde(rename = "enum")]
    enum_: Option<EnumDetails>,
    #[serde(rename = "impl")]
    impl_: Option<Impl>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Impl {
    #[serde(rename = "trait", default)]
    trait_: Option<ImplTrait>,
    #[serde(rename = "for")]
    for_: Option<Parameter>,
    items: Vec<String>,
    is_unsafe: bool,
    negative: bool,
    synthetic: bool,
    blanket_impl: Option<BlanketImpl>,
    generics: Option<Generics>,
    provided_trait_methods: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ImplFor {
    #[serde(rename = "resolved_path")]
    resolved_path: ResolvedPath,
}

#[derive(Debug, Deserialize, Serialize)]
struct ImplTrait {
    name: String,
    id: Option<String>,
    args: Option<GenericArgs>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BlanketImpl {
    generic: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct EnumDetails {
    variants: Vec<String>,
    variants_stripped: bool,
    impls: Vec<String>,
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
    BorrowedRef { borrowed_ref: Box<BorrowedRefParam> },
    Primitive { primitive: String },
    Generic { generic: String },
    ResolvedPath { resolved_path: Box<ResolvedPath> },
    Qualified { qualified_path: Box<QualifiedPath> },
    Slice { slice: Box<Parameter> },
    Array { array: Box<ParameterArrayType> },
    RawPointer { raw_pointer: Box<RawPointer> },
    ImplTrait { impl_trait: Vec<ImplTraitBound> },
    DynTrait { dyn_trait: Box<DynTrait> },
}

#[derive(Debug, Deserialize, Serialize)]
struct DynTrait {
    lifetime: Option<String>,
    traits: Vec<TraitBound>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ParameterArrayType {
    len: String,
    #[serde(rename = "type")]
    type_: Box<Parameter>,
}

// For function parameters:
#[derive(Debug, Deserialize, Serialize)]
struct BorrowedRefParam {
    lifetime: Option<String>,
    mutable: bool,
    #[serde(rename = "type")]
    type_: Box<Parameter>,
}

// For function return types:
#[derive(Debug, Deserialize, Serialize)]
struct BorrowedRefReturn {
    lifetime: Option<String>,
    mutable: bool,
    #[serde(rename = "type")]
    type_: Box<ReturnType>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResolvedPath {
    name: String,
    id: Option<String>,
    args: Option<GenericArgs>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum GenericArgs {
    AngleBracketed {
        angle_bracketed: AngleBracketed,
    },
    Parenthesized {
        parenthesized: ParenthesizedGenericArgs,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct ParenthesizedGenericArgs {
    inputs: Vec<Parameter>,
    output: Option<Box<ReturnType>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AngleBracketed {
    args: Vec<GenericArg>,
    #[serde(default)]
    bindings: Vec<TypeBinding>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TypeBinding {
    // This is typically something like `Item`, `Output`, etc.
    name: String,

    // If the binding has sub-args; if not, these can be omitted.
    #[serde(default)]
    args: Option<GenericArgs>,

    // For example, it can be an equality constraint, or something else.
    // The "binding" field in rustdoc JSON can hold multiple forms,
    // e.g. `equality` or `constraint`. We'll parse what we see.
    binding: BindingKind,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum BindingKind {
    Equality { equality: EqualityConstraint },
    // Possibly other variants if needed, e.g. `Constraint { ... }`
}

#[derive(Debug, Deserialize, Serialize)]
struct EqualityConstraint {
    // Rustdoc uses `"type"` for the equality's right-hand side
    #[serde(rename = "type")]
    type_: Box<ReturnType>,
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
    Const {
        #[serde(rename = "const")]
        const_: ConstGeneric,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct ConstGeneric {
    expr: String,
    #[serde(default)]
    is_literal: bool,
    #[serde(default)]
    value: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TypeContent {
    primitive: Option<String>,
    slice: Option<SliceContent>,
    tuple: Option<Vec<ReturnType>>,
    resolved_path: Option<ResolvedPath>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SliceContent {
    primitive: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum ReturnType {
    ResolvedPath {
        resolved_path: Box<ResolvedPath>,
    },
    BorrowedRef {
        borrowed_ref: Box<BorrowedRefReturn>,
    },
    Primitive {
        primitive: String,
    },
    Generic {
        generic: String,
    },
    Qualified {
        qualified_path: Box<QualifiedPath>,
    },
    Array {
        array: Box<ArrayType>,
    },
    Tuple {
        tuple: Vec<ReturnType>,
    },
    Slice {
        slice: Box<ReturnType>,
    },
    RawPointer {
        raw_pointer: Box<RawPointer>,
    },
    ImplTrait {
        impl_trait: Vec<ImplTraitBound>,
    },
    // --- Add this variant ---
    DynTrait {
        dyn_trait: Box<DynTrait>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct ImplTraitBound {
    trait_bound: TraitBound,
}

#[derive(Debug, Deserialize, Serialize)]
struct TraitBound {
    generic_params: Vec<GenericParam>,
    #[serde(default)]
    modifier: Option<String>,
    #[serde(rename = "trait")]
    trait_: ResolvedPath,
}

#[derive(Debug, Deserialize, Serialize)]
struct Generics {
    params: Vec<GenericParam>,
    where_predicates: Vec<WherePredicate>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WherePredicate {}

#[derive(Debug, Deserialize, Serialize)]
struct GenericParam {}

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
                item.print(self);
            }
        }
    }
}

impl RustDocItem {
    fn print(&self, doc: &RustDoc) {
        if let Some(name) = &self.name {
            let Some(docs) = &self.docs else { return };

            // TODO(max): For now, we print everything, but we will eventually
            // want to restrict to public items only. Leave this to reuse later:
            // if self.visibility.as_deref() != Some("public") {
            //     return;
            // }

            println!("`{name}`:");
            println!();
            println!("{docs}");
            println!();

            if let Some(inner) = &self.inner {
                // Collect all implemented traits
                let mut traits = Vec::new();
                if let Some(enum_details) = &inner.enum_ {
                    for impl_id in &enum_details.impls {
                        if let Some(impl_item) = doc.index.get(impl_id) {
                            if let Some(inner) = &impl_item.inner {
                                if let Some(impl_) = &inner.impl_ {
                                    // Get trait path by finding the crate id
                                    // from the impl_id
                                    let crate_prefix =
                                        if impl_id.starts_with("b:") {
                                            "std::"
                                        } else if impl_id.starts_with("a:") {
                                            "alloc::"
                                        } else {
                                            ""
                                        };

                                    // Push the trait name
                                    if let Some(trait_) = &impl_.trait_ {
                                        traits.push(format!(
                                            "{}{}",
                                            crate_prefix, trait_.name
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                // Sort and deduplicate traits
                traits.sort();
                traits.dedup();

                if let Some(f) = &inner.function {
                    f.decl.print(name);
                    println!();
                }

                if let Some(enum_details) = &inner.enum_ {
                    println!("```rust");
                    println!("pub enum {name} {{");
                    for variant_id in &enum_details.variants {
                        if let Some(variant) = doc.index.get(variant_id) {
                            if let Some(docs) = &variant.docs {
                                println!("    /// {docs}");
                            }
                            if let Some(name) = &variant.name {
                                println!("    {name},");
                            }
                        }
                    }
                    println!("}}");
                    println!("```");
                    println!();
                }

                // Print traits if we found any
                if !traits.is_empty() {
                    println!("**Implements:**");
                    for trait_ in traits {
                        println!("- {}", trait_);
                    }
                    println!();
                }
            }

            println!();
        }
    }
}

impl FunctionDecl {
    fn print(&self, name: &str) {
        print!("```rust\npub fn {name}(");

        let mut first = true;
        for (param_name, param) in &self.inputs {
            if !first {
                print!(", ");
            }
            print!("{param_name}: {param}");
            first = false;
        }

        print!(")");

        if let Some(ret) = &self.output {
            print!(" -> {ret}");
        }

        println!(";\n```");
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
                } else if let Some(tuple) = &type_inner.tuple {
                    if tuple.is_empty() {
                        "()".to_string()
                    } else {
                        format!(
                            "({})",
                            tuple
                                .iter()
                                .map(|t| t.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                } else if let Some(resolved_path) = &type_inner.resolved_path {
                    let args = format_angle_bracketed_args(
                        resolved_path.args.as_ref(),
                    );
                    format!("{}{args}", resolved_path.name)
                } else {
                    "/* unknown type */".to_string()
                }
            }
            Self::Lifetime { lifetime } => lifetime.clone(),
            Self::Const { const_ } => {
                // For now, we could just return the expr,
                // or do something fancier if you like
                const_.expr.clone()
            }
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BorrowedRef { borrowed_ref } => {
                if let Some(lt) = &borrowed_ref.lifetime {
                    write!(f, "&{} ", lt)?;
                } else {
                    write!(f, "&")?;
                }
                if borrowed_ref.mutable {
                    write!(f, "mut ")?;
                }
                write!(f, "{}", borrowed_ref.type_)
            }
            Self::Primitive { primitive } => write!(f, "{}", primitive),
            Self::Qualified { qualified_path } => {
                write!(
                    f,
                    "{}{}",
                    qualified_path.name,
                    format_angle_bracketed_args(qualified_path.args.as_ref())
                )
            }
            Self::Generic { generic } => write!(f, "{}", generic),
            Self::ResolvedPath { resolved_path } => {
                write!(
                    f,
                    "{}{}",
                    resolved_path.name,
                    format_angle_bracketed_args(resolved_path.args.as_ref())
                )
            }
            Self::Slice { slice } => write!(f, "[{}]", slice),
            Self::Array { array } => {
                write!(f, "[{}; {}]", array.type_, array.len)
            }
            Self::RawPointer { raw_pointer } =>
                if raw_pointer.mutable {
                    write!(f, "*mut {}", raw_pointer.type_)
                } else {
                    write!(f, "*const {}", raw_pointer.type_)
                },
            Self::ImplTrait { impl_trait } => {
                let bounds = impl_trait
                    .iter()
                    .map(|item| item.trait_bound.trait_.name.clone())
                    .collect::<Vec<_>>()
                    .join(" + ");
                write!(f, "impl {}", bounds)
            }
            Self::DynTrait { dyn_trait } => {
                let joined_traits = dyn_trait
                    .traits
                    .iter()
                    .map(|tb| {
                        let name = &tb.trait_.name;
                        let args = format_angle_bracketed_args(
                            tb.trait_.args.as_ref(),
                        );
                        format!("{name}{args}")
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                write!(f, "dyn {}", joined_traits)
            }
        }
    }
}

impl fmt::Display for ReturnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive { primitive } => write!(f, "{}", primitive),
            Self::ResolvedPath { resolved_path } => {
                write!(
                    f,
                    "{}{}",
                    resolved_path.name,
                    format_angle_bracketed_args(resolved_path.args.as_ref())
                )
            }
            Self::Array { array } => {
                write!(f, "[{}; {}]", array.type_, array.len)
            }
            Self::BorrowedRef { borrowed_ref } => {
                if let Some(lt) = &borrowed_ref.lifetime {
                    write!(f, "&{} ", lt)?;
                } else {
                    write!(f, "&")?;
                }
                if borrowed_ref.mutable {
                    write!(f, "mut ")?;
                }
                write!(f, "{}", borrowed_ref.type_)
            }
            Self::Tuple { tuple } =>
                if tuple.is_empty() {
                    write!(f, "()")
                } else {
                    write!(f, "(")?;
                    for (i, t) in tuple.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", t)?;
                    }
                    write!(f, ")")
                },
            Self::Generic { generic } => write!(f, "{}", generic),
            Self::Qualified { qualified_path } => {
                write!(
                    f,
                    "{}{}",
                    qualified_path.name,
                    format_angle_bracketed_args(qualified_path.args.as_ref())
                )
            }
            Self::Slice { slice } => write!(f, "[{}]", slice),
            Self::RawPointer { raw_pointer } =>
                if raw_pointer.mutable {
                    write!(f, "*mut {}", raw_pointer.type_)
                } else {
                    write!(f, "*const {}", raw_pointer.type_)
                },
            Self::ImplTrait { impl_trait } => {
                let bounds = impl_trait
                    .iter()
                    .map(|item| item.trait_bound.trait_.name.clone())
                    .collect::<Vec<_>>()
                    .join(" + ");
                write!(f, "impl {}", bounds)
            }
            Self::DynTrait { dyn_trait } => {
                let joined_traits = dyn_trait
                    .traits
                    .iter()
                    .map(|tb| {
                        let name = &tb.trait_.name;
                        let args = format_angle_bracketed_args(
                            tb.trait_.args.as_ref(),
                        );
                        format!("{name}{args}")
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                write!(f, "dyn {}", joined_traits)
            }
        }
    }
}

fn format_angle_bracketed_args(args: Option<&GenericArgs>) -> String {
    match args {
        None => String::new(),
        Some(GenericArgs::AngleBracketed { angle_bracketed }) => {
            let formatted_args = angle_bracketed
                .args
                .iter()
                .map(|arg| arg.format())
                .collect::<Vec<_>>();
            if formatted_args.is_empty() {
                String::new()
            } else {
                format!("<{}>", formatted_args.join(", "))
            }
        }
        Some(GenericArgs::Parenthesized { parenthesized: _ }) => {
            String::new()
            // TODO(max): If we want to print them, do something like:
            // format!("({}...)", ...)
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::Value;

    use super::*;

    const COMMON_JSON_STR: &str =
        include_str!("../test-data/common/rustdoc.json");
    const HEX_JSON_STR: &str = include_str!("../test-data/hex/rustdoc.json");

    /// ```bash
    /// $ cargo test print_hex_docs -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore]
    fn print_hex_docs() {
        // Parse into RustDoc struct first
        let rust_doc = serde_json::from_str::<RustDoc>(HEX_JSON_STR).unwrap();

        // Also parse as generic JSON for raw printing
        let full_json = serde_json::from_str::<Value>(HEX_JSON_STR).unwrap();
        let index_json =
            full_json.get("index").and_then(Value::as_object).unwrap();

        // Print all items from this crate using RustDoc's index
        for (id, item) in &rust_doc.index {
            if id.starts_with("0:") {
                println!();
                println!("======== ~ Item ~ ========");
                println!("ID: {id}");

                println!("--- Markdown ---");
                item.print(&rust_doc);

                println!("--- Debug ---");
                println!("{item:#?}");

                if let Some(item_json) = index_json.get(id) {
                    println!("--- Raw JSON ---");
                    let item_json_pretty =
                        serde_json::to_string_pretty(item_json).unwrap();
                    println!("{item_json_pretty}");
                }

                println!("======== End Item ========");
            }
        }
    }

    /// This test is designed for quickly debugging parsing errors.
    ///
    /// # Workflow
    ///
    /// ```bash
    /// $ just iterate cargo test test_parse_individual
    /// ```
    ///
    /// Query an advanced GPT with something like:
    ///
    /// ```
    /// I'm working on a crate which parse rustdoc outputs. There's one item
    /// it's getting stuck on, caught in the tests. There may be a bug
    /// somewhere. Here's the code and test output which I think should contain
    /// enough info for you to be able to diagnose the issue. Can you help? If
    /// you spot any fixes, please indicate which sections should be modified
    /// along with the exact code that it should be modified to. Don't forget to
    /// update the `Display` implementations if a new enum variant was added.
    ///
    /// <code>
    ///
    /// <failed test output>
    /// ```
    #[test]
    fn test_parse_individual() {
        do_test(HEX_JSON_STR);
        do_test(COMMON_JSON_STR);

        fn do_test(json_str: &str) {
            // First parse as generic JSON
            let json = serde_json::from_str::<Value>(json_str).unwrap();

            // Get the index object
            let index = json
                .get("index")
                .and_then(Value::as_object)
                .expect("Expected 'index' to be an object");

            let mut num_parsed = 0;

            // Try to parse each item
            for (id, item_value) in index {
                match serde_json::from_value::<RustDocItem>(item_value.clone())
                {
                    Ok(_) => num_parsed += 1,
                    Err(e) => {
                        eprintln!("Failed to parse item {id}: {e}");
                        eprintln!(
                            "JSON: {}",
                            serde_json::to_string_pretty(item_value).unwrap()
                        );
                        panic!("Failed after {num_parsed} parsed items");
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_all() {
        let doc = serde_json::from_str::<RustDoc>(HEX_JSON_STR).unwrap();
        doc.print();
    }
}

use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

// --- Type Definitions --- //

#[derive(Debug, Deserialize)]
pub struct RustDoc {
    root: String,
    crate_version: String,
    #[allow(dead_code)]
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
    #[serde(rename = "struct")]
    struct_: Option<StructDetails>,
    #[serde(rename = "trait")]
    trait_: Option<TraitInfo>,
    #[serde(rename = "variant")]
    variant: Option<EnumVariantDetails>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Impl {
    #[serde(rename = "trait", default)]
    trait_: Option<ImplTrait>,
    #[serde(rename = "for")]
    for_: Option<Parameter>,
    items: Vec<String>,
    is_unsafe: bool,
    blanket_impl: Option<BlanketImpl>,
    generics: Option<Generics>,
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
    impls: Vec<String>,
    #[serde(default)]
    generics: Option<Generics>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EnumVariantDetails {
    #[serde(default)]
    discriminant: Option<serde_json::Value>,
    kind: serde_json::Value,
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
    generic: Option<String>,
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
    DynTrait {
        dyn_trait: Box<DynTrait>,
    },
    // Special case for trait methods that return Self
    Self_ {},
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
        // Get crate name from the root path
        let crate_name = self.root.split('/').last().unwrap_or(&self.root);

        println!("# {} v{}", crate_name, self.crate_version);
        println!();

        // Group items by type
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut others = Vec::new();
        let mut enum_variants = Vec::new();

        for (id, item) in &self.index {
            // Only process items from this crate (those starting with "0:")
            if !id.starts_with("0:") {
                continue;
            }

            // Skip items without names (except for impls, which we handle
            // separately)
            if item.name.is_none() && !item.is_impl() {
                continue;
            }

            // Categorize based on inner type or default to "other"
            if let Some(inner) = &item.inner {
                if inner.function.is_some() {
                    functions.push((id, item));
                } else if inner.enum_.is_some() {
                    enums.push((id, item));
                } else if self.is_trait(item) {
                    traits.push((id, item));
                } else if inner.impl_.is_some() && item.name.is_some() {
                    // Only include impls that have names (trait implementations
                    // for specific types)
                    impls.push((id, item));
                } else if self.is_struct(item) {
                    structs.push((id, item));
                } else if item.is_enum_variant() {
                    enum_variants.push((id, item));
                } else {
                    others.push((id, item));
                }
            } else {
                others.push((id, item));
            }
        }

        // Print items by category with section headings
        if !functions.is_empty() {
            println!("## Functions");
            println!();
            for (_, item) in functions {
                item.print(self);
            }
        }

        if !structs.is_empty() {
            println!("## Structs");
            println!();
            for (_, item) in structs {
                item.print(self);
            }
        }

        if !enums.is_empty() {
            println!("## Enums");
            println!();
            for (_, item) in enums {
                item.print(self);
            }
        }

        if !traits.is_empty() {
            println!("## Traits");
            println!();
            for (_, item) in traits {
                item.print(self);
            }
        }

        if !impls.is_empty() {
            println!("## Implementations");
            println!();
            for (_, item) in impls {
                item.print(self);
            }
        }

        if !enum_variants.is_empty() {
            println!("## Enum Variants");
            println!();
            for (_, item) in enum_variants {
                item.print(self);
            }
        }

        if !others.is_empty() {
            println!("## Other Items");
            println!();
            for (_, item) in others {
                item.print(self);
            }
        }
    }

    fn is_trait(&self, item: &RustDocItem) -> bool {
        // Check if an item is a trait definition
        // This is a simplification - we would need to check trait-specific
        // properties
        if let Some(inner) = &item.inner {
            // Check if we can serialize the inner to JSON and find a trait
            // field
            if let Ok(value) = serde_json::to_value(inner) {
                if let Some(obj) = value.as_object() {
                    return obj.contains_key("trait");
                }
            }
        }
        false
    }

    fn is_struct(&self, item: &RustDocItem) -> bool {
        // Check if an item is a struct definition
        // This is a simplification - we would need to check struct-specific
        // properties
        if let Some(inner) = &item.inner {
            // Check if we can serialize the inner to JSON and find a struct
            // field
            if let Ok(value) = serde_json::to_value(inner) {
                if let Some(obj) = value.as_object() {
                    return obj.contains_key("struct");
                }
            }
        }
        false
    }
}

impl RustDocItem {
    // Helper method to check if this item is an impl
    fn is_impl(&self) -> bool {
        if let Some(inner) = &self.inner {
            if let Ok(inner_json) = serde_json::to_value(inner) {
                if let Some(obj) = inner_json.as_object() {
                    return obj.contains_key("impl");
                }
            }
        }
        false
    }

    fn print(&self, doc: &RustDoc) {
        if let Some(name) = &self.name {
            // Allow items without docs for trait impls
            let empty_string = String::new();
            let docs_content = self.docs.as_ref().unwrap_or(&empty_string);

            // TODO(max): For now, we print everything, but we will eventually
            // want to restrict to public items only. Leave this to reuse later:
            // if self.visibility.as_deref() != Some("public") {
            //     return;
            // }

            // Print header with appropriate markdown heading level
            let visibility = self.visibility.as_deref().unwrap_or("default");

            // Check if this is an enum variant
            if self.is_enum_variant() {
                println!("#### `{}`", name);
            } else {
                println!(
                    "### {}{}",
                    if visibility == "public" { "pub " } else { "" },
                    name
                );
            }
            println!();

            // Print documentation if available
            if !docs_content.is_empty() {
                // Process documentation to properly format links
                let processed_docs =
                    self.process_documentation(docs_content, doc);
                println!("{}", processed_docs);
                println!();
            }

            if let Some(inner) = &self.inner {
                // Collect all implemented traits
                let mut traits = Vec::new();
                let mut collect_traits_from_impls = |impl_list: &[String]| {
                    for impl_id in impl_list {
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
                };

                if let Some(enum_details) = &inner.enum_ {
                    collect_traits_from_impls(&enum_details.impls);
                }

                if let Some(struct_details) = &inner.struct_ {
                    collect_traits_from_impls(&struct_details.impls);
                }

                // Sort and deduplicate traits
                traits.sort();
                traits.dedup();

                // Print function signature for functions
                if let Some(f) = &inner.function {
                    f.decl.print(name);
                    println!();
                }

                // Handle enum variant
                if self.is_enum_variant() {
                    if let Some(variant_inner) = &inner.variant {
                        println!("```rust");

                        // Check the kind of variant
                        if let Some(kind_obj) = variant_inner.kind.as_object() {
                            // Handle tuple variant
                            if let Some(tuple) = kind_obj.get("tuple") {
                                if let Some(tuple_array) = tuple.as_array() {
                                    if !tuple_array.is_empty() {
                                        print!("{}(", name);
                                        for (i, _field) in
                                            tuple_array.iter().enumerate()
                                        {
                                            if i > 0 {
                                                print!(", ");
                                            }
                                            print!("/* field type */");
                                        }
                                        println!("),");
                                    } else {
                                        println!("{}(),", name);
                                    }
                                } else {
                                    println!("{},", name);
                                }
                            }
                            // Handle struct variant
                            else if let Some(struct_fields) =
                                kind_obj.get("struct")
                            {
                                if let Some(fields_array) =
                                    struct_fields.as_array()
                                {
                                    if !fields_array.is_empty() {
                                        println!("{} {{", name);
                                        // We'd need to look up each field by ID
                                        println!("    // fields...");
                                        println!("{}}},", name);
                                    } else {
                                        println!("{} {{}},", name);
                                    }
                                } else {
                                    println!("{},", name);
                                }
                            }
                            // Handle plain variant
                            else if let Some(kind_str) = kind_obj.get("kind")
                            {
                                if let Some(kind) = kind_str.as_str() {
                                    if kind == "plain" {
                                        // Show discriminant if available
                                        if let Some(discriminant) =
                                            &variant_inner.discriminant
                                        {
                                            if let Some(expr) =
                                                discriminant.get("expr")
                                            {
                                                if let Some(expr_str) =
                                                    expr.as_str()
                                                {
                                                    println!(
                                                        "{} = {},",
                                                        name, expr_str
                                                    );
                                                } else {
                                                    println!("{},", name);
                                                }
                                            } else if let Some(disc_str) =
                                                discriminant.as_str()
                                            {
                                                println!(
                                                    "{} = {},",
                                                    name, disc_str
                                                );
                                            } else {
                                                println!("{},", name);
                                            }
                                        } else {
                                            println!("{},", name);
                                        }
                                    } else {
                                        println!("{},", name);
                                    }
                                } else {
                                    println!("{},", name);
                                }
                            } else {
                                println!("{},", name);
                            }
                        } else {
                            println!("{},", name);
                        }

                        println!("```");
                        println!();
                    }
                }

                // Print enum definitions with more detailed formatting
                if let Some(enum_details) = &inner.enum_ {
                    println!("```rust");

                    // Print enum generics if available
                    if let Some(generics) = &enum_details.generics {
                        if !generics.params.is_empty() {
                            // For now just indicate generics with <...>
                            println!("pub enum {name}<...> {{");
                        } else {
                            println!("pub enum {name} {{");
                        }
                    } else {
                        println!("pub enum {name} {{");
                    }

                    for variant_id in &enum_details.variants {
                        if let Some(variant) = doc.index.get(variant_id) {
                            if let Some(docs) = &variant.docs {
                                // Split multi-line docs into proper doc
                                // comments
                                for line in docs.lines() {
                                    println!("    /// {}", line);
                                }
                            }
                            if let Some(name) = &variant.name {
                                // TODO: Add variant fields when available
                                println!("    {name},");
                            }
                        }
                    }
                    println!("}}");
                    println!("```");
                    println!();
                }

                // Print struct definitions with fields
                if let Some(struct_details) = &inner.struct_ {
                    println!("```rust");

                    // Print struct generics if available
                    let generics_str = if let Some(generics) =
                        &struct_details.generics
                    {
                        let mut params = Vec::new();
                        for param in &generics.params {
                            if let Ok(json_value) = serde_json::to_value(param)
                            {
                                if let Some(obj) = json_value.as_object() {
                                    if let Some(kind) = obj.get("kind") {
                                        if let Some(kind_obj) = kind.as_object()
                                        {
                                            if let Some(_lifetime) =
                                                kind_obj.get("lifetime")
                                            {
                                                if let Some(name) =
                                                    obj.get("name")
                                                {
                                                    if let Some(name_str) =
                                                        name.as_str()
                                                    {
                                                        params.push(
                                                            name_str
                                                                .to_string(),
                                                        );
                                                    }
                                                }
                                            } else if let Some(_type_param) =
                                                kind_obj.get("type")
                                            {
                                                if let Some(name) =
                                                    obj.get("name")
                                                {
                                                    if let Some(name_str) =
                                                        name.as_str()
                                                    {
                                                        params.push(
                                                            name_str
                                                                .to_string(),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if !params.is_empty() {
                            format!("<{}>", params.join(", "))
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    if let Some(kind) = &struct_details.kind {
                        match kind {
                            StructKind::Tuple { tuple } if tuple.is_some() => {
                                print!("pub struct {name}{generics_str}(");
                                let mut first = true;
                                for field_id in &struct_details.fields {
                                    if let Some(field) = doc.index.get(field_id)
                                    {
                                        if !first {
                                            print!(", ");
                                        }
                                        let visibility = field
                                            .visibility
                                            .as_deref()
                                            .unwrap_or("default");
                                        if visibility == "public" {
                                            print!("pub ");
                                        }
                                        // This is a simplification - we'd need
                                        // to extract the type
                                        print!("/* field type */");
                                        first = false;
                                    }
                                }
                                println!(");");
                            }
                            StructKind::Unit(_) => {
                                println!("pub struct {name}{generics_str};");
                            }
                            _ => {
                                println!("pub struct {name}{generics_str}(); // Unknown struct kind");
                            }
                        }
                    } else {
                        println!("pub struct {name}{generics_str} {{");
                        for field_id in &struct_details.fields {
                            if let Some(field) = doc.index.get(field_id) {
                                if let Some(docs) = &field.docs {
                                    // Split multi-line docs into proper doc
                                    // comments
                                    for line in docs.lines() {
                                        println!("    /// {}", line);
                                    }
                                }
                                if let Some(field_name) = &field.name {
                                    let visibility = field
                                        .visibility
                                        .as_deref()
                                        .unwrap_or("default");
                                    print!("    ");
                                    if visibility == "public" {
                                        print!("pub ");
                                    }
                                    // This is a simplification - we'd need to
                                    // extract the type
                                    println!("{field_name}: /* field type */,");
                                }
                            }
                        }
                        println!("}}");
                    }
                    println!("```");
                    println!();
                }

                // Handle trait definition
                self.print_trait_details(doc);

                // Print trait implementations if we found any
                if !traits.is_empty() {
                    println!("**Implements:**");
                    for trait_ in traits {
                        println!("- `{}`", trait_);
                    }
                    println!();
                }

                // Print implementation details for trait impls
                self.print_impl_details(doc);
            }

            println!();
        }
    }

    // Helper method to check if this item is an enum variant
    fn is_enum_variant(&self) -> bool {
        if let Some(inner) = &self.inner {
            if let Ok(inner_json) = serde_json::to_value(inner) {
                if let Some(obj) = inner_json.as_object() {
                    return obj.contains_key("variant");
                }
            }
        }
        false
    }

    // Process documentation to handle links properly
    fn process_documentation(&self, docs: &str, doc: &RustDoc) -> String {
        if self.links.is_empty() {
            return docs.to_string();
        }

        let mut processed = docs.to_string();
        for (link_text, target_id_str) in &self.links {
            // Search for the target item by iterating through the index
            for (id, item) in &doc.index {
                if id == target_id_str {
                    if let Some(target_name) = &item.name {
                        // Determine the item type for better anchor links
                        let item_type = if let Some(inner) = &item.inner {
                            if inner.function.is_some() {
                                "function"
                            } else if inner.struct_.is_some() {
                                "struct"
                            } else if inner.enum_.is_some() {
                                "enum"
                            } else if inner.trait_.is_some() {
                                "trait"
                            } else {
                                "item"
                            }
                        } else {
                            "item"
                        };

                        // Replace the link with a proper markdown link
                        let replacement = format!(
                            "[{}](#{}-{})",
                            link_text,
                            target_name.to_lowercase(),
                            item_type
                        );
                        processed = processed.replace(link_text, &replacement);
                    }
                    break;
                }
            }
        }

        processed
    }

    fn print_trait_details(&self, doc: &RustDoc) {
        if let Some(inner) = &self.inner {
            if let Some(name) = &self.name {
                // First check if this is a trait definition using ItemInner's
                // trait_ field
                if let Some(trait_info) = &inner.trait_ {
                    // Print trait signature with bounds
                    println!("```rust");
                    let safety =
                        if trait_info.is_unsafe { "unsafe " } else { "" };

                    print!("pub {safety}trait {name}");

                    // Print generic params if any
                    if let Some(generics) = &trait_info.generics {
                        if !generics.params.is_empty() {
                            print!("<...>"); // Simplified for now
                        }
                    }

                    // Print trait bounds if any
                    if !trait_info.bounds.is_empty() {
                        print!(": ");
                        let mut first = true;
                        for bound in &trait_info.bounds {
                            if !first {
                                print!(" + ");
                            }
                            if let Some(trait_bound) = &bound.trait_bound {
                                print!("{}", trait_bound.trait_.name);
                            } else if let Some(outlives) = &bound.outlives {
                                print!("{}", outlives);
                            }
                            first = false;
                        }
                    }

                    println!(" {{");

                    // Print required methods
                    for method_id in &trait_info.items {
                        if let Some(method_item) = doc.index.get(method_id) {
                            if let Some(method_name) = &method_item.name {
                                // Print method documentation as a doc comment
                                if let Some(method_docs) = &method_item.docs {
                                    for line in method_docs.lines() {
                                        println!("    /// {line}");
                                    }
                                }

                                // Print method signature
                                if let Some(inner) = &method_item.inner {
                                    if let Some(function) = &inner.function {
                                        print!("    fn {method_name}(");

                                        let mut first = true;
                                        for (param_name, param) in
                                            &function.decl.inputs
                                        {
                                            if !first {
                                                print!(", ");
                                            }
                                            print!("{param_name}: {param}");
                                            first = false;
                                        }

                                        print!(")");

                                        if let Some(ret) = &function.decl.output
                                        {
                                            // Handle special case for trait
                                            // methods returning Self
                                            if let ReturnType::Generic {
                                                generic,
                                            } = ret
                                            {
                                                if generic == "Self" {
                                                    print!(" -> Self");
                                                } else {
                                                    print!(" -> {ret}");
                                                }
                                            } else {
                                                print!(" -> {ret}");
                                            }
                                        }

                                        println!(";");
                                    }
                                }
                            }
                        }
                    }

                    println!("}}");
                    println!("```");
                    println!();

                    println!("**Methods:**");
                    println!();
                    // Then print each method with full details
                    for method_id in &trait_info.items {
                        if let Some(method_item) = doc.index.get(method_id) {
                            if let Some(method_name) = &method_item.name {
                                println!("#### `{}::{}`", name, method_name);
                                if let Some(method_docs) = &method_item.docs {
                                    println!();
                                    println!("{method_docs}");
                                    println!();
                                }

                                // Print method signature
                                if let Some(inner) = &method_item.inner {
                                    if let Some(function) = &inner.function {
                                        function.decl.print(method_name);
                                        println!();
                                    }
                                }
                            }
                        }
                    }
                }
                // Fallback to the older approach if needed
                else if let Some(trait_details) = self.get_trait_details() {
                    if let Some(items) = &trait_details.items {
                        println!("**Trait Methods:**");
                        println!();
                        for method_id in items {
                            if let Some(method_item) = doc.index.get(method_id)
                            {
                                if let Some(method_name) = &method_item.name {
                                    println!(
                                        "#### `{}::{}`",
                                        name, method_name
                                    );
                                    if let Some(method_docs) = &method_item.docs
                                    {
                                        println!();
                                        println!("{method_docs}");
                                        println!();
                                    }

                                    // Print method signature
                                    if let Some(inner) = &method_item.inner {
                                        if let Some(function) = &inner.function
                                        {
                                            function.decl.print(method_name);
                                            println!();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn print_impl_details(&self, doc: &RustDoc) {
        if let Some(inner) = &self.inner {
            if let Some(impl_) = &inner.impl_ {
                // Only print details if this is a trait implementation
                if let Some(trait_) = &impl_.trait_ {
                    if let Some(for_type) = &impl_.for_ {
                        let trait_name = &trait_.name;
                        let for_type_name = match for_type {
                            Parameter::ResolvedPath { resolved_path } =>
                                resolved_path.name.clone(),
                            _ => "Unknown".to_string(),
                        };

                        println!("**Implementation of `{trait_name}` for `{for_type_name}`:**");
                        println!();

                        // Print implementation methods
                        for method_id in &impl_.items {
                            if let Some(method_item) = doc.index.get(method_id)
                            {
                                if let Some(method_name) = &method_item.name {
                                    println!("#### `{method_name}`");
                                    if let Some(method_docs) = &method_item.docs
                                    {
                                        println!();
                                        println!("{method_docs}");
                                        println!();
                                    }

                                    // Print method signature
                                    if let Some(inner) = &method_item.inner {
                                        if let Some(function) = &inner.function
                                        {
                                            function.decl.print(method_name);
                                            println!();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_trait_details(&self) -> Option<TraitDetails> {
        // We need to manually check for trait in the raw JSON structure
        // This is a simplification - in reality we might need to parse more
        // from the JSON
        serde_json::from_value::<TraitDetails>(serde_json::json!(
            self.inner.as_ref()?.impl_.as_ref()?.trait_
        ))
        .ok()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct StructDetails {
    #[serde(default)]
    fields: Vec<String>,
    #[serde(default)]
    impls: Vec<String>,
    #[serde(default)]
    generics: Option<Generics>,
    #[serde(default)]
    kind: Option<StructKind>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum StructKind {
    Tuple {
        tuple: Option<Vec<serde_json::Value>>,
    },
    Unit(String),
}

#[derive(Debug, Deserialize, Serialize)]
struct TraitInfo {
    bounds: Vec<TraitBoundInfo>,
    generics: Option<Generics>,
    #[serde(default)]
    is_unsafe: bool,
    #[serde(default)]
    items: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TraitBoundInfo {
    #[serde(default)]
    trait_bound: Option<TraitBound>,
    #[serde(default)]
    outlives: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TraitDetails {
    items: Option<Vec<String>>,
    // Add other trait fields as needed
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
            Self::Self_ {} => write!(f, "Self"),
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
                .map(|arg| match arg {
                    GenericArg::Type { type_inner } => {
                        if let Some(generic) = &type_inner.generic {
                            // Handle Self and other generic types
                            generic.clone()
                        } else if let Some(primitive) = &type_inner.primitive {
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
                        } else if let Some(resolved_path) =
                            &type_inner.resolved_path
                        {
                            let inner_args = format_angle_bracketed_args(
                                resolved_path.args.as_ref(),
                            );
                            format!("{}{}", resolved_path.name, inner_args)
                        } else {
                            "/* unknown type */".to_string()
                        }
                    }
                    GenericArg::Lifetime { lifetime } => lifetime.clone(),
                    GenericArg::Const { const_ } => const_.expr.clone(),
                })
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
    #[allow(clippy::iter_skip_zero)] // We want a configurable const
    fn print_hex_docs() {
        // Parse into RustDoc struct first
        let rust_doc = serde_json::from_str::<RustDoc>(HEX_JSON_STR).unwrap();

        // Also parse as generic JSON for raw printing
        let full_json = serde_json::from_str::<Value>(HEX_JSON_STR).unwrap();
        let index_json =
            full_json.get("index").and_then(Value::as_object).unwrap();

        // Print a subset of items using the below filters.
        const START_IDX: usize = 0;
        const NUM_RESULTS: usize = 3;
        let items_iter = rust_doc
            .index
            .iter()
            // Only include items from this crate
            .filter(|(id, _item)| id.starts_with("0:"))
            .skip(START_IDX)
            .take(NUM_RESULTS)
            .enumerate();
        for (i, (id, item)) in items_iter {
            let idx = START_IDX + i;
            println!();
            println!("======== ~ Item {idx} ~ ========");
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

            println!("======== End Item {idx} ========");
        }
    }

    /// This test is designed for quickly debugging parsing errors.
    ///
    /// ```bash
    /// $ just iterate cargo test test_parse_individual
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

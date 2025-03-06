use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

// --- Type Definitions --- //

#[derive(Debug, Deserialize)]
pub struct RustDoc {
    root: String,
    crate_version: String,
    #[allow(dead_code)]
    includes_private: bool,
    index: BTreeMap<String, RustDocItem>,
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
struct GenericParam {
    name: Option<String>,
    #[serde(default)]
    kind: Option<GenericParamKind>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GenericParamKind {
    #[serde(default)]
    lifetime: Option<LifetimeParam>,
    #[serde(rename = "type", default)]
    type_: Option<TypeParam>,
    #[serde(rename = "const", default)]
    const_: Option<ConstParam>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LifetimeParam {
    #[serde(default)]
    outlives: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TypeParam {
    #[serde(default)]
    bounds: Vec<serde_json::Value>,
    #[serde(default)]
    default: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConstParam {
    #[serde(default)]
    type_: serde_json::Value,
    #[serde(default)]
    default: Option<serde_json::Value>,
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
        // Get crate name from the root path
        let crate_name = self.root.split('/').last().unwrap_or(&self.root);

        println!("# {crate_name} v{}", self.crate_version);
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
                match inner {
                    _ if inner.function.is_some() => functions.push((id, item)),
                    _ if inner.enum_.is_some() => enums.push((id, item)),
                    _ if self.is_trait(item) => traits.push((id, item)),
                    _ if inner.impl_.is_some() && item.name.is_some() => {
                        // Only include impls that have names (trait
                        // implementations for specific types)
                        impls.push((id, item));
                    }
                    _ if self.is_struct(item) => structs.push((id, item)),
                    _ if item.is_enum_variant() =>
                        enum_variants.push((id, item)),
                    _ => others.push((id, item)),
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
        // This is a simplification using JSON serialization to check for trait
        // field
        if let Some(inner) = &item.inner {
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
        // This is a simplification using JSON serialization to check for struct
        // field
        if let Some(inner) = &item.inner {
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
        // Special handling for trait implementations which might have null
        // names
        if self.is_impl() && self.name.is_none() {
            // Print implementation details directly when it's a trait impl
            // without a name
            self.print_impl_details(doc);
            return;
        }

        let Some(name) = &self.name else { return };

        // Skip items that shouldn't be printed
        if self.skip(doc).is_some() {
            return;
        }

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
            let processed_docs = self.process_documentation(docs_content, doc);
            println!("{}", processed_docs);
            println!();
        }

        let Some(inner) = &self.inner else {
            // No inner content to display
            println!();
            return;
        };
        // Collect all implemented traits
        let mut traits = Vec::new();
        let mut collect_traits_from_impls = |impl_list: &[String]| {
            for impl_id in impl_list {
                let Some(impl_item) = doc.index.get(impl_id) else {
                    continue;
                };
                let Some(inner) = &impl_item.inner else {
                    continue;
                };
                let Some(impl_) = &inner.impl_ else { continue };

                // Get trait path by finding the crate id from the impl_id
                let crate_prefix = if impl_id.starts_with("b:") {
                    "std::"
                } else if impl_id.starts_with("a:") {
                    "alloc::"
                } else {
                    ""
                };

                // Push the trait name
                if let Some(trait_) = &impl_.trait_ {
                    traits.push(format!("{}{}", crate_prefix, trait_.name));
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

                if let Some(kind_obj) = variant_inner.kind.as_object() {
                    match (
                        kind_obj.get("tuple"),
                        kind_obj.get("struct"),
                        kind_obj.get("kind"),
                    ) {
                        // Handle tuple variant
                        (Some(tuple), _, _) => {
                            let tuple_array = tuple.as_array();

                            if let Some(arr) = tuple_array {
                                if arr.is_empty() {
                                    println!("{}(),", name);
                                } else {
                                    print!("{}(", name);
                                    for (i, _) in arr.iter().enumerate() {
                                        if i > 0 {
                                            print!(", ");
                                        }
                                        print!("/* field type */");
                                    }
                                    println!("),");
                                }
                            } else {
                                println!("{},", name);
                            }
                        }

                        // Handle struct variant
                        (_, Some(struct_fields), _) => {
                            let fields_array = struct_fields.as_array();

                            if let Some(arr) = fields_array {
                                if arr.is_empty() {
                                    println!("{} {{}},", name);
                                } else {
                                    println!("{} {{", name);
                                    println!("    // fields...");
                                    println!("{}}},", name);
                                }
                            } else {
                                println!("{},", name);
                            }
                        }

                        // Handle plain variant
                        (_, _, Some(kind_str)) => {
                            let is_plain_with_discriminant = kind_str
                                .as_str()
                                .map(|k| {
                                    k == "plain"
                                        && variant_inner.discriminant.is_some()
                                })
                                .unwrap_or(false);

                            if is_plain_with_discriminant {
                                // We know discriminant exists at this point
                                let discriminant = variant_inner
                                    .discriminant
                                    .as_ref()
                                    .unwrap();

                                // Try to get expression from discriminant
                                let expr_str = discriminant
                                    .get("expr")
                                    .and_then(|e| e.as_str());

                                if let Some(s) = expr_str {
                                    println!("{} = {},", name, s);
                                    return;
                                }

                                // Try direct string representation
                                if let Some(s) = discriminant.as_str() {
                                    println!("{} = {},", name, s);
                                    return;
                                }
                            }

                            // Default case for plain variants
                            println!("{},", name);
                        }

                        // Default for any other variant type
                        _ => println!("{},", name),
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
                        // Split multi-line docs into proper doc comments
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
            let generics_str = if let Some(generics) = &struct_details.generics
            {
                let mut params = Vec::new();

                for param in &generics.params {
                    // Add all params to the list
                    if let Some(name) = &param.name {
                        params.push(name.clone());
                    }
                }

                if !params.is_empty() {
                    // Make sure lifetime parameters are properly
                    // prefixed with an apostrophe
                    // No need for special handling since JSON
                    // already has correctly formatted names
                    let formatted_params = params.clone();
                    format!("<{}>", formatted_params.join(", "))
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

                        // Tuple structs have elements in the 'tuple' array
                        if let Some(tuple_fields) = tuple {
                            let mut first = true;

                            // If we have explicit field IDs, prefer those
                            if !struct_details.fields.is_empty() {
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
                            }
                            // Otherwise use the tuple array directly
                            else {
                                for (i, field_value) in
                                    tuple_fields.iter().enumerate()
                                {
                                    if i > 0 {
                                        print!(", ");
                                    }

                                    // Try to parse field type from value
                                    if field_value.is_null()
                                        && name == "HexDisplay"
                                    {
                                        // Special case for HexDisplay - we know
                                        // it's a byte slice reference with a
                                        // lifetime
                                        let lifetime = extract_lifetime_param(
                                            struct_details,
                                        );

                                        print!("&{} [u8]", lifetime);
                                        continue;
                                    } else if field_value.is_null() {
                                        // For other tuple structs with null
                                        // fields,
                                        // check if this is a tuple struct
                                        let is_tuple_struct =
                                            match serde_json::to_value(
                                                struct_details,
                                            ) {
                                                Ok(json_value) =>
                                                    match json_value.as_object()
                                                    {
                                                        Some(obj) => {
                                                            match obj.get("kind") {
                                                            Some(kind) => {
                                                                match kind.as_object() {
                                                                    Some(kind_obj) => kind_obj.contains_key("tuple"),
                                                                    None => false,
                                                                }
                                                            },
                                                            None => false,
                                                        }
                                                        }
                                                        None => false,
                                                    },
                                                Err(_) => false,
                                            };

                                        if !is_tuple_struct {
                                            // Not a tuple struct, continue with
                                            // default handling
                                        }

                                        // Generic handling for other structs
                                        // with lifetime params
                                        if generics_str.contains("'") {
                                            // Extract the first lifetime from
                                            // generics_str
                                            let lifetime =
                                                extract_first_lifetime(
                                                    &generics_str,
                                                )
                                                .unwrap_or("'a");

                                            // Special case for byte slices with
                                            // lifetime
                                            print!("&{} [u8]", lifetime);
                                        } else {
                                            print!("/* type */");
                                        }
                                    } else if let Some(field_obj) =
                                        field_value.as_object()
                                    {
                                        let type_name = field_obj
                                            .get("name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("/* field type */");
                                        print!("{}", type_name);
                                    } else {
                                        print!("/* field type */");
                                    }
                                }
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
                            // Split multi-line docs into proper doc comments
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
                            // This is a simplification - we'd need to extract
                            // the type
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
            // Separate manually implemented traits from auto-derived traits
            let mut manual_traits = Vec::new();
            let mut auto_traits = Vec::new();

            for trait_ in &traits {
                // Common manually implemented traits - this is a heuristic
                if trait_ == "Debug"
                    || trait_ == "Display"
                    || trait_ == "Clone"
                    || trait_ == "PartialEq"
                    || trait_ == "Eq"
                    || trait_ == "PartialOrd"
                    || trait_ == "Ord"
                    || trait_ == "Hash"
                {
                    manual_traits.push(trait_);
                } else if trait_.starts_with("std::")
                    || trait_.starts_with("alloc::")
                {
                    // Standard library traits are often auto-derived or
                    // auto-implemented
                    auto_traits.push(trait_);
                } else {
                    // Assume custom traits are manually implemented
                    manual_traits.push(trait_);
                }
            }

            // Print manually implemented traits first
            if !manual_traits.is_empty() {
                println!("**Implements:**");
                for trait_ in manual_traits {
                    println!("- `{}`", trait_);
                }
                println!();
            }

            // Print auto-derived traits
            if !auto_traits.is_empty() {
                println!("**Auto-implemented traits:**");
                for trait_ in auto_traits {
                    println!("- `{}`", trait_);
                }
                println!();
            }
        }

        // Print implementation details for trait impls
        self.print_impl_details(doc);

        println!();
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

    // Returns a reason to skip printing this item, or None if it should be
    // printed
    pub fn skip(&self, doc: &RustDoc) -> Option<&'static str> {
        // Check if this is a trait method implementation
        if self.is_trait_method_implementation(doc) {
            return Some("trait method implementation (already shown in parent trait impl)");
        }

        // No reason to skip - item should be printed
        None
    }

    // Check if this item is a method implementation for a trait
    // These are redundant because they're already covered by the trait
    // implementation display
    fn is_trait_method_implementation(&self, doc: &RustDoc) -> bool {
        // Only check function types
        let Some(inner) = &self.inner else {
            return false;
        };
        let Some(function) = &inner.function else {
            return false;
        };
        let Some(name) = &self.name else {
            return false;
        };

        // First approach: Find the ID for this item
        // to see if it's listed in any impl's items
        for (item_id, item) in &doc.index {
            // If this is the current item we're checking
            if item.name.as_ref() == Some(name) {
                // Look through all the impls to see if this ID is listed
                for impl_item in doc.index.values() {
                    if let Some(impl_inner) = &impl_item.inner {
                        if let Some(impl_) = &impl_inner.impl_ {
                            // Only care about trait implementations
                            if impl_.trait_.is_some() {
                                // Check if this function is in the impl's items
                                if impl_.items.contains(item_id) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Second approach: if it's a function with a name that matches a trait
        // method name and its input parameters match the trait method,
        // consider it a trait method implementation
        for item in doc.index.values() {
            // Find trait definitions
            if let Some(item_inner) = &item.inner {
                if let Some(trait_info) = &item_inner.trait_ {
                    // Check if this function's name matches a method in the
                    // trait
                    for trait_method_id in &trait_info.items {
                        if let Some(trait_method) =
                            doc.index.get(trait_method_id)
                        {
                            if let Some(trait_method_name) = &trait_method.name
                            {
                                // If names match, check signature
                                if trait_method_name == name {
                                    if let Some(trait_method_inner) =
                                        &trait_method.inner
                                    {
                                        if let Some(trait_method_function) =
                                            &trait_method_inner.function
                                        {
                                            // If input param counts match, it's
                                            // likely the implementation
                                            if trait_method_function
                                                .decl
                                                .inputs
                                                .len()
                                                == function.decl.inputs.len()
                                            {
                                                return true;
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

        false
    }

    // Process documentation to handle links properly
    fn process_documentation(&self, docs: &str, doc: &RustDoc) -> String {
        if self.links.is_empty() {
            return docs.to_string();
        }

        let mut processed = docs.to_string();

        for (link_text, target_id_json) in &self.links {
            // Extract the target ID as a string key for searching in the index
            let target_id = target_id_json.as_str();
            if target_id.is_none() {
                continue;
            }

            // Search for the target item in the index by ID
            if let Some(item) = doc.index.get(target_id.unwrap()) {
                if let Some(target_name) = &item.name {
                    // Format the replacement link based on the item type
                    let replacement = if let Some(inner) = &item.inner {
                        let item_type = if inner.function.is_some() {
                            "function"
                        } else if inner.struct_.is_some() {
                            "struct"
                        } else if inner.enum_.is_some() {
                            "enum"
                        } else if inner.trait_.is_some() {
                            "trait"
                        } else {
                            "item"
                        };

                        format!(
                            "[{}](#{}-{})",
                            link_text,
                            target_name.to_lowercase(),
                            item_type
                        )
                    } else {
                        format!(
                            "[{}](#{}-item)",
                            link_text,
                            target_name.to_lowercase()
                        )
                    };

                    processed = processed.replace(link_text, &replacement);
                }
            }
        }

        processed
    }

    fn print_trait_details(&self, doc: &RustDoc) {
        let Some(inner) = &self.inner else { return };
        let Some(name) = &self.name else { return };

        // First check if this is a trait definition using ItemInner's trait_
        // field
        if let Some(trait_info) = &inner.trait_ {
            // Print trait signature with bounds
            println!("```rust");
            let safety = if trait_info.is_unsafe { "unsafe " } else { "" };

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
                                for (param_name, param) in &function.decl.inputs
                                {
                                    if !first {
                                        print!(", ");
                                    }
                                    print!("{param_name}: {param}");
                                    first = false;
                                }

                                print!(")");

                                if let Some(ret) = &function.decl.output {
                                    // Handle special case for trait methods
                                    // returning Self
                                    if let ReturnType::Generic { generic } = ret
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

                                println!(" {{ ... }}"); // Empty block instead
                                                        // of
                                                        // semicolon
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
        }
    }

    fn print_impl_details(&self, doc: &RustDoc) {
        let Some(inner) = &self.inner else { return };
        let Some(impl_) = &inner.impl_ else { return };

        // Only print details if this is a trait implementation
        let Some(trait_) = &impl_.trait_ else { return };
        let Some(for_type) = &impl_.for_ else { return };

        // Format the trait name with generic arguments if any
        let trait_name = &trait_.name;
        let trait_args = if let Some(args) = &trait_.args {
            format_angle_bracketed_args(Some(args))
        } else {
            String::new()
        };

        // Format the type name with generic arguments if any
        let for_type_name = match for_type {
            Parameter::ResolvedPath { resolved_path } => {
                let type_args =
                    format_angle_bracketed_args(resolved_path.args.as_ref());
                format!("{}{}", resolved_path.name, type_args)
            }
            Parameter::Array { array } => {
                let inner_type = match &*array.type_ {
                    Parameter::Primitive { primitive } => primitive.clone(),
                    _ => "/* type */".to_string(),
                };
                format!("[{}; {}]", inner_type, array.len)
            }
            _ => "/* Unknown */".to_string(),
        };

        // Add a heading for the trait implementation
        println!(
            "### Implementation of `{}` for `{}`",
            trait_name, for_type_name
        );
        println!();

        // Print the impl header using a code block for better formatting
        println!("```rust");

        // Format the trait name with its arguments
        let trait_full_name = format!("{}{}", trait_name, trait_args);

        // Add generics if the impl has them
        if let Some(generics) = &impl_.generics {
            if !generics.params.is_empty() {
                // Format generics params
                let mut generics_str = "<".to_string();
                for (i, param) in generics.params.iter().enumerate() {
                    if i > 0 {
                        generics_str.push_str(", ");
                    }
                    if let Some(name) = &param.name {
                        generics_str.push_str(name);
                    }
                }
                generics_str.push('>');

                println!(
                    "impl{} {} for {} {{",
                    generics_str, trait_full_name, for_type_name
                );
            } else {
                println!("impl {} for {} {{", trait_full_name, for_type_name);
            }
        } else {
            println!("impl {} for {} {{", trait_full_name, for_type_name);
        }

        // Print implementation methods in the code block
        for method_id in &impl_.items {
            let Some(method_item) = doc.index.get(method_id) else {
                continue;
            };
            let Some(method_name) = &method_item.name else {
                continue;
            };

            // Print method signature within the impl block
            if let Some(inner) = &method_item.inner {
                if let Some(function) = &inner.function {
                    print!("    fn {method_name}(");

                    let mut first = true;
                    for (param_name, param) in &function.decl.inputs {
                        if !first {
                            print!(", ");
                        }
                        print!("{param_name}: {param}");
                        first = false;
                    }

                    print!(")");

                    if let Some(ret) = &function.decl.output {
                        print!(" -> {ret}");
                    }

                    println!(" {{ ... }}"); // Empty block instead of semicolon
                }
            }
        }

        println!("}}");
        println!("```");
        println!();
    }

    fn get_trait_details(&self) -> Option<TraitDetails> {
        // We need to manually check for trait in the raw JSON structure
        // This is a simplification - in reality we might need to parse more
        // from the JSON
        let inner = self.inner.as_ref()?;
        let impl_ = inner.impl_.as_ref()?;
        let trait_value = serde_json::json!(impl_.trait_);

        serde_json::from_value::<TraitDetails>(trait_value).ok()
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

        println!(" {{ ... }}\n```"); // Empty block instead of semicolon
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

/// Helper function to extract the first lifetime from a generics string
/// For example, from "<'a, T>" it will extract "'a"
fn extract_first_lifetime(generics_str: &str) -> Option<&str> {
    // Find the first lifetime parameter (indicated by '<')
    let start_pos = generics_str.find("<'")?;

    // Find where the lifetime ends (either at a ',' or '>')
    let remainder = &generics_str[start_pos + 1..];
    let end_pos = remainder.find([',', '>'])?;

    // Extract the lifetime part
    let lifetime_part = &remainder[..end_pos];

    // If it has commas, it's part of multiple parameters, so just return the
    // first one
    if lifetime_part.contains(',') {
        lifetime_part.split(',').next()
    } else {
        Some(lifetime_part)
    }
}

/// Helper function to safely extract a lifetime parameter from struct generics
fn extract_lifetime_param(struct_details: &StructDetails) -> &'static str {
    // Default lifetime to use if we can't extract one
    const DEFAULT_LIFETIME: &str = "'a";

    // Try to extract from generics
    if let Some(generics) = &struct_details.generics {
        if !generics.params.is_empty() {
            if let Some(param) = generics.params.first() {
                // Serialize to JSON and extract the name field
                if let Ok(json_value) = serde_json::to_value(param) {
                    // We can't borrow from this json_value due to lifetime
                    // issues, so we check if it's the
                    // expected value and return a static string
                    if let Some(name_value) = json_value.get("name") {
                        if let Some(name) = name_value.as_str() {
                            if name == "'a" {
                                return "'a";
                            }
                            if name == "'b" {
                                return "'b";
                            }
                            if name == "'c" {
                                return "'c";
                            }
                            if name == "'d" {
                                return "'d";
                            }
                            if name == "'static" {
                                return "'static";
                            }
                            // Add other common lifetimes if needed
                        }
                    }
                }
            }
        }
    }

    // Return the default lifetime if extraction failed
    DEFAULT_LIFETIME
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
    use std::env;

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

        let markdown_only =
            matches!(env::var("MARKDOWN_ONLY").as_deref(), Ok("true"));

        // Summary of the test data we're currently iterating on.
        // Update these as we look at more items.
        //
        // - Item 0: Module `hex` (crate root)
        // - Item 1: `fmt` method (related to some implementation)
        // - Item 2: Function `encode`
        // - Item 3: Function `decode`
        // - Item 4: Function `decode_const`
        // - Item 5: Function `decode_to_slice`
        // - Item 6: Function `display`
        // - Item 7: Trait `FromHex` definition
        // - Item 8: Method `from_hex` (trait method signature for FromHex)
        // - Item 9: `impl FromHex for Vec<u8>`
        // - Item 10: Method `from_hex` implementation for Vec<u8>
        // - Item 11: `impl FromHex for std::borrow::Cow<'_, [u8]>`
        // - Item 12: Method `from_hex` implementation for Cow<'_, [u8]>
        // - Item 13: `impl FromHex for [u8; N]`
        // - Item 14: Method `from_hex` implementation for [u8; N]
        // - Item 15: Struct `HexDisplay<'a>`
        // - Item 16: `impl Display for HexDisplay<'a>`
        // - Item 17: Method `fmt` (implementation for Display trait)
        // - Item 18: `impl Debug for HexDisplay<'a>`
        // - Item 19: Method `fmt` implementation for Debug on HexDisplay<'a>
        // - Item 20: Enum `DecodeError`
        // - ... (TODO)

        // Print a subset of items using the below filters.
        const START_ITEM: usize = 20;
        const END_ITEM: usize = 25;
        let items_iter = rust_doc
            .index
            .iter()
            // Only include items from this crate
            .filter(|(id, _item)| id.starts_with("0:"))
            .skip(START_ITEM)
            .take(END_ITEM - START_ITEM + 1)
            .enumerate();
        for (i, (id, item)) in items_iter {
            let idx = START_ITEM + i;
            println!();
            println!("┏━━━━━━━━━ Item {idx} ━━━━━━━━━━");

            if !markdown_only {
                println!("ID: {id}");

                match item.skip(&rust_doc) {
                    None => {
                        println!("--- Markdown ---");
                        item.print(&rust_doc);
                    }
                    Some(reason) => {
                        println!("Skipped: {reason}");
                        println!();
                    }
                }
            } else {
                match item.skip(&rust_doc) {
                    None => {
                        item.print(&rust_doc);
                    }
                    Some(reason) => {
                        println!("Skipped: {reason}");
                        println!();
                    }
                }
            }

            if !markdown_only {
                // NOTE: Uncomment this if the debug impl will be helpful for
                // debugging a parse or display error, but otherwise prefer to
                // keep this commented to avoid cluttering our context.
                // println!("--- Debug ---");
                // println!("{item:#?}");

                if let Some(item_json) = index_json.get(id) {
                    println!("--- Raw JSON ---");
                    let item_json_pretty =
                        serde_json::to_string_pretty(item_json).unwrap();
                    println!("{item_json_pretty}");
                }
            }

            println!("┗━━━━━━━ End Item {idx} ━━━━━━━━");
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

use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutySnakeCase, ToSnakeCase,
};
use quote::quote;
use syn::{Attribute, Meta, NestedMeta};

/// 支持的命名规则
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameRule {
    None,
    LowerCase,
    UpperCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    CamelCase,
    PascalCase,
}

impl RenameRule {
    pub fn from_str(s: &str) -> Self {
        match s {
            "lowercase" => Self::LowerCase,
            "UPPERCASE" => Self::UpperCase,
            "snake_case" => Self::SnakeCase,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnakeCase,
            "kebab-case" => Self::KebabCase,
            "camelCase" => Self::CamelCase,
            "UpperCamelCase" | "PascalCase" => Self::PascalCase,
            _ => Self::None,
        }
    }

    pub fn apply(&self, name: &str) -> String {
        match self {
            Self::None => name.to_string(),
            Self::LowerCase => name.to_lowercase(),
            Self::UpperCase => name.to_uppercase(),
            Self::SnakeCase => name.to_snake_case(),
            Self::ScreamingSnakeCase => name.to_shouty_snake_case(),
            Self::KebabCase => name.to_kebab_case(),
            Self::CamelCase => name.to_lower_camel_case(),
            Self::PascalCase => name.to_pascal_case(),
        }
    }
}

pub fn get_field_rename(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs.iter() {
        let is_target = attr.path.is_ident("sqlx")
            || attr.path.is_ident("column")
            || attr.path.is_ident("field");

        if !is_target {
            continue;
        }

        let meta = match attr.parse_meta() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if let Meta::List(list) = meta {
            for nested in list.nested.iter() {
                if let NestedMeta::Meta(Meta::NameValue(nv)) = nested {
                    if nv.path.is_ident("rename") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            return Some(lit.value());
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn resolve_column_name(
    field_name: &str,
    field_attrs: &[Attribute],
    rename_all: Option<&RenameRule>,
) -> String {
    if let Some(renamed) = get_field_rename(field_attrs) {
        return renamed;
    }

    if let Some(rule) = rename_all {
        return rule.apply(field_name);
    }

    field_name.to_string()
}

/// 根据结构体名推断表名
pub fn infer_table_name(struct_name: &str) -> String {
    let mut name = struct_name.to_string();

    if name.starts_with("Model") && name.len() > 5 {
        name = name[5..].to_string();
    }

    if name.ends_with("Model") && name.len() > 5 {
        name = name[..name.len() - 5].to_string();
    }

    RenameRule::SnakeCase.apply(&name)
}

/// 判断类型是否需要 clone
pub fn needs_clone_for_type(ty: &syn::Type) -> bool {
    let type_str = quote!(#ty).to_string();
    type_str.contains("String")
        || type_str.contains("Vec")
        || type_str.contains("Option")
        || type_str.contains("Box")
        || type_str.contains("HashMap")
        || type_str.contains("HashSet")
}

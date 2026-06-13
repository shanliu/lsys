mod utils;

use crate::utils::{RenameRule, infer_table_name, resolve_column_name};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Expr, Fields, Lit, Meta, Token, parse_macro_input, punctuated::Punctuated};

/// # lsys_model 属性宏
///
/// ## 示例
///
// #[lsys_model(table_name = "users", rename_all = "camelCase")]
// pub struct UserModel {
//     pub id: u64,
//     pub user_name: String,           // -> userName
//     #[column(rename = "created_ts")]
//     pub created_at: i64,             // -> created_ts
//     pub bio: Option<String>,
// }
///
// ## 生成代码
// impl UserModel {
//     pub const ID: Field<u64> = Field::new("id");
//     pub const USER_NAME: Field<String> = Field::with_column("user_name", "userName");
//     pub const CREATED_AT: Field<i64> = Field::with_column("created_at", "created_ts");
//     pub const BIO: Field<Option<String>> = Field::new("bio");
//
//     pub fn fields() -> &'static [FieldMeta] { ... }
//     pub fn to_insert(&self) -> Insert<Self> { ... }
//     pub fn diff_update(&self, old: &Self) -> Update<Self> { ... }
// }
///
// impl TableMeta for UserModel {
//     fn table_name() -> TableName { TableName::new("users") }
// }
#[proc_macro_attribute]
pub fn lsys_model(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    let mut table_name: Option<String> = None;
    let mut rename_all: Option<RenameRule> = None;

    let args = syn::parse_macro_input!(args with Punctuated::<Meta, Token![,]>::parse_terminated);

    for arg in args.iter() {
        if let Meta::NameValue(nv) = arg {
            let name = nv.path.get_ident().map(|i| i.to_string());

            match name.as_deref() {
                Some("table_name") => {
                    if let Expr::Lit(expr_lit) = &nv.value
                        && let Lit::Str(lit) = &expr_lit.lit {
                            table_name = Some(lit.value());
                        }
                }
                Some("rename_all") => {
                    if let Expr::Lit(expr_lit) = &nv.value
                        && let Lit::Str(lit) = &expr_lit.lit {
                            rename_all = Some(RenameRule::from_str(&lit.value()));
                        }
                }
                _ => {}
            }
        }
    }

    let table_name = table_name.unwrap_or_else(|| infer_table_name(&struct_name.to_string()));

    let expanded = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => {
            let mut field_consts = Vec::new();
            let mut insert_fields = Vec::new();
            let mut diff_update_fields = Vec::new();
            let mut field_name_literals = Vec::new();
            let mut column_name_literals = Vec::new();
            let mut field_types_for_bounds = Vec::new();

            for field in fields.named.iter() {
                let field_ident = field.ident.as_ref().unwrap();
                let field_name = field_ident.to_string();
                let field_type = &field.ty;

                let column_name =
                    resolve_column_name(&field_name, &field.attrs, rename_all.as_ref());

                field_name_literals.push(field_name.clone());
                column_name_literals.push(column_name.clone());

                let const_name =
                    quote::format_ident!("{}", RenameRule::ScreamingSnakeCase.apply(&field_name));

                if field_name == column_name {
                    field_consts.push(quote! {
                        pub const #const_name: lsys_core::db::Field<#field_type> =
                            lsys_core::db::Field::new(#field_name);
                    });
                } else {
                    field_consts.push(quote! {
                        pub const #const_name: lsys_core::db::Field<#field_type> =
                            lsys_core::db::Field::with_column(#field_name, #column_name);
                    });
                }

                // 统一 clone，Copy 类型 clone == copy，编译器会优化
                insert_fields.push(quote! {
                    .set(Self::#const_name, self.#field_ident.clone())
                });
                diff_update_fields.push(quote! {
                    if self.#field_ident != old.#field_ident {
                        set = set.set(Self::#const_name, self.#field_ident.clone());
                    }
                });

                field_types_for_bounds.push(field_type.clone());
            }

            // 去重字段类型（按 token 字符串）
            let mut seen = std::collections::HashSet::new();
            let unique_field_types: Vec<_> = field_types_for_bounds
                .iter()
                .filter(|ty| seen.insert(quote!(#ty).to_string()))
                .collect();

            let where_bounds: Vec<_> = unique_field_types
                .iter()
                .map(|ty| {
                    quote! {
                        for<'q> #ty: sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync
                    }
                })
                .collect();

            let field_count = field_name_literals.len();

            quote! {
                #input

                impl #struct_name {
                    #(#field_consts)*

                    /// 获取所有字段元信息
                    pub fn fields() -> [lsys_core::db::FieldMeta; #field_count] {
                        [
                            #(lsys_core::db::FieldMeta::new(#field_name_literals, #column_name_literals)),*
                        ]
                    }

                    /// 转换为 Insert 构建器
                    pub fn to_insert<DB: sqlx::Database>(&self) -> lsys_core::db::Insert<DB, Self>
                    where
                        #(#where_bounds),*
                    {
                        lsys_core::db::Insert::<DB, Self>::new()
                            #(#insert_fields)*
                    }

                    /// 生成差异 Update（只包含变化的字段）
                    pub fn diff_update<DB: sqlx::Database>(&self, old: &Self) -> lsys_core::db::Update<DB, Self>
                    where
                        #(#where_bounds),*
                    {
                        let mut set = lsys_core::db::Update::<DB, Self>::new();
                        #(#diff_update_fields)*
                        set
                    }
                }

                impl lsys_core::db::TableMeta for #struct_name {
                    fn table_name() -> lsys_core::db::TableName {
                        lsys_core::db::TableName::new(#table_name)
                    }
                }
            }
        }
        _ => {
            panic!("#[lsys_model] only supports structs with named fields");
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
/// model 状态枚举辅助宏
pub fn lsys_model_status(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;
    let args = syn::parse_macro_input!(args with Punctuated::<Meta, Token![,]>::parse_terminated);
    let mut field_type = None;

    for cattr in args.iter() {
        if let Meta::NameValue(nv) = cattr
            && nv.path.get_ident().map(|i| i == "field_type").unwrap_or(false)
                && let Expr::Lit(expr_lit) = &nv.value
                    && let Lit::Str(lit) = &expr_lit.lit {
                        field_type = Some(lit.value());
                    }
    }
    let field_type = field_type.expect("status type not set");
    let field_type = quote::format_ident!("{}", field_type);
    let expanded = match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let fields: Vec<_> = variants
                .iter()
                .map(|field| {
                    let field_name = field.ident.clone();
                    quote! {
                        #struct_name::#field_name
                    }
                })
                .collect();
            quote! {
                #input
                lsys_core::db_model_enum_status_define!(#struct_name,#field_type,{#(#fields),*});
            }
        }
        _ => panic!("sorry, Show is not implemented for union or enum type."),
    };
    expanded.into()
}

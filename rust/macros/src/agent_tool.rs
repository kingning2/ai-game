//! `#[agent_tool]`：工具契约 struct → `definition()` + `parse()`。
//!
//! 生成物只有元数据与反序列化（机械代码）；Action 构造、合法性校验等
//! 业务逻辑由调用方手写，宏不触碰。

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Token, Type, parse2, spanned::Spanned};

/// `#[param(description = "…")]` 字段辅助属性
#[derive(Default)]
struct ParamAttr {
    description: Option<LitStr>,
}

/// 宏入参：`name = "…"`, `description = "…"`（均必填）
struct ToolAttrs {
    name: LitStr,
    description: LitStr,
}

/// 字段的 schema 片段与是否必填
struct FieldSchema {
    ident: syn::Ident,
    schema_tokens: TokenStream,
    required: bool,
}

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let tool_attrs: ToolAttrs = syn::parse2(attr)?;
    let mut input: DeriveInput = parse2(item)?;

    let struct_name = &input.ident;
    let fields = match &mut input.data {
        Data::Struct(data) => &mut data.fields,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "#[agent_tool] 只能作用于 struct（工具契约）",
            ));
        }
    };
    let named = match fields {
        Fields::Named(fields) => &mut fields.named,
        _ => {
            return Err(syn::Error::new(
                fields.span(),
                "#[agent_tool] 的 struct 必须使用命名字段（与 JSON 参数对象对应）",
            ));
        }
    };

    let tool_name = &tool_attrs.name;
    let tool_description = &tool_attrs.description;

    let mut schemas = Vec::new();
    for field in named.iter_mut() {
        schemas.push(field_schema(field)?);
        // 剔除已消费的辅助属性，避免输出中出现 rustc 不认识的 #[param]
        field.attrs.retain(|attr| !attr.path().is_ident("param"));
    }

    let properties = schemas.iter().map(|f| {
        let key = f.ident.to_string();
        let schema = &f.schema_tokens;
        quote! { #key: #schema }
    });
    let required: Vec<_> = schemas
        .iter()
        .filter(|f| f.required)
        .map(|f| f.ident.to_string())
        .collect();
    let required_part = if required.is_empty() {
        quote! {}
    } else {
        quote! { , "required": [#(#required),*] }
    };

    let expanded = quote! {
        #[derive(::serde::Deserialize)]
        #input

        impl #struct_name {
            /// 工具定义（名称 + 描述 + 参数 JSON Schema）
            pub fn definition() -> ::agent::ToolDefinition {
                ::agent::ToolDefinition::new(
                    #tool_name,
                    #tool_description,
                    ::serde_json::json!({
                        "type": "object",
                        "properties": { #(#properties),* } #required_part
                    }),
                )
            }

            /// 解析模型给出的工具参数；格式非法返回 None（由调用方兑底）
            pub fn parse(arguments: &str) -> ::core::option::Option<Self> {
                ::serde_json::from_str(arguments).ok()
            }
        }
    };
    Ok(expanded)
}

impl syn::parse::Parse for ToolAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut description = None;
        while !input.is_empty() {
            let key = input.parse::<syn::Ident>()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
            match key.to_string().as_str() {
                "name" => name = Some(value),
                "description" => description = Some(value),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("未知属性 `{other}`，支持 name / description"),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        let name = name.ok_or_else(|| syn::Error::new(input.span(), "缺少 name = \"工具名\""))?;
        let description = description
            .ok_or_else(|| syn::Error::new(input.span(), "缺少 description = \"工具描述\""))?;
        Ok(Self { name, description })
    }
}

/// 单字段 → schema 片段；`#[param(description = …)]` 提供参数描述
fn field_schema(field: &syn::Field) -> syn::Result<FieldSchema> {
    let mut param = ParamAttr::default();
    for attr in &field.attrs {
        if !attr.path().is_ident("param") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("description") {
                let value: LitStr = meta.value()?.parse()?;
                param.description = Some(value);
                Ok(())
            } else {
                Err(syn::Error::new(
                    meta.path.span(),
                    "未知 #[param] 属性，仅支持 description",
                ))
            }
        })?;
    }

    let ident = field.ident.clone().expect("命名字段必有 ident");
    let (schema_tokens, required) = type_schema(&field.ty, param.description.as_ref())?;
    Ok(FieldSchema {
        ident,
        schema_tokens,
        required,
    })
}

/// 类型 → JSON Schema 片段（支持基础类型 / Option<T> / Vec<T>，其余编译错误）
fn type_schema(ty: &Type, description: Option<&LitStr>) -> syn::Result<(TokenStream, bool)> {
    let description_part = match description {
        Some(text) => quote! { , "description": #text },
        None => quote! {},
    };

    let scalar = |json_type: &str| {
        let tokens = quote! { { "type": #json_type #description_part } };
        Ok((tokens, true))
    };

    match ty {
        Type::Path(path) if path.qself.is_none() => {
            let segments: Vec<_> = path.path.segments.iter().collect();
            let joined = segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            match joined.as_str() {
                "String" | "str" => scalar("string"),
                "bool" => scalar("boolean"),
                "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize" => {
                    scalar("integer")
                }
                "f32" | "f64" => scalar("number"),
                "Option" => {
                    let inner = last_generic(&segments)?;
                    let (inner_tokens, _) = type_schema(&inner, description)?;
                    Ok((inner_tokens, false))
                }
                "Vec" => {
                    let inner = last_generic(&segments)?;
                    let (inner_tokens, _) = type_schema(&inner, None)?;
                    let tokens =
                        quote! { { "type": "array", "items": #inner_tokens #description_part } };
                    Ok((tokens, true))
                }
                other => Err(syn::Error::new(
                    ty.span(),
                    format!(
                        "#[agent_tool] 不支持参数类型 `{other}`：仅支持 String/bool/整数/浮点 \
                         及其 Option/Vec 包装；复杂结构请拆为多个基础类型参数"
                    ),
                )),
            }
        }
        _ => Err(syn::Error::new(
            ty.span(),
            "#[agent_tool] 不支持该参数类型形式（引用/泛型别名等），仅支持基础类型路径",
        )),
    }
}

/// `Vec<T>` / `Option<T>` 的最后一个泛型实参
fn last_generic(segments: &[&syn::PathSegment]) -> syn::Result<Type> {
    let last = segments.last().expect("路径至少一段");
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return Err(syn::Error::new(
            last.ident.span(),
            "#[agent_tool] 的 Option/Vec 需要显式泛型参数，如 Option<u8>",
        ));
    };
    let arg = args
        .args
        .last()
        .ok_or_else(|| syn::Error::new(last.ident.span(), "#[agent_tool] 泛型参数为空"))?;
    match arg {
        syn::GenericArgument::Type(ty) => Ok(ty.clone()),
        _ => Err(syn::Error::new(
            arg.span(),
            "#[agent_tool] 泛型实参必须是类型（不支持生命周期/常量）",
        )),
    }
}

//! `#[game_event]`：游戏事件 enum → `day()` 元数据分发。
//!
//! 变体含 `day: u32` 字段即归入该天数，否则 `None`——
//! 新增变体不需要维护任何 match（漏标自然归 None，不会编译失败也不会 panic）。
//! serde 派生与序列化格式完全由原声明决定，宏不触碰。

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse2, spanned::Spanned};

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attr.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[game_event] 不接受参数（事件名即 serde tag=type，由 serde 派生管理）",
        ));
    }
    let input: DeriveInput = parse2(item)?;
    let enum_name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return Err(syn::Error::new(
                enum_name.span(),
                "#[game_event] 只能作用于 enum（游戏事件集合）",
            ));
        }
    };

    let mut day_arms = Vec::new();
    let mut none_arms = Vec::new();
    for variant in variants {
        let variant_ident = &variant.ident;
        match &variant.fields {
            Fields::Named(fields) => {
                let has_day = fields
                    .named
                    .iter()
                    .any(|f| f.ident.as_ref().is_some_and(|id| id == "day"));
                if has_day {
                    day_arms.push(quote! { Self::#variant_ident { day, .. } });
                } else {
                    none_arms.push(quote! { Self::#variant_ident { .. } });
                }
            }
            Fields::Unit => none_arms.push(quote! { Self::#variant_ident }),
            Fields::Unnamed(_) => {
                return Err(syn::Error::new(
                    variant.span(),
                    "#[game_event] 事件变体必须使用命名字段或无字段形式",
                ));
            }
        }
    }

    let expanded = quote! {
        #input

        impl #enum_name {
            /// 事件所属天数；无天数概念的事件返回 None。
            /// 由 #[game_event] 从变体字段推导——含 `day: u32` 字段即计入，
            /// 新增变体无需维护任何 match。需要哨兵值语义的调用方自行
            /// `day_of().unwrap_or(0)` 薄委托（见 werewolf::event）。
            pub fn day_of(&self) -> ::core::option::Option<u32> {
                match self {
                    #(#day_arms => ::core::option::Option::Some(*day),)*
                    #(#none_arms => ::core::option::Option::None,)*
                }
            }
        }
    };
    Ok(expanded)
}

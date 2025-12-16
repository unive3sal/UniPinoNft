#![no_std]
#![allow(non_snake_case)]

extern crate alloc;

mod error;
mod instructions;
mod state {
    pub mod platform_state;
}

#[cfg(feature = "bpf-entrypoint")]
mod entrypoint {
    use pinocchio::{
        entrypoint,
        account_info::AccountInfo,
        program_error::ProgramError,
        pubkey::Pubkey,
        ProgramResult
    };

    use crate::instructions::{
        platform_management::*
    };

    use pinocchio_pubkey::declare_id;

    declare_id!("6jpuWYTM3ARc5CHrMBtR1c7gyjkMTsJoYT7PqqhMpRWh");

    entrypoint!(process_instruction);

    pub fn process_instruction(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        match instruction_data.split_first() {
            Some((InitPlatform::DISCRIMINATOR, _)) => InitPlatform::try_from(accounts)?.process(),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/******* The following marco is copy from anchor repo ************/
use alloc::collections::VecDeque;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    Attribute,
    DeriveInput,
    Field,
    Fields,
};

// source code copy from anchor
#[proc_macro_derive(InitSpace, attributes(max_len))]
pub fn derive_init_space(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = input.ident;

    let process_struct_fields = |fields: Punctuated<Field, Comma>| {
        let recurse = fields.into_iter().map(|f| {
            let mut max_len_args = get_max_len_args(&f.attrs);
            len_from_type(f.ty, &mut max_len_args)
        });

        quote! {
            #[automatically_derived]
            impl #impl_generics anchor_lang::Space for #name #ty_generics #where_clause {
                const INIT_SPACE: usize = 0 #(+ #recurse)*;
            }
        }
    };

    let expanded: TokenStream2 = match input.data {
        syn::Data::Struct(strct) => match strct.fields {
            Fields::Named(named) => process_struct_fields(named.named),
            Fields::Unnamed(unnamed) => process_struct_fields(unnamed.unnamed),
            Fields::Unit => quote! {
                #[automatically_derived]
                impl #impl_generics anchor_lang::Space for #name #ty_generics #where_clause {
                    const INIT_SPACE: usize = 0;
                }
            },
        },
        syn::Data::Enum(enm) => {
            let variants = enm.variants.into_iter().map(|v| {
                let len = v.fields.into_iter().map(|f| {
                    let mut max_len_args = get_max_len_args(&f.attrs);
                    len_from_type(f.ty, &mut max_len_args)
                });

                quote! {
                    0 #(+ #len)*
                }
            });

            let max = gen_max(variants);

            quote! {
                #[automatically_derived]
                impl anchor_lang::Space for #name {
                    const INIT_SPACE: usize = 1 + #max;
                }
            }
        }
        _ => unimplemented!(),
    };

    TokenStream::from(expanded)
}

fn gen_max<T: Iterator<Item = TokenStream2>>(mut iter: T) -> TokenStream2 {
    if let Some(item) = iter.next() {
        let next_item = gen_max(iter);
        quote!(anchor_lang::__private::max(#item, #next_item))
    } else {
        quote!(0)
    }
}

fn get_max_len_args(attributes: &[Attribute]) -> Option<VecDeque<TokenStream2>> {
    attributes
        .iter()
        .find(|a| a.path.is_ident("max_len"))
        .and_then(|a| a.parse_args_with(parse_len_arg).ok())
}

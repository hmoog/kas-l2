use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{ItemImpl, Token, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn builtin(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the incoming impl block
    let impl_block = parse_macro_input!(input as ItemImpl);

    // We only support inherent impls: `impl Type { ... }`
    if impl_block.trait_.is_some() {
        return quote! {
            compile_error!("#[builtin] must be placed on an inherent impl (e.g., `impl MyType { ... }`), not a trait impl.");
        }
        .into();
    }

    let self_ty = &impl_block.self_ty; // the type on which we're impl'ing

    // Generate the declare_builtin_function! call
    let prelude = quote! {
        ::solana_sbpf::declare_builtin_function!(
            #self_ty,
            fn rust(
                context_object: &mut RuntimeContext,
                in_ptr: u64,
                in_len: u64,
                out_ptr: u64,
                out_len: u64,
                _unused: u64,
                memory_mapping: &mut MemoryMapping,
            ) -> Result<u64, Box<dyn std::error::Error>> {
                #self_ty::route(
                    context_object,
                    in_ptr,
                    in_len,
                    out_ptr,
                    out_len,
                    _unused,
                    memory_mapping
                )
            }
        );
    };

    // Re-emit the original impl unchanged (preserves generics, where-clauses, etc.)
    let expanded = quote! {
        #prelude
        #impl_block
    };

    expanded.into()
}

use quote::format_ident;
use syn::{
    Fields, ItemStruct, Result,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
};

#[derive(Default)]
struct Args {
    deref_field: Option<Ident>,
}

// Support: #[smart_pointer(deref = field)], #[smart_pointer(deref(field))]
enum Arg {
    DerefEq(Ident),
    DerefParen(Ident),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?; // expect `deref`
        if key != "deref" {
            return Err(syn::Error::new(
                key.span(),
                "unknown argument; expected `deref`",
            ));
        }
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let field: Ident = input.parse()?;
            Ok(Arg::DerefEq(field))
        } else if input.peek(syn::token::Paren) {
            let content;
            let _paren = syn::parenthesized!(content in input);
            let field: Ident = content.parse()?;
            Ok(Arg::DerefParen(field))
        } else {
            Err(syn::Error::new(
                key.span(),
                "expected `=` field_name or `(field_name)`",
            ))
        }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Args::default());
        }
        let args: Punctuated<Arg, Comma> = input.parse_terminated(Arg::parse, Comma)?;
        let mut out = Args::default();
        for arg in args {
            match arg {
                Arg::DerefEq(id) | Arg::DerefParen(id) => {
                    if out.deref_field.is_some() {
                        return Err(syn::Error::new(id.span(), "duplicate `deref` argument"));
                    }
                    out.deref_field = Some(id);
                }
            }
        }
        Ok(out)
    }
}

#[proc_macro_attribute]
pub fn smart_pointer(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let item = parse_macro_input!(input as ItemStruct);

    match expand_smart_pointer(item, args) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_smart_pointer(orig: ItemStruct, args: Args) -> Result<proc_macro2::TokenStream> {
    // Only named-field structs.
    let fields_named = match &orig.fields {
        Fields::Named(named) => named, // borrow for later lookup
        _ => {
            return Err(syn::Error::new(
                orig.span(),
                "#[smart_pointer] only supports structs with named fields",
            ));
        }
    };
    let fields_copy = fields_named.named.clone(); // for the Data struct body

    let vis_wrapper = &orig.vis;
    let ident = &orig.ident;
    let data_ident = format_ident!("{ident}Data");

    // Preserve generics
    let generics_for_header = orig.generics.clone();
    let params = generics_for_header.params.clone();
    let where_clause_hdr = generics_for_header.where_clause.clone();

    let generics_for_impl = orig.generics.clone();
    let (impl_generics, ty_generics, where_clause_impl) = generics_for_impl.split_for_impl();

    // Data struct is pub(crate)
    let vis_data: syn::Visibility = parse_quote!(pub);

    // Optional: build a Deref impl for the Data struct to a chosen field
    let data_deref_impl = if let Some(field_ident) = args.deref_field {
        // Find the field and capture its type
        let field = fields_named
            .named
            .iter()
            .find(|f| f.ident.as_ref() == Some(&field_ident))
            .ok_or_else(|| {
                syn::Error::new(
                    field_ident.span(),
                    format!("`deref` field `{}` not found on struct", field_ident),
                )
            })?;

        let ty = &field.ty;
        Some(quote! {
            impl #impl_generics ::core::ops::Deref for #data_ident #ty_generics #where_clause_impl {
                type Target = #ty;
                #[inline]
                fn deref(&self) -> &Self::Target {
                    &self.#field_ident
                }
            }
        })
    } else {
        None
    };

    let out = quote! {
        // Wrapper tuple struct with full generics/bounds
        #vis_wrapper struct #ident <#params> (
            pub(crate) ::std::sync::Arc<#data_ident #ty_generics>
        ) #where_clause_hdr ;

        // Data struct with original fields, same generics/bounds
        #vis_data struct #data_ident <#params> #where_clause_hdr {
            #fields_copy
        }

        // Deref to Data
        impl #impl_generics ::core::ops::Deref for #ident #ty_generics #where_clause_impl {
            type Target = #data_ident #ty_generics;
            #[inline]
            fn deref(&self) -> &Self::Target { &self.0 }
        }

        // Clone clones the Arc
        impl #impl_generics ::core::clone::Clone for #ident #ty_generics #where_clause_impl {
            #[inline]
            fn clone(&self) -> Self { Self(self.0.clone()) }
        }

        // Optional inner-field Deref on the Data struct
        #data_deref_impl
    };

    Ok(out)
}

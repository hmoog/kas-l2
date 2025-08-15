use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemImpl, parse_macro_input};

/// Usage:
///     #[builtin]
///     impl MyClass { /* items */ }
///
/// Expands to:
///     declare_builtin_function!(MyClass, fn rust(...) -> Result<_, _> { MyClass::route(...) });
///     impl MyClass { /* items */ }
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
                context_object: &mut ExecutionContext,
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

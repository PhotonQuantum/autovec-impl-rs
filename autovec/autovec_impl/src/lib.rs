extern crate proc_macro;

use proc_macro2::TokenStream;
use syn::{parse_macro_input, ItemFn, FnArg, ReturnType, Ident, Signature, token, parse_quote};
use quote::{quote, format_ident};
use syn::punctuated::Punctuated;

#[proc_macro_attribute]
pub fn auto_vec(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream)
                -> proc_macro::TokenStream {
    // The auto_vec macro

    let func_ast = parse_macro_input!(item as ItemFn);  // Parse tokens into ast
    let scalar_func_ast = func_ast.clone(); // Keep the original function
    let vectorized_ts = generate_fn_vector(&func_ast);  // Generate a vector wrapper function

    let result = quote! { #scalar_func_ast #vectorized_ts };
    println!("{}", result);
    result.into()
}

fn transform_return_type(x: &ReturnType) -> TokenStream {
    match x {
        ReturnType::Type(_, ty) => quote!(Vec<#ty>),
        ReturnType::Default => panic!("Expected a return type, got ().")
    }
}

fn generate_fn_vector(func_ast: &ItemFn) -> TokenStream {
    // Transform function signature to its vector version
    let (input_idents, input_args)
        = transform_input_types(&func_ast.sig.inputs);
    let output_type = transform_return_type(&func_ast.sig.output);
    let fn_vector_ident = format_ident!("{}_vec", &func_ast.sig.ident);

    build_fn_vector_wrapper(func_ast, &fn_vector_ident, input_idents, input_args, output_type)
}

fn transform_input_types(inputs: &Punctuated<FnArg, token::Comma>)
                         -> (Vec<TokenStream>, Vec<TokenStream>) {
    if inputs.is_empty() { panic!("Expected at least 1 argument, got none.") };

    inputs.iter()
        .enumerate()
        .map(|(id, arg)|
            match arg {
                FnArg::Typed(pat_ty) => {
                    let pat = &*pat_ty.pat;
                    let ty = &*pat_ty.ty;
                    match pat {
                        syn::Pat::Ident(pat_ident) => {
                            // pat_ident can't be used directly cuz it contains ref, mut, and binding properties
                            let ident = &pat_ident.ident;
                            (quote!(#ident), quote!(#ident: Vec<#ty>))
                        }
                        _ => {
                            // Something like PatTuple or PatStruct.
                            // Rewrite them into PatIdent and let the scalar func do the unpacking.
                            let ident = format_ident!("arg_{}", id);
                            (quote!(#ident), parse_quote!(#ident: Vec<#ty>))
                        }
                    }
                }
                // TODO self not considered.
                FnArg::Receiver(_) => panic!("Expected typed argument, got self.")
            }
        )
        .unzip()
}

fn build_fn_vector_wrapper(fn_scalar_ast: &ItemFn, fn_vector_ident: &Ident,
                           input_idents: Vec<TokenStream>, input_args: Vec<TokenStream>,
                           output_type: TokenStream) -> TokenStream {
    // Extract signature from the original function
    let ItemFn { attrs, vis, sig, .. } = fn_scalar_ast;
    let Signature {
        constness, unsafety, abi, ident,
        generics, variadic, ..
    }
        = sig;

    // Split input_idents for vector length check
    let (input_first_ident, input_succ_idents)
        = (&input_idents.first(), &input_idents[1..]);

    // Modified izip in the front package.
    let izip = quote! {autovec::izip};
    quote! {
        #(#attrs)* #vis #constness #unsafety #abi
        fn #fn_vector_ident #generics ( #(#input_args),* #variadic ) -> #output_type {
            // Assert vector length
            let vec_length = #input_first_ident.len();
            #(assert_eq!(vec_length, #input_succ_idents.len(),
              "Input vector length mismatch. Expected length {}, got {}.",
              vec_length, #input_succ_idents.len());)*

            #izip!(#(#input_idents),*).into_iter()
                .map(|(#(#input_idents),*)|#ident(#(#input_idents),*))
                .collect()
        }
    }
}

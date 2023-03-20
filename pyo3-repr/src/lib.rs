#![feature(let_chains)]
extern crate proc_macro;

use pm2::Ident;
use proc_macro::TokenStream;
use proc_macro2 as pm2;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed};

#[proc_macro_derive(PyDebug, attributes(py_debug))]
pub fn py_debug(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match do_expand(&ast, false) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(PyRepr, attributes(py_debug))]
pub fn py_repr(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match do_expand(&ast, true) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

type MacroResult = syn::Result<pm2::TokenStream>;

fn do_expand(ast: &DeriveInput, gen_repr: bool) -> MacroResult {
    if !ast.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            ast,
            "Generics are not supported by PyRepr.",
        ));
    }
    match &ast.data {
        Data::Struct(structure) => impl_struct_repr(ast, structure, gen_repr),
        _ => Err(syn::Error::new_spanned(
            ast,
            "Must define on a Struct".to_string(),
        )),
    }
}

fn impl_struct_repr(ast: &DeriveInput, structure: &DataStruct, gen_repr: bool) -> MacroResult {
    let fields = &structure.fields;
    match fields {
        Fields::Named(named) => Ok({
            let mut token_stream =
                gen_impl_block(&ast.ident, gen_named_impl(ast.ident.to_string(), named)?);
            if gen_repr {
                let ident = &ast.ident;
                token_stream.extend(quote!(
                    #[pymethods]
                    impl #ident {
                        fn __repr__(&self) -> String {
                            format!("{:?}", self)
                        }
                    }
                ));
            }
            token_stream
        }),
        Fields::Unnamed(_) => todo!(),
        Fields::Unit => unimplemented!(),
    }
}

fn gen_impl_block(ident: &Ident, core_stream: pm2::TokenStream) -> pm2::TokenStream {
    quote!(
        impl ::std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::pyo3::marker::Python::with_gil(|py| {
                    #core_stream
                })
            }
        }
    )
}

fn is_py_ptr(ty: &syn::Type) -> bool {
    if let syn::Type::Path(pth) = ty {
        if pth
            .path
            .segments
            .iter()
            .any(|seg| seg.ident == "Py" || seg.ident == "PyObject")
        {
            return true;
        }
    }
    false
}

fn gen_named_impl(ident: String, fields: &FieldsNamed) -> MacroResult {
    let mut core_stream = pm2::TokenStream::new();
    core_stream.extend(quote!(
        f.debug_struct(#ident)
    ));
    for f in fields.named.iter() {
        let field_name_ident = f.ident.as_ref().unwrap();
        let field_name_literal = field_name_ident.to_string();
        let mut py_convert = is_py_ptr(&f.ty);
        for attr in f.attrs.iter() {
            attr.parse_nested_meta(|meta| {
                let ident = meta.path.get_ident().ok_or_else(|| {
                    syn::Error::new_spanned(
                        meta.path.clone(),
                        "py_repr only supports bare ident as arg.",
                    )
                })?;
                match ident.to_string().as_str() {
                    "skip" => return Ok(()),
                    "py" => {
                        py_convert = true;
                    }
                    "debug" => {
                        py_convert = false;
                    }
                    _ => return Err(syn::Error::new_spanned(ident, "Unexpected option")),
                }
                Ok(())
            })?;
        }
        if py_convert {
            core_stream.extend(quote!(
                .field(#field_name_literal, self.#field_name_ident.as_ref(py))
            ));
        } else {
            core_stream.extend(quote!(
                .field(#field_name_literal, &self.#field_name_ident)
            ));
        }
    }
    core_stream.extend(quote!(
        .finish()
    ));
    Ok(core_stream)
}

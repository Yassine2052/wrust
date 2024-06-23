extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, Type};

// Thanks, Mr. ChatGpt for making this clean code 😚😚😚
#[proc_macro_derive(InjectStruct)]
pub fn inject_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the Annotated Type Name
    let name = input.ident;

    // Extract Annotated Type Fields and Panic if it is not a Struct
    let fields = match input.data {
        syn::Data::Struct(data_struct) => data_struct.fields,
        _ => panic!("InjectStruct can only be derived for structs"),
    };

    // Init Struct with Default Values
    let initializations = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Expected a named field");
        let field_ty = &field.ty;

        quote! {
                #field_name: <#field_ty as Default>::default()
            }
    }).collect::<Vec<_>>();

    // Init Fields From Hashmap - If something went wrong the server will be down 💥💥💥
    let hashmap_initializations = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Expected a named field");
        let name_str = field_name.to_string();
        let field_ty = &field.ty;

        match field_ty {
            Type::Path(type_path) if type_path.path.is_ident("String") => {
                quote! {
                    #field_name: hashmap.get(#name_str)
                        .and_then(|param| {
                            if let shared::query::QueryParamValue::Single(shared::query::QueryParamValueType::Str(ref v)) = param.value {
                                Some(v.clone())
                            } else {
                                None
                            }
                        })
                        .expect(&format!("Missing or invalid type for field: {}", #name_str))
                }
            },
            Type::Path(type_path) if type_path.path.is_ident("isize") => {
                quote! {
                    #field_name: hashmap.get(#name_str)
                        .and_then(|param| {
                            if let shared::query::QueryParamValue::Single(shared::query::QueryParamValueType::Int(ref v)) = param.value {
                                Some(*v)
                            } else {
                                None
                            }
                        })
                        .expect(&format!("Missing or invalid type for field: {}", #name_str))
                }
            },
            Type::Path(type_path) if type_path.path.is_ident("usize") => {
                quote! {
                    #field_name: hashmap.get(#name_str)
                        .and_then(|param| {
                            if let shared::query::QueryParamValue::Single(shared::query::QueryParamValueType::UInt(ref v)) = param.value {
                                Some(*v)
                            } else {
                                None
                            }
                        })
                        .expect(&format!("Missing or invalid type for field: {}", #name_str))
                }
            },
            Type::Path(type_path) if type_path.path.is_ident("f64") => {
                quote! {
                    #field_name: hashmap.get(#name_str)
                        .and_then(|param| {
                            if let shared::query::QueryParamValue::Single(shared::query::QueryParamValueType::Float(ref v)) = param.value {
                                Some(*v)
                            } else {
                                None
                            }
                        })
                        .expect(&format!("Missing or invalid type for field: {}", #name_str))
                }
            },
            Type::Path(type_path) if type_path.path.is_ident("bool") => {
                quote! {
                    #field_name: hashmap.get(#name_str)
                        .and_then(|param| {
                            if let shared::query::QueryParamValue::Single(shared::query::QueryParamValueType::Boolean(ref v)) = param.value {
                                Some(*v)
                            } else {
                                None
                            }
                        })
                        .expect(&format!("Missing or invalid type for field: {}", #name_str))
                }
            },
            Type::Path(type_path) if type_path.path.is_ident("Vec") => {
                let inner_ty = if let syn::PathArguments::AngleBracketed(args) = &type_path.path.segments[0].arguments {
                    if let syn::GenericArgument::Type(inner_ty) = &args.args[0] {
                        inner_ty
                    } else {
                        panic!("Unsupported field type for field: {}", name_str);
                    }
                } else {
                    panic!("Unsupported field type for field: {}", name_str);
                };

                match inner_ty {
                    Type::Path(inner_type_path) if inner_type_path.path.is_ident("String") => {
                        quote! {
                            #field_name: hashmap.get(#name_str)
                                .and_then(|param| {
                                    if let shared::query::QueryParamValue::Multiple(ref values) = param.value {
                                        let mut vec = Vec::new();
                                        for value in values {
                                            if let shared::query::QueryParamValueType::Str(ref v) = value {
                                                vec.push(v.clone());
                                            }
                                        }
                                        Some(vec)
                                    } else {
                                        None
                                    }
                                })
                                .expect(&format!("Missing or invalid type for field: {}", #name_str))
                        }
                    },
                    Type::Path(inner_type_path) if inner_type_path.path.is_ident("isize") => {
                        quote! {
                            #field_name: hashmap.get(#name_str)
                                .and_then(|param| {
                                    if let shared::query::QueryParamValue::Multiple(ref values) = param.value {
                                        let mut vec = Vec::new();
                                        for value in values {
                                            if let shared::query::QueryParamValueType::Int(ref v) = value {
                                                vec.push(*v);
                                            }
                                        }
                                        Some(vec)
                                    } else {
                                        None
                                    }
                                })
                                .expect(&format!("Missing or invalid type for field: {}", #name_str))
                        }
                    },
                    Type::Path(inner_type_path) if inner_type_path.path.is_ident("usize") => {
                        quote! {
                            #field_name: hashmap.get(#name_str)
                                .and_then(|param| {
                                    if let shared::query::QueryParamValue::Multiple(ref values) = param.value {
                                        let mut vec = Vec::new();
                                        for value in values {
                                            if let shared::query::QueryParamValueType::UInt(ref v) = value {
                                                vec.push(*v);
                                            }
                                        }
                                        Some(vec)
                                    } else {
                                        None
                                    }
                                })
                                .expect(&format!("Missing or invalid type for field: {}", #name_str))
                        }
                    },
                    Type::Path(inner_type_path) if inner_type_path.path.is_ident("f64") => {
                        quote! {
                            #field_name: hashmap.get(#name_str)
                                .and_then(|param| {
                                    if let shared::query::QueryParamValue::Multiple(ref values) = param.value {
                                        let mut vec = Vec::new();
                                        for value in values {
                                            if let shared::query::QueryParamValueType::Float(ref v) = value {
                                                vec.push(*v);
                                            }
                                        }
                                        Some(vec)
                                    } else {
                                        None
                                    }
                                })
                                .expect(&format!("Missing or invalid type for field: {}", #name_str))
                        }
                    },
                    Type::Path(inner_type_path) if inner_type_path.path.is_ident("bool") => {
                        quote! {
                            #field_name: hashmap.get(#name_str)
                                .and_then(|param| {
                                    if let shared::query::QueryParamValue::Multiple(ref values) = param.value {
                                        let mut vec = Vec::new();
                                        for value in values {
                                            if let shared::query::QueryParamValueType::Boolean(ref v) = value {
                                                vec.push(*v);
                                            }
                                        }
                                        Some(vec)
                                    } else {
                                        None
                                    }
                                })
                                .expect(&format!("Missing or invalid type for field: {}", #name_str))
                        }
                    },
                    _ => {
                        quote! {
                            panic!("Unsupported inner type for Vec in field: {}", #name_str)
                        }
                    },
                }
            },
            _ => quote! {
                panic!("Unsupported field type for field: {}", #name_str)
            },
        }
    });

    // Generate the `from_hashmap` function implementation
    let implementation = quote! {
        impl shared::wrust_traits::InjectStructTrait for #name {
            fn init() -> Self {
                Self {
                    #(#initializations),*
                }
            }
            fn from_hashmap(hashmap: &shared::request::RequestQueriesHashMap) -> Self {
                Self {
                    #(#hashmap_initializations),*
                }
            }
        }
    };

    TokenStream::from(implementation)
}
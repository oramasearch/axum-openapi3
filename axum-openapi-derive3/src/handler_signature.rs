use syn::{spanned::Spanned, FnArg, Signature};

use crate::util::recursive_type;

#[derive(Debug)]
pub enum HandlerArgument {
    RequestBody(String),
    Query(String),
    State(String),
    Path(String),
}

pub fn parse_handler_arguments(sig: &Signature) -> Result<Vec<HandlerArgument>, syn::Error> {
    let mut ret = vec![];
    for input in &sig.inputs {
        match input {
            FnArg::Typed(pat_type) => {
                let ty = &pat_type.ty;

                match ty.as_ref() {
                    syn::Type::Path(path) => {
                        let mut v = vec![];
                        recursive_type(path, &mut v);

                        let extractor = v.remove(0);

                        match extractor.as_str() {
                            "Json" => {
                                let l = v.len();

                                ret.push(HandlerArgument::RequestBody(
                                    v.join("<") + ">".repeat(l - 1).as_str(),
                                ));
                            }
                            "Query" => {
                                let l = v.len();

                                ret.push(HandlerArgument::Query(
                                    v.join("<") + ">".repeat(l - 1).as_str(),
                                ));
                            }
                            "State" => {
                                let l = v.len();

                                ret.push(HandlerArgument::State(
                                    v.join("<") + ">".repeat(l - 1).as_str(),
                                ));
                            }
                            "Path" => {
                                let l = v.len();

                                ret.push(HandlerArgument::Path(
                                    v.join("<") + ">".repeat(l - 1).as_str(),
                                ));
                            }
                            _ => continue,
                        };
                    }
                    _ => {
                        return Err(syn::Error::new(ty.span(), "Expected Type::Path type"));
                    }
                }
            }
            _ => {
                return Err(syn::Error::new(
                    input.span(),
                    "Self argument is unsupported",
                ));
            }
        };
    }

    Ok(ret)
}

pub fn parse_handler_ret_type(sig: &Signature) -> Result<Option<String>, syn::Error> {
    match &sig.output {
        syn::ReturnType::Default => {
            Err(syn::Error::new(sig.output.span(), "Expected return type"))
        }
        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            syn::Type::Path(path) => {
                let mut v = vec![];
                recursive_type(path, &mut v);

                if v.remove(0) != "Json" {
                    Ok(None)
                } else {
                    let l = v.len();
                    Ok(Some(v.join("<") + ">".repeat(l - 1).as_str()))
                }
            }
            _ => Ok(None),
        },
    }
}

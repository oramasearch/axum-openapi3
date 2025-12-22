use syn::{parse::Parse, spanned::Spanned, Expr, Lit, MetaNameValue, Token};

#[derive(Debug)]
pub struct MacroArgs {
    pub method: http::Method,
    pub path: String,
    pub description: Option<String>,
}
impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut method = None;
        let mut path = None;
        let mut description = None;

        while !input.is_empty() {
            // Parse key-value pairs
            let meta: MetaNameValue = input.parse()?;

            if meta.path.is_ident("method") {
                method = match meta.value {
                    Expr::Lit(s) => match s.lit {
                        Lit::Str(lit) => Some(lit.value()),
                        _ => {
                            return Err(syn::Error::new(
                                meta.path.span(),
                                "Expected literal string",
                            ))
                        }
                    },
                    _ => return Err(syn::Error::new(meta.path.span(), "Expected literal")),
                };
            } else if meta.path.is_ident("path") {
                path = match meta.value {
                    Expr::Lit(s) => match s.lit {
                        Lit::Str(lit) => Some(lit.value()),
                        _ => {
                            return Err(syn::Error::new(
                                meta.path.span(),
                                "Expected literal string",
                            ))
                        }
                    },
                    _ => return Err(syn::Error::new(meta.path.span(), "Expected literal")),
                };
            } else if meta.path.is_ident("description") {
                description = match meta.value {
                    Expr::Lit(s) => match s.lit {
                        Lit::Str(lit) => Some(lit.value()),
                        _ => {
                            return Err(syn::Error::new(
                                meta.path.span(),
                                "Expected literal string",
                            ))
                        }
                    },
                    _ => return Err(syn::Error::new(meta.path.span(), "Expected literal")),
                };
            } else {
                return Err(syn::Error::new(meta.path.span(), "Unexpected argument"));
            }

            // Consume optional commas between arguments
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        // Ensure both `method` and `path` are provided
        let method =
            method.ok_or_else(|| syn::Error::new(input.span(), "Missing `method` argument"))?;
        let method = method.to_uppercase();
        let method: http::Method = method
            .parse()
            .map_err(|_| syn::Error::new(input.span(), "Invalid HTTP method"))?;

        let path = path.ok_or_else(|| syn::Error::new(input.span(), "Missing `path` argument"))?;

        Ok(MacroArgs {
            method,
            path,
            description,
        })
    }
}

use syn::TypePath;

pub fn recursive_type(type_path: &TypePath, v: &mut Vec<String>) {
    v.push(type_path.path.segments[0].ident.to_string());
    if let syn::PathArguments::AngleBracketed(ref args) = type_path.path.segments[0].arguments {
        for arg in args.args.iter() {
            match arg {
                syn::GenericArgument::Type(ty) => {
                    match ty {
                        syn::Type::Path(path) => {
                            recursive_type(path, v);
                        }
                        syn::Type::Reference(r) => {
                            match r.elem.as_ref() {
                                syn::Type::Path(path) => {
                                    recursive_type(path, v);
                                }
                                _ => {
                                    // eprintln!("Unknown type: {:?}", a);
                                }
                            }
                        }
                        _ => {
                            // println!("Unknown type: {:?}", a);
                        }
                    }
                }
                _ => {
                    // println!("Unknown type: {:?}", a);
                }
            }
        }
    }
}

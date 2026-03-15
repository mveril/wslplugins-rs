use syn::{
    GenericArgument, Lifetime, Path, PathArguments, Type, TypeBareFn, TypePath, TypeReference,
    TypeTuple,
};

pub(super) fn replace_lifetime_in_type(
    ty: &mut Type,
    old_lifetime: &Lifetime,
    new_lifetime: &Lifetime,
) {
    match ty {
        Type::Reference(TypeReference {
            ref mut lifetime, ..
        }) => {
            if let Some(ref current_lifetime) = lifetime {
                if current_lifetime == old_lifetime {
                    *lifetime = Some(new_lifetime.clone());
                }
            }
        }
        Type::Path(TypePath { ref mut path, .. }) => {
            for segment in &mut path.segments {
                if let PathArguments::AngleBracketed(ref mut args) = segment.arguments {
                    for arg in &mut args.args {
                        if let GenericArgument::Lifetime(ref mut lifetime) = arg {
                            if lifetime == old_lifetime {
                                *lifetime = new_lifetime.clone();
                            }
                        } else if let GenericArgument::Type(ref mut inner_type) = arg {
                            replace_lifetime_in_type(inner_type, old_lifetime, new_lifetime);
                        }
                    }
                }
            }
        }
        Type::BareFn(TypeBareFn {
            ref mut inputs,
            ref mut output,
            ..
        }) => {
            for input in inputs {
                replace_lifetime_in_type(&mut input.ty, old_lifetime, new_lifetime);
            }
            if let syn::ReturnType::Type(_, ref mut output_type) = output {
                replace_lifetime_in_type(output_type, old_lifetime, new_lifetime);
            }
        }
        Type::Tuple(TypeTuple { ref mut elems, .. }) => {
            for elem in elems {
                replace_lifetime_in_type(elem, old_lifetime, new_lifetime);
            }
        }
        _ => {}
    }
}

pub(super) fn get_path_lifetime(path: &Path) -> Option<&Lifetime> {
    for segment in &path.segments {
        if let PathArguments::AngleBracketed(ref args) = segment.arguments {
            for arg in &args.args {
                if let GenericArgument::Lifetime(ref lifetime) = arg {
                    return Some(lifetime);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::{parse_quote, Type};

    #[test]
    fn test_replace_lifetime_in_reference_type() {
        let mut ty: Type = parse_quote!(&'a i32);
        let old_lifetime: Lifetime = parse_quote!('a);
        let new_lifetime: Lifetime = parse_quote!('b);

        replace_lifetime_in_type(&mut ty, &old_lifetime, &new_lifetime);

        let expected_ty: Type = parse_quote!(&'b i32);
        assert_eq!(ty, expected_ty);
    }

    #[test]
    fn test_replace_lifetime_in_path_type() {
        let mut ty: Type = parse_quote!(MyStruct<'a>);
        let old_lifetime: Lifetime = parse_quote!('a);
        let new_lifetime: Lifetime = parse_quote!('b);

        replace_lifetime_in_type(&mut ty, &old_lifetime, &new_lifetime);

        let expected_ty: Type = parse_quote!(MyStruct<'b>);
        assert_eq!(
            ty.to_token_stream().to_string(),
            expected_ty.to_token_stream().to_string()
        );
    }

    #[test]
    fn test_replace_lifetime_in_tuple_type() {
        let mut ty: Type = parse_quote!((&'a i32, &'a str));
        let old_lifetime: Lifetime = parse_quote!('a);
        let new_lifetime: Lifetime = parse_quote!('b);

        replace_lifetime_in_type(&mut ty, &old_lifetime, &new_lifetime);

        let expected_ty: Type = parse_quote!((&'b i32, &'b str));
        assert_eq!(ty, expected_ty);
    }

    #[test]
    fn test_get_path_lifetime() {
        let path: Path = parse_quote!(MyStruct<'a>);
        let expected_lifetime: Lifetime = parse_quote!('a);

        let lifetime = get_path_lifetime(&path);

        assert_eq!(lifetime, Some(&expected_lifetime));
    }

    #[test]
    fn test_get_path_lifetime_no_lifetime() {
        let path: Path = parse_quote!(MyStruct);

        let lifetime = get_path_lifetime(&path);

        assert_eq!(lifetime, None);
    }
}

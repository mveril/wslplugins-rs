use syn::parse::{Parse, ParseStream};
use syn::LitInt;
use syn::{Result, Token};

use crate::acc_syn_result;

#[derive(Debug)]

pub struct RequiredVersion {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
}
impl Parse for RequiredVersion {
    fn parse(input: ParseStream) -> Result<Self> {
        // Result of parsing the major version to u32
        let major_lit = input.parse::<LitInt>()?;
        // Result of parsing the coma version to u32
        _ = input.parse::<Token![,]>()?;
        // Result of parsing the minor version to u32
        let minor_lit = input.parse::<LitInt>()?;
        // Parse the revision if it exists
        let revision_lit = if input.peek(Token![,]) {
            input.parse::<Token![,]>().unwrap();
            Some(input.parse::<LitInt>()?)
        } else {
            None
        };
        if input.is_empty() {
            Ok(())
        } else {
            Err(input.error("unexpected additional components in version"))
        }?;
        let major_result = major_lit.base10_parse::<u32>();
        let minor_result = minor_lit.base10_parse::<u32>();
        let revision_result = revision_lit
            .map(|lit| lit.base10_parse::<u32>())
            .unwrap_or(Ok(0));
        acc_syn_result!(major_result, minor_result, revision_result).map(
            |(major, minor, revision)| RequiredVersion {
                major,
                minor,
                revision,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn test_parse_valid_version_with_revision() {
        let version_tokens = quote! { 1, 2, 3 };
        let parsed_version: RequiredVersion = parse2(version_tokens).unwrap();

        assert_eq!(parsed_version.major, 1);
        assert_eq!(parsed_version.minor, 2);
        assert_eq!(parsed_version.revision, 3);
    }

    #[test]
    fn test_parse_valid_version_without_revision() {
        let version_tokens = quote! { 1, 2 };
        let parsed_version: RequiredVersion = parse2(version_tokens).unwrap();

        assert_eq!(parsed_version.major, 1);
        assert_eq!(parsed_version.minor, 2);
        assert_eq!(parsed_version.revision, 0);
    }

    #[test]
    fn test_parse_invalid_version_missing_major() {
        let version_tokens = quote! { , 2, 3 }; // Missing major part
        let parsed_result = parse2::<RequiredVersion>(version_tokens);

        assert!(parsed_result.is_err());
    }

    #[test]
    fn test_parse_invalid_version_non_numeric() {
        let version_tokens = quote! { 1, a, 3 }; // Non-numeric minor version
        let parsed_result = parse2::<RequiredVersion>(version_tokens);

        assert!(parsed_result.is_err());
    }

    #[test]
    fn test_parse_invalid_version_extra_component() {
        let version_tokens = quote! { 1, 2, 3, 4 }; // Extra component
        let parsed_result = parse2::<RequiredVersion>(version_tokens);

        assert!(parsed_result.is_err());
        assert_eq!(
            parsed_result.unwrap_err().to_string(),
            "unexpected additional components in version"
        );
    }

    #[test]
    fn test_parse_invalid_version_extra_trailing_comma() {
        let version_tokens = quote! { 1, 2, 3, }; // Trailing comma
        let parsed_result = parse2::<RequiredVersion>(version_tokens);

        assert!(parsed_result.is_err());
        assert_eq!(
            parsed_result.unwrap_err().to_string(),
            "unexpected additional components in version"
        );
    }
}

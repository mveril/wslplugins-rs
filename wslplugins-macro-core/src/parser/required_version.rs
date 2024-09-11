use std::result;

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
        // Define the function to get the u32
        let base_10_parse = |lit: LitInt| lit.base10_parse::<u32>();
        // Result of parsing the major version to u32
        let major_result = input.parse::<LitInt>().and_then(base_10_parse);
        // Result of parsing the coma version to u32
        let comma_result = input.parse::<Token![,]>();
        // Result of parsing the minor version to u32
        let minor_result = input.parse::<LitInt>().and_then(base_10_parse);

        // Parse the revision if it exists, otherwise default to 0
        let revision_result = if input.peek(Token![,]) {
            input.parse::<Token![,]>().unwrap();
            input.parse::<LitInt>().and_then(base_10_parse)
        } else {
            Ok(0)
        };
        let no_extra_result: Result<()> = if input.is_empty() {
            Ok(())
        } else {
            Err(input.error("unexpected additional components in version"))
        };

        let (major, minor, revision, _, _) = acc_syn_result!(
            major_result,
            minor_result,
            revision_result,
            comma_result,
            no_extra_result
        )?;
        Ok(RequiredVersion {
            major: major,
            minor: minor,
            revision: revision,
        })
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

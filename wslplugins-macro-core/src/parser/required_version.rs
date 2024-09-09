use syn::parse::{Parse, ParseStream};
use syn::LitInt;
use syn::{Result, Token};

#[derive(Debug)]

pub struct RequiredVersion {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
}
impl Parse for RequiredVersion {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse the major version
        let major: LitInt = input.parse()?;
        // Parse the comma separator
        input.parse::<Token![,]>()?;
        // Parse the minor version
        let minor: LitInt = input.parse()?;

        // Parse the revision if it exists, otherwise default to 0
        let revision = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            input.parse::<LitInt>()?.base10_parse()?
        } else {
            0
        };

        // Ensure there are no extra tokens after the revision
        if !input.is_empty() {
            return Err(input.error("unexpected additional components in version"));
        }

        Ok(RequiredVersion {
            major: major.base10_parse()?,
            minor: minor.base10_parse()?,
            revision,
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

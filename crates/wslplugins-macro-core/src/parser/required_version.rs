use syn::parse::{Parse, ParseStream};
use syn::{ExprPath, LitInt};
use syn::{Result, Token};

use crate::acc_syn_result;

#[derive(Debug)]
pub enum RequiredVersion {
    Version {
        major: u32,
        minor: u32,
        revision: u32,
    },
    Capabilities(Vec<ExprPath>),
}

impl Default for RequiredVersion {
    fn default() -> Self {
        Self::Version {
            major: 0,
            minor: 0,
            revision: 0,
        }
    }
}

impl Parse for RequiredVersion {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        if !input.peek(LitInt) {
            let mut capabilities = vec![input.parse::<ExprPath>()?];
            while input.peek(Token![|]) {
                input.parse::<Token![|]>()?;
                capabilities.push(input.parse::<ExprPath>()?);
            }
            if input.is_empty() {
                return Ok(Self::Capabilities(capabilities));
            }
            return Err(input.error("unexpected additional tokens after capabilities"));
        }

        // Result of parsing the major version to u32
        let major_lit = input.parse::<LitInt>()?;
        // Result of parsing the coma version to u32
        _ = input.parse::<Token![,]>()?;
        // Result of parsing the minor version to u32
        let minor_lit = input.parse::<LitInt>()?;
        // Parse the revision if it exists
        let revision_lit = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
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
        let revision_result = revision_lit.map_or(Ok(0), |lit| lit.base10_parse::<u32>());
        acc_syn_result!(major_result, minor_result, revision_result).map(
            |(major, minor, revision)| Self::Version {
                major,
                minor,
                revision,
            },
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "Test code")]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    #[allow(clippy::panick)]
    fn test_parse_valid_version_with_revision() {
        let version_tokens = quote! { 1, 2, 3 };
        let parsed_version: RequiredVersion = parse2(version_tokens).unwrap();
        match parsed_version {
            RequiredVersion::Version {
                major,
                minor,
                revision,
            } => {
                assert_eq!(major, 1);
                assert_eq!(minor, 2);
                assert_eq!(revision, 3);
            }
            RequiredVersion::Capabilities(_) => panic!("expected explicit version"),
        }
    }

    #[test]
    fn test_parse_valid_version_without_revision() {
        let version_tokens = quote! { 1, 2 };
        let parsed_version: RequiredVersion = parse2(version_tokens).unwrap();

        match parsed_version {
            RequiredVersion::Version {
                major,
                minor,
                revision,
            } => {
                assert_eq!(major, 1);
                assert_eq!(minor, 2);
                assert_eq!(revision, 0);
            }
            RequiredVersion::Capabilities(_) => panic!("expected explicit version"),
        }
    }

    #[test]
    fn test_parse_empty_version_as_minimum_version() {
        let version_tokens = quote! {};
        let parsed_version: RequiredVersion = parse2(version_tokens).unwrap();

        match parsed_version {
            RequiredVersion::Version {
                major,
                minor,
                revision,
            } => {
                assert_eq!(major, 0);
                assert_eq!(minor, 0);
                assert_eq!(revision, 0);
            }
            RequiredVersion::Capabilities(_) => panic!("expected minimum version"),
        }
    }

    #[test]
    fn test_parse_valid_capability() {
        let version_tokens =
            quote! { ::wslplugins_rs::WSLVersionCapability::DistributionRegisteredHook };
        let parsed_version: RequiredVersion = parse2(version_tokens).unwrap();

        match parsed_version {
            RequiredVersion::Version { .. } => panic!("expected capability"),
            RequiredVersion::Capabilities(capabilities) => {
                assert_eq!(
                    quote!(#(#capabilities)|*).to_string(),
                    quote!(::wslplugins_rs::WSLVersionCapability::DistributionRegisteredHook)
                        .to_string()
                );
            }
        }
    }

    #[test]
    fn test_parse_valid_capability_set() {
        let version_tokens = quote! {
            ::wslplugins_rs::WSLVersionCapability::DistributionRegisteredHook
            | ::wslplugins_rs::WSLVersionCapability::DistributionUnregisteredHook
        };
        let parsed_version: RequiredVersion = parse2(version_tokens).unwrap();

        match parsed_version {
            RequiredVersion::Version { .. } => panic!("expected capability set"),
            RequiredVersion::Capabilities(capabilities) => {
                assert_eq!(capabilities.len(), 2);
                assert_eq!(
                    quote!(#(#capabilities)|*).to_string(),
                    quote!(
                        ::wslplugins_rs::WSLVersionCapability::DistributionRegisteredHook
                            | ::wslplugins_rs::WSLVersionCapability::DistributionUnregisteredHook
                    )
                    .to_string()
                );
            }
        }
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

    proptest! {
        #[test]
        fn parse_valid_version_with_generated_components(
            major in any::<u32>(),
            minor in any::<u32>(),
            revision in any::<u32>(),
        ) {
            let version_tokens = quote! { #major, #minor, #revision };
            let parsed_version: RequiredVersion = parse2(version_tokens)?;

            match parsed_version {
                RequiredVersion::Version {
                    major: parsed_major,
                    minor: parsed_minor,
                    revision: parsed_revision,
                } => {
                    prop_assert_eq!(parsed_major, major);
                    prop_assert_eq!(parsed_minor, minor);
                    prop_assert_eq!(parsed_revision, revision);
                }
                RequiredVersion::Capabilities(_) => {
                    prop_assert!(false, "expected explicit version");
                }
            }
        }

        #[test]
        fn parse_valid_version_without_generated_revision(
            major in any::<u32>(),
            minor in any::<u32>(),
        ) {
            let version_tokens = quote! { #major, #minor };
            let parsed_version: RequiredVersion = parse2(version_tokens)?;

            match parsed_version {
                RequiredVersion::Version {
                    major: parsed_major,
                    minor: parsed_minor,
                    revision,
                } => {
                    prop_assert_eq!(parsed_major, major);
                    prop_assert_eq!(parsed_minor, minor);
                    prop_assert_eq!(revision, 0);
                }
                RequiredVersion::Capabilities(_) => {
                    prop_assert!(false, "expected explicit version");
                }
            }
        }
    }
}

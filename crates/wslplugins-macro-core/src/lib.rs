#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::panic)]
#![allow(clippy::panic_in_result_fn)]
//! Core implementation for the WSL plugin procedural macros.
mod generator;
mod hooks;
mod parser;
mod utils;

use generator::generate;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Result};

use crate::parser::{ParsedImpl, RequiredVersion};
#[inline]
pub fn wsl_plugin_v1(attr: TokenStream, item: &TokenStream) -> Result<TokenStream> {
    let parsed_impl_result = parse2::<ParsedImpl>(item.clone());
    let required_version_result = parse2::<RequiredVersion>(attr);
    let (parsed_impl, required_version) =
        acc_syn_result!(parsed_impl_result, required_version_result,)?;
    let generated_tokens = generate(&parsed_impl, &required_version)?;

    Ok(quote! {
        #item
        #generated_tokens
    })
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::wsl_plugin_v1;

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "Test contains long example implementation"
    )]
    fn test_wsl_plugin_v1() {
        let attr = quote! {1,0,5};
        let item = quote! {
            impl WSLPluginV1 for Plugin {
                fn try_new() -> Result<Self> {
                    setup_logging()?;
                    let plugin = Plugin {};
                    Ok(plugin)
                }

                fn on_vm_started(
                    &self,
                    session: &WSLSessionInformation,
                    user_settings: &WSLVmCreationSettings,
                ) -> Result<()> {
                    info!(
                        "User configuration {:?}",
                        user_settings.custom_configuration_flags()
                    );

                    let ver_args = ["/bin/cat", "/proc/version"];
                    match WSLContext::get_current().api.execute_binary(session, ver_args[0], &ver_args) {
                        Ok(mut stream) => {
                            let mut buf = String::new();
                            if stream.read_to_string(&mut buf).is_ok_and(|size| size != 0) {
                                info!("Kernel version info: {}", buf.trim());
                            } else {
                                warn!("No version found");
                            }
                        }
                        Err(err) => {
                            warn!(
                                "Error on binary execution {}: {}",
                                stringify!(on_vm_started),
                                err
                            )
                        }
                    };
                    let ver_args = ["/bin/cat", "/proc/version"];
                    match WSLContext::get_current().api.execute_binary(session, ver_args[0], &ver_args) {
                        Ok(mut stream) => {
                            let mut buf = String::new();
                            if stream.read_to_string(&mut buf).is_ok_and(|size| size != 0) {
                                info!("Kernel version info: {}", buf.trim());
                            } else {
                                warn!("No version found");
                            }
                        }
                        Err(err) => {
                            warn!(
                                "Error on binary execution {}: {}",
                                stringify!(on_vm_started),
                                err
                            )
                        }
                    };
                    self.log_os_release(session, None);
                    Ok(())
                }

                fn on_distribution_started(
                    &self,
                    session: &WSLSessionInformation,
                    distribution: &DistributionInformation,
                ) -> Result<()> {
                    info!(
                        "Distribution started. Sessionid= {:}, Id={:?} Name={:}, Package={}, PidNs={}, InitPid={}",
                        session.id(),
                        distribution.id(),
                        distribution.name().to_string_lossy(),
                        distribution.package_family_name().unwrap_or_default().to_string_lossy(),
                        distribution.pid_namespace(),
                        distribution.init_pid().unwrap()
                    );
                    self.log_os_release(session, Some(distribution.id()));
                    Ok(())
                }

                fn on_vm_stopping(&self, session: &WSLSessionInformation) -> Result<()> {
                    info!("VM Stopping. SessionId={:?}", session.id());
                    Ok(())
                }

                fn on_distribution_stopping(
                    &self,
                    session: &WSLSessionInformation,
                    distribution: &DistributionInformation,
                ) -> Result<()> {
                    info!(
                        "Distribution Stopping. SessionId={}, Id={:?} name={}, package={}, PidNs={}, InitPid={}",
                        session.id(),
                        distribution.id(),
                        distribution.name().to_string_lossy(),
                        distribution.package_family_name().unwrap_or_default().to_string_lossy(),
                        distribution.pid_namespace(),
                        distribution.init_pid().unwrap()
                    );
                    Ok(())
                }
            }
        };
        let result = wsl_plugin_v1(attr, &item);
        assert!(result.is_ok());
    }
}

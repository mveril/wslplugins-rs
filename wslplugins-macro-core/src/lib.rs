pub(crate) mod generator;
pub(crate) mod hooks;
pub(crate) mod parser;

use generator::generate;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Result};

use crate::parser::{ParsedImpl, RequiredVersion};

pub fn wsl_plugin_v1(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Clonage pour éviter le déplacement
    let parsed_impl = parse2::<ParsedImpl>(item.clone())?;
    let required_version = parse2::<RequiredVersion>(attr.clone())?;
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
    fn test_wsl_plugin_v1() {
        let attr = quote! {1,0,5};
        let item = quote! {
            impl WSLPluginV1<'a> for Plugin<'a> {
                fn try_new(api: ApiV1<'a>) -> Result<Self> {
                    setup_logging()?;
                    let plugin = Plugin { api };
                    info!("Plugin created");
                    Ok(plugin)
                }

                #[instrument]
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
                    match self.api.execute_binary(session, &ver_args[0], &ver_args) {
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
                    match self.api.execute_binary(session, &ver_args[0], &ver_args) {
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

                #[instrument]
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
                        distribution.init_pid()
                    );
                    self.log_os_release(session, Some(distribution.id()));
                    Ok(())
                }

                #[instrument]
                fn on_vm_stopping(&self, session: &WSLSessionInformation) -> Result<()> {
                    info!("VM Stopping. SessionId={:?}", session.id());
                    Ok(())
                }

                #[instrument]
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
                        distribution.init_pid()
                    );
                    Ok(())
                }
            }
        };
        let result = wsl_plugin_v1(attr, item);
        assert!(result.is_ok());
        eprint!("{}", result.unwrap())
    }
}

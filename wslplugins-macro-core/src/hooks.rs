use heck::ToSnakeCase;
use strum::IntoEnumIterator;
include!(concat!(env!("OUT_DIR"), "/hooks.rs"));

impl Hooks {
    pub fn get_c_method_name(&self) -> String {
        self.to_string().to_snake_case()
    }

    pub fn get_hook_field_name(&self) -> String {
        self.to_string()
    }

    pub fn get_trait_method_name(&self) -> String {
        self.to_string().to_snake_case()
    }

    pub fn from_trait_method_name(trait_method_name: impl AsRef<str>) -> Option<Hooks> {
        Hooks::iter().find(|hook| hook.get_trait_method_name() == trait_method_name.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_c_method_name() {
        assert_eq!(Hooks::OnVMStarted.get_c_method_name(), "on_vm_started");
        assert_eq!(Hooks::OnVMStopping.get_c_method_name(), "on_vm_stopping");
        assert_eq!(
            Hooks::OnDistributionStarted.get_c_method_name(),
            "on_distribution_started"
        );
        assert_eq!(
            Hooks::OnDistributionStopping.get_c_method_name(),
            "on_distribution_stopping"
        );
        assert_eq!(
            Hooks::OnDistributionRegistered.get_c_method_name(),
            "on_distribution_registered"
        );
        assert_eq!(
            Hooks::OnDistributionUnregistered.get_c_method_name(),
            "on_distribution_unregistered"
        );
    }

    #[test]
    fn test_get_hook_field_name() {
        assert_eq!(Hooks::OnVMStarted.get_hook_field_name(), "OnVMStarted");
        assert_eq!(Hooks::OnVMStopping.get_hook_field_name(), "OnVMStopping");
        assert_eq!(
            Hooks::OnDistributionStarted.get_hook_field_name(),
            "OnDistributionStarted"
        );
        assert_eq!(
            Hooks::OnDistributionStopping.get_hook_field_name(),
            "OnDistributionStopping"
        );
        assert_eq!(
            Hooks::OnDistributionRegistered.get_hook_field_name(),
            "OnDistributionRegistered"
        );
        assert_eq!(
            Hooks::OnDistributionUnregistered.get_hook_field_name(),
            "OnDistributionUnregistered"
        );
    }

    #[test]
    fn test_get_trait_method_name() {
        assert_eq!(Hooks::OnVMStarted.get_trait_method_name(), "on_vm_started");
        assert_eq!(
            Hooks::OnVMStopping.get_trait_method_name(),
            "on_vm_stopping"
        );
        assert_eq!(
            Hooks::OnDistributionStarted.get_trait_method_name(),
            "on_distribution_started"
        );
        assert_eq!(
            Hooks::OnDistributionStopping.get_trait_method_name(),
            "on_distribution_stopping"
        );
        assert_eq!(
            Hooks::OnDistributionRegistered.get_trait_method_name(),
            "on_distribution_registered"
        );
        assert_eq!(
            Hooks::OnDistributionUnregistered.get_trait_method_name(),
            "on_distribution_unregistered"
        );
    }

    #[test]
    fn test_from_trait_method_name() {
        assert_eq!(
            Hooks::from_trait_method_name("on_vm_started").unwrap(),
            Hooks::OnVMStarted
        );
        assert_eq!(
            Hooks::from_trait_method_name("on_vm_stopping").unwrap(),
            Hooks::OnVMStopping
        );
        assert_eq!(
            Hooks::from_trait_method_name("on_distribution_started").unwrap(),
            Hooks::OnDistributionStarted
        );
        assert_eq!(
            Hooks::from_trait_method_name("on_distribution_stopping").unwrap(),
            Hooks::OnDistributionStopping
        );
        assert_eq!(
            Hooks::from_trait_method_name("on_distribution_registered").unwrap(),
            Hooks::OnDistributionRegistered
        );
        assert_eq!(
            Hooks::from_trait_method_name("on_distribution_unregistered").unwrap(),
            Hooks::OnDistributionUnregistered
        );
        assert!(Hooks::from_trait_method_name("invalid_method_name").is_none());
    }
}

use crate::WSLPluginAPIV1;
use windows::core::HRESULT;
use windows::Win32::Foundation::{SEVERITY_ERROR, S_OK};
use windows::Win32::System::Diagnostics::Debug::{FACILITY_CODE, FACILITY_ITF};

#[inline(always)]
const fn make_hresult(severity: u32, facility: FACILITY_CODE, code: u32) -> HRESULT {
    HRESULT(((severity << 31) | (facility.0 << 16) | code) as i32)
}

pub const WSL_E_PLUGIN_REQUIRES_UPDATE: HRESULT =
    make_hresult(SEVERITY_ERROR, FACILITY_ITF, 0x8004032A);

/// Ensures the WSL Plugin API version meets the minimum required version.
///
/// This function compares the version of the API passed as a parameter against the required
/// version numbers specified (`required_major`, `required_minor`, `required_revision`).
/// If the API version is lower than required, it returns `WSL_E_PLUGIN_REQUIRES_UPDATE`.
/// Otherwise, it returns `S_OK`.
///
/// # Parameters
///
/// - `required_major`: The major version number required by the plugin.
/// - `required_minor`: The minor version number required by the plugin.
/// - `required_revision`: The revision number required by the plugin.
/// - `api`: A pointer to the `WSLPluginAPIV1` structure, containing the current API version.
///
/// # Returns
///
/// - `S_OK`: If the API version meets or exceeds the required version.
/// - `WSL_E_PLUGIN_REQUIRES_UPDATE`: If the API version is below the required minimum.
///
/// # Safety
///
/// This function is `unsafe` because it dereferences a raw pointer (`api`). The caller must
/// ensure that the pointer is valid and points to a properly initialized `WSLPluginAPIV1`
/// structure.
#[inline(always)]
pub const unsafe fn require_version(
    required_major: u32,
    required_minor: u32,
    required_revision: u32,
    api: *const WSLPluginAPIV1,
) -> HRESULT {
    let version = &(*api).Version;

    if version.Major < required_major
        || (version.Major == required_major && version.Minor < required_minor)
        || (version.Major == required_major
            && version.Minor == required_minor
            && version.Revision < required_revision)
    {
        WSL_E_PLUGIN_REQUIRES_UPDATE
    } else {
        S_OK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WSLPluginAPIV1, WSLVersion};
    use windows::Win32::Foundation::S_OK;

    #[test]
    fn test_version_exact_match() {
        let api = WSLPluginAPIV1 {
            Version: WSLVersion {
                Major: 1,
                Minor: 0,
                Revision: 0,
            },
            MountFolder: None,
            ExecuteBinary: None,
            PluginError: None,
            ExecuteBinaryInDistribution: None,
        };

        assert_eq!(unsafe { require_version(1, 0, 0, &api) }, S_OK);
    }

    #[test]
    fn test_version_major_too_low() {
        let api = WSLPluginAPIV1 {
            Version: WSLVersion {
                Major: 0,
                Minor: 9,
                Revision: 0,
            },
            MountFolder: None,
            ExecuteBinary: None,
            PluginError: None,
            ExecuteBinaryInDistribution: None,
        };

        assert_eq!(
            unsafe { require_version(1, 0, 0, &api) },
            WSL_E_PLUGIN_REQUIRES_UPDATE
        );
    }

    #[test]
    fn test_version_minor_too_low() {
        let api = WSLPluginAPIV1 {
            Version: WSLVersion {
                Major: 1,
                Minor: 0,
                Revision: 0,
            },
            MountFolder: None,
            ExecuteBinary: None,
            PluginError: None,
            ExecuteBinaryInDistribution: None,
        };

        assert_eq!(
            unsafe { require_version(1, 1, 0, &api) },
            WSL_E_PLUGIN_REQUIRES_UPDATE
        );
    }

    #[test]
    fn test_version_revision_too_low() {
        let api = WSLPluginAPIV1 {
            Version: WSLVersion {
                Major: 1,
                Minor: 0,
                Revision: 0,
            },
            MountFolder: None,
            ExecuteBinary: None,
            PluginError: None,
            ExecuteBinaryInDistribution: None,
        };

        assert_eq!(
            unsafe { require_version(1, 0, 1, &api) },
            WSL_E_PLUGIN_REQUIRES_UPDATE
        );
    }

    #[test]
    fn test_version_high_enough() {
        let api = WSLPluginAPIV1 {
            Version: WSLVersion {
                Major: 1,
                Minor: 2,
                Revision: 3,
            },
            MountFolder: None,
            ExecuteBinary: None,
            PluginError: None,
            ExecuteBinaryInDistribution: None,
        };

        assert_eq!(unsafe { require_version(1, 0, 1, &api) }, S_OK);
    }
}

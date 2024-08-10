use crate::WSLPluginAPIV1;
use windows::core::HRESULT;
use windows::Win32::Foundation::{SEVERITY_ERROR, S_OK};
use windows::Win32::System::Diagnostics::Debug::{FACILITY_CODE, FACILITY_ITF};

#[inline(always)]
const fn make_hresult(severity: u32, facility: FACILITY_CODE, code: u32) -> HRESULT {
    HRESULT(((severity << 31) | (facility.0 << 16) | code) as i32)
}

const WSL_E_PLUGIN_REQUIRES_UPDATE: HRESULT =
    make_hresult(SEVERITY_ERROR, FACILITY_ITF, 0x8004032A);

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

        assert_eq!(require_version(1, 0, 0, &api), S_OK);
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

        assert_eq!(require_version(1, 0, 0, &api), WSL_E_PLUGIN_REQUIRES_UPDATE);
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

        assert_eq!(require_version(1, 1, 0, &api), WSL_E_PLUGIN_REQUIRES_UPDATE);
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

        assert_eq!(require_version(1, 0, 1, &api), WSL_E_PLUGIN_REQUIRES_UPDATE);
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

        assert_eq!(require_version(1, 0, 1, &api), S_OK);
    }
}

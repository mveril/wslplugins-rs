#![doc = include_str!("../README.md")]

use std::{
    fs::OpenOptions,
    io::Write,
    mem::{size_of, zeroed},
    os::windows::io::AsRawHandle,
};
use windows::Win32::{
    Foundation::{E_FAIL, HANDLE},
    Security::{GetTokenInformation, TokenStatistics, TOKEN_STATISTICS},
};
use wslplugins_rs::prelude::*;

const LOG_PATH: &str = "C:\\wsl-session-user-audit.log";

#[derive(Debug)]
pub(crate) struct Plugin;

#[wsl_plugin_v1]
impl WSLPluginV1 for Plugin {
    fn try_new(_: &'static WSLContext) -> WinResult<Self> {
        Ok(Self)
    }

    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        _: &WSLVmCreationSettings,
    ) -> PluginResult<()> {
        let sid = session.user_sid();
        let account = sid.lookup_local_sid().and_then(Result::ok).map_or_else(
            || "<unresolved>".to_owned(),
            |lookup| lookup.domain_name.to_string(),
        );
        let authentication_id = authentication_id(session)?;

        write_audit_line(session.id(), &sid.to_string(), &account, authentication_id)?;
        Ok(())
    }
}

fn authentication_id(session: &WSLSessionInformation) -> WinResult<u64> {
    // SAFETY: `TOKEN_STATISTICS` is a plain Windows output structure whose
    // fields may all be initialized to zero before `GetTokenInformation` fills it.
    let mut statistics: TOKEN_STATISTICS = unsafe { zeroed() };
    let mut returned_size = 0;

    // SAFETY: `statistics` is a writable buffer of the exact size required for
    // `TOKEN_STATISTICS`. The handle comes from WSL and remains valid for the
    // lifetime of `session`; this call only reads from it.
    unsafe {
        GetTokenInformation(
            HANDLE(session.user_token().as_raw_handle()),
            TokenStatistics,
            Some((&raw mut statistics).cast()),
            size_of::<TOKEN_STATISTICS>() as u32,
            &raw mut returned_size,
        )?;
    }

    let high_part = u64::from(statistics.AuthenticationId.HighPart as u32);
    let low_part = u64::from(statistics.AuthenticationId.LowPart);
    Ok((high_part << 32) | low_part)
}

fn write_audit_line(
    session_id: SessionID,
    sid: &str,
    account: &str,
    authentication_id: u64,
) -> WinResult<()> {
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)
        .map_err(|_| WinError::from(E_FAIL))?;

    writeln!(
        log,
        "session={session_id}; sid={sid}; account={account}; authentication_id=0x{authentication_id:016X}"
    )
    .map_err(|_| WinError::from(E_FAIL))
}

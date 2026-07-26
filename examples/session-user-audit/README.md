# Session user audit example

This plugin writes the Windows identity associated with each WSL session to
`C:\wsl-session-user-audit.log`.

It demonstrates the two user-identity APIs exposed by `WSLSessionInformation`:

- `session.user_sid()` returns a borrowed [`Sid`]. The example formats it and
  resolves the local `DOMAIN\\user` account name when Windows can find one.
- `session.user_token()` returns a `BorrowedHandle`. The example passes it to
  `GetTokenInformation(TokenStatistics)` to record the token authentication ID.

The handle is owned by WSL and is valid only for the callback's session. A
`BorrowedHandle` must not be stored beyond that lifetime and must never be
closed by the plugin.

The lookup is diagnostic only: an unresolved SID does not block WSL startup.

[`Sid`]: https://docs.rs/win-security-identifier/latest/win_security_identifier/struct.Sid.html

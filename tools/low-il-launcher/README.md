# W22 Low IL launcher

Windows / .NET 10. No NuGet dependencies. The supervisor starts this as a
**separate, non-elevated Medium IL process**. It creates one new private Low IL
directory, lowers only its own token, creates a suspended worker, checks its Low
token, assigns it to a kill-on-close job, and resumes it. Failure never falls back
to a Medium worker. Closing the launcher terminates the worker job and descendants.

```powershell
dotnet build tools/low-il-launcher/LowIlLauncher.csproj -c Release
& '<absolute path>/LowIlLauncher.exe' -- '<absolute worker.exe>' '<argument>'
```

The default temp parent is Windows `FOLDERID_LocalAppDataLow`. `--temp-parent`
accepts an existing local NTFS directory for isolated tests/deployments; it never
relabels that parent. Each invocation creates a fresh `Nemu-worker-<GUID>` child
with a protected current-user/SYSTEM DACL and an inheritable Low No-Write-Up label.
`TEMP`, `TMP`, the profile/cache environment and working directory point there.
The launcher prints `worker_temp=<path>`; the supervisor owns eventual cleanup.
There is no recursive deletion in the launcher. Existing ACLs, storage labels,
registry, machine/user environment and system policy are not changed.

Executable paths must be absolute local NTFS `.exe` paths. Reparse-point ancestors
are rejected. Arguments use Windows C-runtime quoting; no shell or PATH lookup.
The child receives an environment allowlist and no inherited handles or standard
streams. Future pipe authentication uses the supervisor/worker contract, not an
inherited storage handle. The caller must protect executable and ancestor paths
against concurrent replacement; path checks do not establish a trusted installer.

Low IL provides a **write boundary**, not confidentiality or network isolation:
it does not prevent reading otherwise readable Medium files, or writing other
Low resources accessible to the same user. Keep authority storage at Medium IL
with no pre-opened writable handles crossing the boundary. Authentication, protocol
capabilities, resource budgets, supervisor registration and actual core-store
integration remain pending (see `tasks/requests/P5-05-permission-integration.md`).

## Verification

Run from a separate PowerShell process, with no active build/CI job. The script
uses only process-local environment values and a new D-drive work directory.

```powershell
pwsh -NoProfile -File tools/low-il-launcher/test.ps1 -WorkRoot D:/Dev/cache/P5-05/<new-run>
```

The test uses synthetic database-named files, not an actual SQLite/core-store
database. It checks Medium positive writes; Low worker/temporary directory labels;
14 denied write/create/delete/rename operations; unchanged fixture hashes/labels;
Low positive writes and label inheritance; argument preservation; environment
filtering; descendant termination; unchanged supervisor IL; invalid paths.
This is OS-layer evidence only, not full SYS09, protocol isolation, or I26.
All files and logs are retained under the supplied work root, including failures.

## API references

- [Mandatory Integrity Control](https://learn.microsoft.com/en-us/windows/win32/secauthz/mandatory-integrity-control)
- [CreateDirectoryW: security applied at creation](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createdirectoryw)
- [CreateProcessW: caller token, suspended creation, handle inheritance](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw)
- [Job extended limits](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information)
- [Known folder identifiers](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [Windows C-runtime argument parsing](https://learn.microsoft.com/en-us/cpp/c-language/parsing-c-command-line-arguments?view=msvc-170)

# Redact machine-specific roots before logs reach either the console or artifacts.
function Protect-WindowsEvidenceText {
    [CmdletBinding()]
    param([Parameter(ValueFromPipeline)]$InputObject)
    begin {
        $roots = @(
            @([IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..')), '<checkout>'),
            @($env:USERPROFILE, '<user-profile>'), @($env:LOCALAPPDATA, '<local-app-data>'),
            @($env:APPDATA, '<app-data>'), @($env:RUNNER_TEMP, '<runner-temp>'),
            @($env:CARGO_HOME, '<cargo-home>'), @($env:RUSTUP_HOME, '<rustup-home>'),
            @($env:CARGO_TARGET_DIR, '<cargo-target>'), @($env:TEMP, '<temp>'), @($env:TMP, '<temp>')
        ) | Where-Object { $_[0] } | Sort-Object { $_[0].Length } -Descending
    }
    process {
        $line = [string]$InputObject
        foreach ($pair in $roots) {
            foreach ($rootPath in @($pair[0], $pair[0].Replace('\', '/'))) {
                $line = [regex]::Replace($line, [regex]::Escape($rootPath), $pair[1], 'IgnoreCase')
            }
        }
        $line
    }
}

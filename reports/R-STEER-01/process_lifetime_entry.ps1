# Keyless reproduction of the former attached PowerShell -> Python ownership.
[CmdletBinding()]
param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
[ordered]@{utc=[DateTimeOffset]::UtcNow.ToString('o'); pid=$PID; phase='attached_entry'} | ConvertTo-Json | Set-Content (Join-Path $OutputDirectory 'entry.json') -Encoding utf8NoBOM
& 'D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe' -B (Join-Path $PSScriptRoot 'process_lifetime_probe.py') beat --output $OutputDirectory
[ordered]@{utc=[DateTimeOffset]::UtcNow.ToString('o'); exit_code=$LASTEXITCODE} | ConvertTo-Json | Set-Content (Join-Path $OutputDirectory 'entry-terminal.json') -Encoding utf8NoBOM

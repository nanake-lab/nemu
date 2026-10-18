[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function ConvertTo-DoctorVersion {
    param([AllowNull()][string]$Text)
    if ([string]::IsNullOrWhiteSpace($Text)) { return $null }
    $match = [regex]::Match($Text, '(?<!\d)(\d+\.\d+(?:\.\d+){0,2})(?!\d)')
    if (-not $match.Success) { return $null }
    try { return [version]$match.Groups[1].Value } catch { return $null }
}

function New-DoctorCheck {
    param(
        [Parameter(Mandatory)][string]$Project,
        [Parameter(Mandatory)][string]$Requirement,
        [AllowNull()][string]$Actual,
        [Parameter(Mandatory)][ValidateSet('PASS','FAIL','BLOCKED')][string]$Status,
        [Parameter(Mandatory)][string]$Repair
    )
    [pscustomobject][ordered]@{
        Project = $Project
        Requirement = $Requirement
        Actual = if ([string]::IsNullOrWhiteSpace($Actual)) { '未检测到' } else { $Actual.Trim() }
        Status = $Status
        Repair = $Repair
    }
}

function New-DoctorVersionCheck {
    param(
        [Parameter(Mandatory)][string]$Project,
        [Parameter(Mandatory)][string]$Requirement,
        [Parameter(Mandatory)][bool]$Present,
        [AllowNull()][string]$ActualText,
        [string]$MinimumVersion,
        [string]$MaximumExclusive,
        [string]$ExactVersion,
        [Parameter(Mandatory)][string]$Repair
    )
    if (-not $Present) {
        return New-DoctorCheck $Project $Requirement $ActualText 'FAIL' $Repair
    }
    $actual = ConvertTo-DoctorVersion $ActualText
    if ($null -eq $actual) {
        return New-DoctorCheck $Project $Requirement $ActualText 'FAIL' $Repair
    }
    if ($ExactVersion) {
        $expected = ConvertTo-DoctorVersion $ExactVersion
        if ($null -eq $expected -or $actual -ne $expected) {
            return New-DoctorCheck $Project $Requirement $ActualText 'FAIL' $Repair
        }
    }
    if ($MinimumVersion) {
        $minimum = ConvertTo-DoctorVersion $MinimumVersion
        if ($null -eq $minimum -or $actual -lt $minimum) {
            return New-DoctorCheck $Project $Requirement $ActualText 'FAIL' $Repair
        }
    }
    if ($MaximumExclusive) {
        $maximum = ConvertTo-DoctorVersion $MaximumExclusive
        if ($null -eq $maximum -or $actual -ge $maximum) {
            return New-DoctorCheck $Project $Requirement $ActualText 'FAIL' $Repair
        }
    }
    New-DoctorCheck $Project $Requirement $ActualText 'PASS' $Repair
}

function Invoke-DoctorCommand {
    param([Parameter(Mandatory)][string]$Name, [string[]]$Arguments = @())
    $command = Get-Command -Name $Name -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $command) {
        return [pscustomobject]@{ Found = $false; ExitCode = 127; Text = '' }
    }
    try {
        $output = @(& $command.Source @Arguments 2>&1 | ForEach-Object { $_.ToString() })
        $exitCode = if ($null -eq $LASTEXITCODE) { 0 } else { [int]$LASTEXITCODE }
        [pscustomobject]@{ Found = $true; ExitCode = $exitCode; Text = ($output -join "`n").Trim() }
    } catch {
        [pscustomobject]@{ Found = $true; ExitCode = 1; Text = $_.Exception.Message }
    }
}

function Add-DoctorVersionRow {
    param(
        [System.Collections.Generic.List[object]]$Rows,
        [string]$Project,
        [string]$Requirement,
        [string]$Command,
        [string[]]$Arguments,
        [string]$MinimumVersion,
        [string]$MaximumExclusive,
        [string]$ExactVersion,
        [string]$Repair
    )
    $result = Invoke-DoctorCommand $Command $Arguments
    $Rows.Add((New-DoctorVersionCheck -Project $Project -Requirement $Requirement -Present $result.Found -ActualText $result.Text -MinimumVersion $MinimumVersion -MaximumExclusive $MaximumExclusive -ExactVersion $ExactVersion -Repair $Repair))
}

function Get-InstalledPrograms {
    $uninstallPaths = @(
        'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*',
        'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*',
        'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*'
    )
    foreach ($path in $uninstallPaths) {
        Get-ItemProperty -Path $path -ErrorAction SilentlyContinue
    }
}

function Test-VRoidStartAppName {
    param([AllowNull()][string]$Name)
    return $Name -match '^VRoid\s*Studio(?:\s|$)'
}

function Find-BlenderExecutable {
    $candidates = [System.Collections.Generic.List[string]]::new()
    if ($env:NEMU_BLENDER) {
        if (Test-Path -LiteralPath $env:NEMU_BLENDER -PathType Leaf) { $candidates.Add($env:NEMU_BLENDER) }
        elseif (Test-Path -LiteralPath $env:NEMU_BLENDER -PathType Container) { $candidates.Add((Join-Path $env:NEMU_BLENDER 'blender.exe')) }
    }
    $command = Get-Command blender -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($command) { $candidates.Add($command.Source) }
    foreach ($program in (Get-InstalledPrograms | Where-Object { $_.PSObject.Properties['DisplayName'] -and $_.DisplayName -match '^Blender' })) {
        if ($program.PSObject.Properties['InstallLocation'] -and $program.InstallLocation) { $candidates.Add((Join-Path $program.InstallLocation 'blender.exe')) }
        if ($program.PSObject.Properties['DisplayIcon'] -and $program.DisplayIcon) { $candidates.Add(($program.DisplayIcon -replace ',\d+$','').Trim('"')) }
    }
    foreach ($base in @(
        (Join-Path $env:ProgramFiles 'Blender Foundation'),
        (Join-Path $env:LOCALAPPDATA 'Programs\Blender Foundation')
    )) {
        if (Test-Path -LiteralPath $base -PathType Container) {
            Get-ChildItem -LiteralPath $base -Filter blender.exe -File -Recurse -Depth 3 -ErrorAction SilentlyContinue | ForEach-Object { $candidates.Add($_.FullName) }
        }
    }
    foreach ($candidate in $candidates) {
        if (Test-Path -LiteralPath $candidate -PathType Leaf) { return (Resolve-Path -LiteralPath $candidate).Path }
    }
    $null
}

function Get-BlenderInfo {
    param([Parameter(Mandatory)][string]$Executable)
    $expression = "import bpy; print('NEMU_BLENDER_VERSION=' + bpy.app.version_string); print('NEMU_VRM_ADDON=' + str('io_scene_vrm' in bpy.context.preferences.addons))"
    try {
        $lines = @(& $Executable --background --python-expr $expression 2>&1 | ForEach-Object { $_.ToString() })
        $exitCode = if ($null -eq $LASTEXITCODE) { 0 } else { [int]$LASTEXITCODE }
        $versionLine = $lines | Where-Object { $_ -match 'NEMU_BLENDER_VERSION=' } | Select-Object -Last 1
        $addonLine = $lines | Where-Object { $_ -match 'NEMU_VRM_ADDON=' } | Select-Object -Last 1
        [pscustomobject]@{
            ExitCode = $exitCode
            Version = if ($versionLine) { ($versionLine -replace '^.*NEMU_BLENDER_VERSION=','').Trim() } else { '' }
            AddonEnabled = if ($addonLine) { ($addonLine -replace '^.*NEMU_VRM_ADDON=','').Trim() -eq 'True' } else { $false }
            Output = ($lines -join "`n").Trim()
        }
    } catch {
        [pscustomobject]@{ ExitCode = 1; Version = ''; AddonEnabled = $false; Output = $_.Exception.Message }
    }
}

function Test-CurrentUserNamedPipe {
    try {
        Add-Type -AssemblyName System.IO.Pipes.AccessControl -ErrorAction Stop
        $security = [System.IO.Pipes.PipeSecurity]::new()
        $currentSid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User
        $rule = [System.IO.Pipes.PipeAccessRule]::new($currentSid, [System.IO.Pipes.PipeAccessRights]::FullControl, [System.Security.AccessControl.AccessControlType]::Allow)
        $security.AddAccessRule($rule)
        $name = 'nemu-doctor-' + [guid]::NewGuid().ToString('N')
        $pipe = [System.IO.Pipes.NamedPipeServerStreamAcl]::Create($name, [System.IO.Pipes.PipeDirection]::InOut, 1, [System.IO.Pipes.PipeTransmissionMode]::Byte, [System.IO.Pipes.PipeOptions]::None, 0, 0, $security)
        $pipe.Dispose()
        [pscustomobject]@{ Passed = $true; Detail = '已创建仅含当前用户 ACE 的命名管道' }
    } catch {
        [pscustomobject]@{ Passed = $false; Detail = $_.Exception.Message }
    }
}

function Test-LowIntegrityWriteDenial {
    $temporaryDirectory = Join-Path $env:TEMP ('nemu-doctor-lowil-' + [guid]::NewGuid().ToString('N'))
    $targetFile = Join-Path $temporaryDirectory 'must-not-write.txt'
    try {
        New-Item -ItemType Directory -Path $temporaryDirectory | Out-Null
        $icacls = Get-Command icacls.exe -ErrorAction SilentlyContinue
        if (-not $icacls) { return [pscustomobject]@{ Passed = $false; Detail = 'icacls.exe is unavailable' } }
        $labelOutput = & $icacls.Source $temporaryDirectory '/setintegritylevel' 'M' 2>&1
        if ($LASTEXITCODE -ne 0) { return [pscustomobject]@{ Passed = $false; Detail = ($labelOutput -join ' ') } }
        if (-not ('NemuDoctor.LowIntegrityProcess' -as [type])) {
            Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
namespace NemuDoctor {
  public static class LowIntegrityProcess {
    [StructLayout(LayoutKind.Sequential)] private struct SidAndAttributes { public IntPtr Sid; public uint Attributes; }
    [StructLayout(LayoutKind.Sequential)] private struct TokenMandatoryLabel { public SidAndAttributes Label; }
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] private struct StartupInfo {
      public int cb; public string lpReserved; public string lpDesktop; public string lpTitle;
      public int dwX; public int dwY; public int dwXSize; public int dwYSize; public int dwXCountChars;
      public int dwYCountChars; public int dwFillAttribute; public int dwFlags; public short wShowWindow;
      public short cbReserved2; public IntPtr lpReserved2; public IntPtr hStdInput; public IntPtr hStdOutput; public IntPtr hStdError;
    }
    [StructLayout(LayoutKind.Sequential)] private struct ProcessInformation { public IntPtr hProcess; public IntPtr hThread; public int dwProcessId; public int dwThreadId; }
    [DllImport("kernel32.dll", SetLastError=true)] private static extern IntPtr GetCurrentProcess();
    [DllImport("kernel32.dll", SetLastError=true)] private static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll", SetLastError=true)] private static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);
    [DllImport("kernel32.dll", SetLastError=true)] private static extern bool GetExitCodeProcess(IntPtr process, out uint exitCode);
    [DllImport("advapi32.dll", SetLastError=true)] private static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
    [DllImport("advapi32.dll", SetLastError=true)] private static extern bool DuplicateTokenEx(IntPtr existing, uint access, IntPtr attributes, int impersonationLevel, int tokenType, out IntPtr token);
    [DllImport("advapi32.dll", SetLastError=true)] private static extern bool ConvertStringSidToSid(string sid, out IntPtr result);
    [DllImport("advapi32.dll", SetLastError=true)] private static extern uint GetLengthSid(IntPtr sid);
    [DllImport("advapi32.dll", SetLastError=true)] private static extern bool SetTokenInformation(IntPtr token, int informationClass, IntPtr information, uint length);
    [DllImport("advapi32.dll", SetLastError=true, CharSet=CharSet.Unicode)] private static extern bool CreateProcessWithTokenW(IntPtr token, uint logonFlags, string application, StringBuilder commandLine, uint creationFlags, IntPtr environment, string currentDirectory, ref StartupInfo startup, out ProcessInformation process);
    [DllImport("kernel32.dll", SetLastError=true)] private static extern bool TerminateProcess(IntPtr process, uint exitCode);
    [DllImport("kernel32.dll", SetLastError=true)] private static extern IntPtr LocalFree(IntPtr memory);
    private static void Fail(string action) { throw new Win32Exception(Marshal.GetLastWin32Error(), action); }
    public static int Run(string executable, string arguments, string workingDirectory) {
      IntPtr source = IntPtr.Zero, token = IntPtr.Zero, sid = IntPtr.Zero, labelBuffer = IntPtr.Zero;
      ProcessInformation process = new ProcessInformation();
      try {
        if (!OpenProcessToken(GetCurrentProcess(), 0x0002 | 0x0008, out source)) Fail("OpenProcessToken");
        uint rights = 0x0001 | 0x0002 | 0x0008 | 0x0080 | 0x0100;
        if (!DuplicateTokenEx(source, rights, IntPtr.Zero, 2, 1, out token)) Fail("DuplicateTokenEx");
        if (!ConvertStringSidToSid("S-1-16-4096", out sid)) Fail("ConvertStringSidToSid");
        TokenMandatoryLabel label = new TokenMandatoryLabel { Label = new SidAndAttributes { Sid = sid, Attributes = 0x00000020 } };
        int size = Marshal.SizeOf(typeof(TokenMandatoryLabel));
        labelBuffer = Marshal.AllocHGlobal(size);
        Marshal.StructureToPtr(label, labelBuffer, false);
        uint labelLength = (uint)(size + GetLengthSid(sid));
        if (!SetTokenInformation(token, 25, labelBuffer, labelLength)) Fail("SetTokenInformation(TokenIntegrityLevel)");
        StartupInfo startup = new StartupInfo(); startup.cb = Marshal.SizeOf(typeof(StartupInfo));
        if (!CreateProcessWithTokenW(token, 0, executable, new StringBuilder(arguments), 0x08000000, IntPtr.Zero, workingDirectory, ref startup, out process)) Fail("CreateProcessWithTokenW");
        uint wait = WaitForSingleObject(process.hProcess, 30000);
        if (wait == 0x00000102) { TerminateProcess(process.hProcess, 0xFFFFFFFF); throw new TimeoutException("Low integrity process timed out."); }
        uint exitCode;
        if (!GetExitCodeProcess(process.hProcess, out exitCode)) Fail("GetExitCodeProcess");
        return unchecked((int)exitCode);
      } finally {
        if (process.hThread != IntPtr.Zero) CloseHandle(process.hThread);
        if (process.hProcess != IntPtr.Zero) CloseHandle(process.hProcess);
        if (labelBuffer != IntPtr.Zero) Marshal.FreeHGlobal(labelBuffer);
        if (sid != IntPtr.Zero) LocalFree(sid);
        if (token != IntPtr.Zero) CloseHandle(token);
        if (source != IntPtr.Zero) CloseHandle(source);
      }
    }
  }
}
'@
        }
        $powerShell = Join-Path $PSHOME 'pwsh.exe'
        if (-not (Test-Path -LiteralPath $powerShell -PathType Leaf)) { return [pscustomobject]@{ Passed = $false; Detail = 'pwsh.exe was not found beside the current PowerShell host' } }
        $escapedTarget = $targetFile.Replace("'", "''")
        $command = "try { [System.IO.File]::WriteAllText('$escapedTarget','probe'); exit 0 } catch [System.UnauthorizedAccessException] { exit 1 } catch { exit 2 }"
        $arguments = '-NoLogo -NoProfile -NonInteractive -Command "' + $command + '"'
        $exitCode = [NemuDoctor.LowIntegrityProcess]::Run($powerShell, $arguments, $temporaryDirectory)
        $written = Test-Path -LiteralPath $targetFile
        if ($exitCode -eq 1 -and -not $written) { return [pscustomobject]@{ Passed = $true; Detail = 'Low IL 子进程写入 Medium IL 临时目录时被拒绝' } }
        if ($written) { return [pscustomobject]@{ Passed = $false; Detail = 'Low IL 子进程意外写入了 Medium IL 临时目录' } }
        [pscustomobject]@{ Passed = $false; Detail = "Low IL 子进程返回非预期代码 $exitCode" }
    } catch {
        [pscustomobject]@{ Passed = $false; Detail = $_.Exception.Message }
    } finally {
        Remove-Item -LiteralPath $temporaryDirectory -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Invoke-NemuDoctor {
    $rows = [System.Collections.Generic.List[object]]::new()
    $root = Split-Path -Parent $PSScriptRoot
    $packageRepair = '按缺少项执行 winget install <包 ID>；包 ID 可用 winget search 核对。'
    Add-DoctorVersionRow $rows 'Git' 'Git 已安装' 'git' @('--version') '2.0.0' $null $null 'winget install Git.Git'
    Add-DoctorVersionRow $rows 'Git LFS' 'Git LFS 已安装' 'git' @('lfs','version') '3.0.0' $null $null 'winget install GitHub.GitLFS'
    Add-DoctorVersionRow $rows 'Node.js' '24.x' 'node' @('--version') '24.0.0' '25.0.0' $null 'fnm install 24; fnm default 24'
    Add-DoctorVersionRow $rows 'pnpm' 'corepack pnpm 12.11.2' 'corepack' @('pnpm','--version') $null $null '12.11.2' 'corepack enable; corepack prepare pnpm@12.11.2 --activate'
    Add-DoctorVersionRow $rows 'Python' '3.12.x' 'python' @('--version') '3.12.0' '3.13.0' $null 'winget install Python.Python.3.12'
    Add-DoctorVersionRow $rows 'uv' 'uv 已安装' 'uv' @('--version') '0.1.0' $null $null 'winget install astral-sh.uv'

    $rustup = Invoke-DoctorCommand 'rustup' @('show','active-toolchain')
    $rustTargets = Invoke-DoctorCommand 'rustup' @('target','list','--installed')
    $rustOk = $rustup.Found -and $rustup.ExitCode -eq 0 -and $rustTargets.Text -match 'x86_64-pc-windows-msvc'
    $rows.Add((New-DoctorCheck 'Rust/MSVC' 'rustup stable 与 x86_64-pc-windows-msvc target' ($rustup.Text + '; targets=' + $rustTargets.Text) $(if ($rustOk) {'PASS'} else {'FAIL'}) 'winget install Rustlang.Rustup; rustup default stable; rustup target add x86_64-pc-windows-msvc'))

    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    $vsPath = ''
    if (Test-Path -LiteralPath $vswhere) {
        $vsOutput = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($LASTEXITCODE -eq 0) { $vsPath = ($vsOutput | Select-Object -First 1).Trim() }
    }
    $rows.Add((New-DoctorCheck 'VS Build Tools' 'C++ x64/x86 workload (vswhere)' $vsPath $(if ($vsPath) {'PASS'} else {'FAIL'}) 'winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive"'))

    $dotnet = Invoke-DoctorCommand 'dotnet' @('--list-sdks')
    $sdk10 = $dotnet.Text -split "`n" | Where-Object { $_ -match '^10\.' } | Select-Object -First 1
    $rows.Add((New-DoctorCheck ' .NET SDK' '10.x SDK' $sdk10 $(if ($sdk10) {'PASS'} else {'FAIL'}) 'winget install Microsoft.DotNet.SDK.10'))

    $nvidia = Invoke-DoctorCommand 'nvidia-smi' @('--query-gpu=name,driver_version,memory.free','--format=csv,noheader,nounits')
    $rows.Add((New-DoctorCheck 'NVIDIA Driver/GPU' 'nvidia-smi 可查询 GPU 与驱动' $nvidia.Text $(if ($nvidia.Found -and $nvidia.ExitCode -eq 0 -and $nvidia.Text) {'PASS'} else {'FAIL'}) '安装或更新 NVIDIA Studio Driver；使用 nvidia-smi 验证'))
    $nvcc = Invoke-DoctorCommand 'nvcc' @('--version')
    $rows.Add((New-DoctorCheck 'CUDA Toolkit' 'nvcc 可用' $nvcc.Text $(if ($nvcc.Found -and $nvcc.ExitCode -eq 0) {'PASS'} else {'FAIL'}) '按项目 CUDA/驱动要求安装 CUDA Toolkit；用 winget search CUDA 核对安装包'))
    $torch = Invoke-DoctorCommand 'uv' @('run','--project',(Join-Path $root 'workers/model-adapter'),'--no-sync','python','-c',"import torch; print('torch=' + torch.__version__ + '; cuda=' + str(torch.cuda.is_available()))")
    $torchAvailable = $torch.Found -and $torch.ExitCode -eq 0 -and $torch.Text -match 'cuda=True'
    $rows.Add((New-DoctorCheck 'PyTorch CUDA' 'model-adapter uv 环境 torch.cuda.is_available()' $torch.Text $(if ($torchAvailable) {'PASS'} else {'FAIL'}) '在 workers/model-adapter 执行 uv sync；检查 CUDA 版 PyTorch 与驱动兼容性'))

    $modelLockPath = Join-Path $root 'config/models.lock.json'
    $expectedCodex = (Get-Content -LiteralPath $modelLockPath -Raw | ConvertFrom-Json).codex.version
    $codex = Invoke-DoctorCommand 'codex' @('--version')
    $codexVersion = ConvertTo-DoctorVersion $codex.Text
    $codexVersionOk = $codex.Found -and $codex.ExitCode -eq 0 -and $codexVersion -eq (ConvertTo-DoctorVersion $expectedCodex)
    $rows.Add((New-DoctorCheck 'Codex CLI' "版本与 models.lock.json 一致 ($expectedCodex)" $codex.Text $(if ($codexVersionOk) {'PASS'} else {'FAIL'}) 'npm install -g @openai/codex@<锁定版本>'))
    $login = Invoke-DoctorCommand 'codex' @('login','status')
    $loginOk = $login.Found -and $login.ExitCode -eq 0 -and $login.Text -match 'Logged in'
    $rows.Add((New-DoctorCheck 'Codex 登录' 'codex login status 显示已登录' $login.Text $(if ($loginOk) {'PASS'} else {'FAIL'}) '运行 codex login；doctor 只检查状态，不读取凭据'))

    $blenderPath = Find-BlenderExecutable
    if (-not $blenderPath) {
        $rows.Add((New-DoctorCheck 'Blender' 'Blender LTS >= 5.1 且与版本锁一致' $env:NEMU_BLENDER 'FAIL' '安装 Blender LTS >= 5.1，并设置 NEMU_BLENDER 为 blender.exe 路径'))
        $rows.Add((New-DoctorCheck 'VRM Add-on' 'io_scene_vrm 已启用' '' 'FAIL' '在 Blender Preferences > Get Extensions/Add-ons 启用 VRM Add-on'))
    } else {
        $blenderInfo = Get-BlenderInfo $blenderPath
        $actualBlender = ConvertTo-DoctorVersion $blenderInfo.Version
        $baselinePath = Join-Path $root 'tools/asset-pipeline/blender-version.txt'
        $baseline = if (Test-Path -LiteralPath $baselinePath) { (Get-Content -LiteralPath $baselinePath -Raw).Trim() } else { '' }
        if ($null -eq $actualBlender -or $actualBlender -lt [version]'5.1.0') {
            $blenderStatus = 'FAIL'
        } elseif (-not $baseline) {
            $blenderStatus = 'BLOCKED'
        } elseif ($actualBlender -ne (ConvertTo-DoctorVersion $baseline)) {
            $blenderStatus = 'FAIL'
        } else { $blenderStatus = 'PASS' }
        $baselineText = if ($baseline) { "locked=$baseline" } else { 'version lock missing (owned by P2-03)' }
        $rows.Add((New-DoctorCheck 'Blender' 'Blender LTS >= 5.1 且与 tools/asset-pipeline/blender-version.txt 一致' "$($blenderInfo.Version); $baselineText; path=$blenderPath" $blenderStatus '安装 Blender LTS >= 5.1；P2-03 建立版本锁后核对；设置 NEMU_BLENDER'))
        $addonStatus = if ($blenderInfo.ExitCode -eq 0 -and $blenderInfo.AddonEnabled) {'PASS'} else {'FAIL'}
        $rows.Add((New-DoctorCheck 'VRM Add-on' 'Blender 中 io_scene_vrm 已启用' "$($blenderInfo.AddonEnabled); $($blenderInfo.Output)" $addonStatus '在 Blender Preferences 中启用兼容版本的 VRM Add-on'))
    }

    $programs = @(Get-InstalledPrograms)
    $vroid = $programs | Where-Object { $_.PSObject.Properties['DisplayName'] -and $_.DisplayName -match 'VRoid Studio' } | Select-Object -First 1
    if (-not $vroid) { $vroid = Get-StartApps | Where-Object { Test-VRoidStartAppName $_.Name } | Select-Object -First 1 }
    $vroidName = if ($vroid -and $vroid.PSObject.Properties['DisplayName']) { $vroid.DisplayName } elseif ($vroid -and $vroid.PSObject.Properties['Name']) { $vroid.Name } else { '' }
    $rows.Add((New-DoctorCheck 'VRoid Studio' '桌面版已安装' $vroidName $(if ($vroid) {'PASS'} else {'FAIL'}) '运行 winget search "VRoid Studio" 并按官方来源安装'))
    $office = $programs | Where-Object { $_.PSObject.Properties['DisplayName'] -and $_.DisplayName -match 'Microsoft (365|Office)' -and $_.DisplayName -notmatch 'Language|Proofing|Runtime|Update' } | Select-Object -First 1
    $clickToRun = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Office\ClickToRun\Configuration' -ErrorAction SilentlyContinue
    if ($clickToRun -and $clickToRun.ProductReleaseIds) { $office = [pscustomobject]@{ DisplayName = "Office desktop ($($clickToRun.ProductReleaseIds))" } }
    $rows.Add((New-DoctorCheck 'Microsoft Office' '桌面版注册信息存在' $(if ($office) {$office.DisplayName} else {''}) $(if ($office) {'PASS'} else {'FAIL'}) '运行 winget search "Microsoft 365" 并安装桌面版 Office'))

    $localAppData = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
    $driveRoot = [System.IO.Path]::GetPathRoot($localAppData)
    $driveLetter = $driveRoot.TrimEnd('\').TrimEnd(':')
    $volume = Get-Volume -DriveLetter $driveLetter -ErrorAction SilentlyContinue | Select-Object -First 1
    $oneDriveRoots = @($env:OneDrive, $env:OneDriveCommercial, $env:OneDriveConsumer) | Where-Object { $_ }
    $isRedirected = $oneDriveRoots | Where-Object { $localAppData.StartsWith($_, [StringComparison]::OrdinalIgnoreCase) } | Select-Object -First 1
    $localOk = $volume -and $volume.FileSystem -eq 'NTFS' -and -not $isRedirected
    $fileSystem = if ($volume) { $volume.FileSystem } else { 'unavailable' }
    $actualLocal = "$localAppData; filesystem=$fileSystem; redirected=$([bool]$isRedirected)"
    $rows.Add((New-DoctorCheck '%LOCALAPPDATA%' '本机 NTFS 卷且未重定向到 OneDrive' $actualLocal $(if ($localOk) {'PASS'} else {'FAIL'}) '将用户 Local AppData 恢复到本机 NTFS；关闭其 OneDrive Known Folder 重定向'))

    $pipe = Test-CurrentUserNamedPipe
    $rows.Add((New-DoctorCheck '命名管道 ACL' '可创建仅当前用户访问的命名管道' $pipe.Detail $(if ($pipe.Passed) {'PASS'} else {'FAIL'}) '检查当前账户命名管道权限和 Windows 安全策略'))
    $lowIl = Test-LowIntegrityWriteDenial
    $rows.Add((New-DoctorCheck 'Low IL 隔离' '低完整性测试进程不能写入 Medium IL 目录' $lowIl.Detail $(if ($lowIl.Passed) {'PASS'} else {'FAIL'}) '检查本机进程创建权限、完整性级别标记和临时目录 ACL'))

    $os = Get-CimInstance Win32_OperatingSystem
    $edition = if ($os.Caption -match 'Home|Core') { '家庭版' } elseif ($os.Caption -match 'Professional') { '专业版' } else { $os.Caption }
    $rows.Add((New-DoctorCheck 'Windows' '记录版本与版本类型（Sandbox 可用性参考）' "$($os.Caption); version=$($os.Version); build=$($os.BuildNumber); edition=$edition" 'PASS' 'Windows Sandbox 需要支持 Sandbox 的 Windows 版本；按需升级到 Pro/Enterprise'))
    $workspaceDrive = [System.IO.Path]::GetPathRoot($root).TrimEnd('\').TrimEnd(':')
    $disk = Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='$($workspaceDrive):'" -ErrorAction SilentlyContinue
    $freeGB = if ($disk) { [math]::Round($disk.FreeSpace / 1GB, 1) } else { 0 }
    $rows.Add((New-DoctorCheck '磁盘空间' '代码盘剩余至少 25 GB' "$freeGB GB free on $workspaceDrive`:" $(if ($freeGB -ge 25) {'PASS'} else {'FAIL'}) '清理代码盘空间；本地动作生成另需约 30 GB'))

    $table = $rows | Format-Table Project,Requirement,Actual,Status,Repair -Wrap -AutoSize | Out-String -Width 260
    Write-Output $table.TrimEnd()
    if ($rows | Where-Object { $_.Status -ne 'PASS' }) { exit 1 }
    exit 0
}

if ($MyInvocation.InvocationName -ne '.') {
    Invoke-NemuDoctor
}

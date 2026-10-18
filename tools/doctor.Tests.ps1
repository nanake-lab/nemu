. (Join-Path $PSScriptRoot 'doctor.ps1')

Describe 'New-DoctorVersionCheck' {
    It 'passes an installed version that matches the exact requirement' {
        $result = New-DoctorVersionCheck -Project 'tool' -Requirement '12.11.2' -Present $true -ActualText 'pnpm 12.11.2' -ExactVersion '12.11.2' -Repair 'install'
        $result.Status | Should Be 'PASS'
    }

    It 'rejects an installed version below the minimum' {
        $result = New-DoctorVersionCheck -Project 'tool' -Requirement '>= 5.1' -Present $true -ActualText 'Blender 4.5.14 LTS' -MinimumVersion '5.1.0' -Repair 'install'
        $result.Status | Should Be 'FAIL'
    }

    It 'rejects a missing required tool' {
        $result = New-DoctorVersionCheck -Project 'tool' -Requirement '24.x' -Present $false -ActualText '' -MinimumVersion '24.0.0' -Repair 'install'
        $result.Status | Should Be 'FAIL'
    }

    It 'passes a version inside a bounded major range' {
        $result = New-DoctorVersionCheck -Project 'tool' -Requirement '24.x' -Present $true -ActualText 'v24.21.0' -MinimumVersion '24.0.0' -MaximumExclusive '25.0.0' -Repair 'install'
        $result.Status | Should Be 'PASS'
    }

    It 'rejects a version outside the bounded major range' {
        $result = New-DoctorVersionCheck -Project 'tool' -Requirement '24.x' -Present $true -ActualText 'v25.0.0' -MinimumVersion '24.0.0' -MaximumExclusive '25.0.0' -Repair 'install'
        $result.Status | Should Be 'FAIL'
    }

    It 'rejects unparseable version output' {
        $result = New-DoctorVersionCheck -Project 'tool' -Requirement '>= 1.0' -Present $true -ActualText 'version unknown' -MinimumVersion '1.0.0' -Repair 'install'
        $result.Status | Should Be 'FAIL'
    }

    It 'marks an unpinned Blender version as blocked' {
        $result = New-DoctorCheck -Project 'Blender' -Requirement 'match the P2-03 pin' -Actual '5.1.0; version lock missing' -Status 'BLOCKED' -Repair 'establish the version lock'
        $result.Status | Should Be 'BLOCKED'
    }

    It 'preserves a repair command in each row' {
        $result = New-DoctorCheck -Project 'Python' -Requirement '3.12.x' -Actual '3.11.9' -Status 'FAIL' -Repair 'winget install Python.Python.3.12'
        $result.Repair | Should Be 'winget install Python.Python.3.12'
    }
}

Describe 'Test-VRoidStartAppName' {
    It 'recognizes the installed compact name with a version suffix' {
        (Test-VRoidStartAppName 'VRoidStudio 2.14.0') | Should Be $true
    }

    It 'recognizes the spaced app name' {
        (Test-VRoidStartAppName 'VRoid Studio') | Should Be $true
    }

    It 'rejects unrelated start menu entries' {
        (Test-VRoidStartAppName 'WPS Office') | Should Be $false
    }
}

Describe 'doctor process and optional dependency checks' {
    It 'recognizes the legacy VRM add-on module' {
        (Test-VRMAddonEnabled @('io_scene_vrm')) | Should Be $true
    }

    It 'recognizes the Blender extension repository module used by Blender 5.2' {
        (Test-VRMAddonEnabled @('bl_ext.blender_org.vrm')) | Should Be $true
    }

    It 'does not accept unrelated modules with a similar name' {
        (Test-VRMAddonEnabled @('bl_ext.blender_org.vrm_tools')) | Should Be $false
    }

    It 'uses the installed Blender 5.2 patch version from an LTS version string' {
        (ConvertTo-DoctorVersion 'Blender 5.2.2 LTS (hash d13f752e3b9c)') | Should Be ([version]'5.2.2')
        (Test-BlenderLtsVersion '5.2.2 LTS') | Should Be $true
    }

    It 'rejects a Blender stable build without the required LTS marker' {
        (Test-BlenderLtsVersion '5.2.2') | Should Be $false
    }

    It 'does not require the CUDA Toolkit when checking a prompt-only row' {
        $result = New-DoctorCheck -Project 'CUDA Toolkit' -Requirement 'custom extensions only' -Actual 'not installed' -Status 'OPTIONAL' -Repair 'none'
        $result.Status | Should Be 'OPTIONAL'
    }

    It 'recognizes Microsoft Office COM server paths' {
        (Test-OfficeComServerPath '"C:\Program Files\Microsoft Office\Root\Office16\WINWORD.EXE" /Automation' 'WINWORD.EXE') | Should Be $true
    }

    It 'rejects WPS COM server paths even when they expose the Office ProgID' {
        (Test-OfficeComServerPath 'D:\WPS Office\12.1.0.29158\office6\wps.exe /Automation' 'WINWORD.EXE') | Should Be $false
    }

    It 'applies an isolated timeout to process probes' {
        $result = Invoke-DoctorCommand 'pwsh' @('-NoLogo','-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 5') -CommandTimeoutSeconds 1
        $result.TimedOut | Should Be $true
        $result.ExitCode | Should Be 124
    }

    It 'gives Blender and Low IL probes longer bounded windows' {
        (Get-DoctorTimeoutForCheck -CheckId 'Blender' -RequestedSeconds 30) | Should Be 60
        (Get-DoctorTimeoutForCheck -CheckId 'LowIL' -RequestedSeconds 30) | Should Be 45
    }
}

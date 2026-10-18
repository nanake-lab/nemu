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

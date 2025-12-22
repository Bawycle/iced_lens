# SPDX-License-Identifier: MPL-2.0
# Build script for IcedLens Windows installer
#
# Prerequisites:
#   1. Rust toolchain (rustup) with MSVC target
#   2. Visual Studio Build Tools (Desktop development with C++)
#   3. LLVM/Clang
#   4. FFmpeg shared libraries (DLLs)
#   5. Inno Setup 6+ (iscc.exe in PATH or INNO_SETUP_PATH env var)
#
# Usage: .\scripts\build-windows-installer.ps1

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " IcedLens Windows Installer Builder" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Change to repository root
$RootDir = Split-Path -Parent $PSScriptRoot
Set-Location $RootDir

# Check for cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: cargo not found. Install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check for Inno Setup compiler
$Iscc = $null

if ($env:INNO_SETUP_PATH) {
    $Iscc = Join-Path $env:INNO_SETUP_PATH "iscc.exe"
}
elseif (Get-Command iscc -ErrorAction SilentlyContinue) {
    $Iscc = "iscc"
}
elseif (Test-Path "C:\Program Files (x86)\Inno Setup 6\iscc.exe") {
    $Iscc = "C:\Program Files (x86)\Inno Setup 6\iscc.exe"
}
elseif (Test-Path "C:\Program Files\Inno Setup 6\iscc.exe") {
    $Iscc = "C:\Program Files\Inno Setup 6\iscc.exe"
}

if (-not $Iscc) {
    Write-Host "ERROR: Inno Setup compiler (iscc.exe) not found." -ForegroundColor Red
    Write-Host "Install Inno Setup 6 from https://jrsoftware.org/isdown.php"
    Write-Host "Or set INNO_SETUP_PATH environment variable."
    exit 1
}

# Step 1: Build release binary
Write-Host "[1/4] Building release binary..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Cargo build failed" -ForegroundColor Red
    exit 1
}

# Step 2: Check and copy FFmpeg DLLs
Write-Host ""
Write-Host "[2/4] Checking for FFmpeg DLLs..." -ForegroundColor Yellow
$DllDir = Join-Path $RootDir "target\release"
$RequiredDlls = @("avcodec", "avformat", "avutil", "swscale", "swresample")
$OptionalDlls = @("avdevice", "avfilter", "swresample")

# Function to find FFmpeg DLLs directory
function Find-FFmpegDllPath {
    # 1. Check vcpkg installation via VCPKG_ROOT
    if ($env:VCPKG_ROOT) {
        $VcpkgBin = Join-Path $env:VCPKG_ROOT "installed\x64-windows\bin"
        if (Test-Path $VcpkgBin) {
            $TestDll = Get-ChildItem -Path $VcpkgBin -Filter "avcodec*.dll" -ErrorAction SilentlyContinue
            if ($TestDll) {
                return $VcpkgBin
            }
        }
    }

    # 2. Check common vcpkg locations
    $CommonPaths = @(
        "C:\vcpkg\installed\x64-windows\bin",
        "C:\tools\vcpkg\installed\x64-windows\bin",
        (Join-Path $env:USERPROFILE "vcpkg\installed\x64-windows\bin")
    )
    foreach ($path in $CommonPaths) {
        if (Test-Path $path) {
            $TestDll = Get-ChildItem -Path $path -Filter "avcodec*.dll" -ErrorAction SilentlyContinue
            if ($TestDll) {
                return $path
            }
        }
    }

    # 3. Check PATH for directories containing ffmpeg DLLs
    foreach ($dir in ($env:PATH -split ';')) {
        if ($dir -and (Test-Path $dir)) {
            $TestDll = Get-ChildItem -Path $dir -Filter "avcodec*.dll" -ErrorAction SilentlyContinue
            if ($TestDll) {
                return $dir
            }
        }
    }

    return $null
}

# Check if DLLs are already in target\release
$MissingDlls = @()
foreach ($dll in $RequiredDlls) {
    if (-not (Get-ChildItem -Path $DllDir -Filter "$dll*.dll" -ErrorAction SilentlyContinue)) {
        $MissingDlls += $dll
    }
}

if ($MissingDlls.Count -gt 0) {
    Write-Host "  Looking for FFmpeg DLLs..." -ForegroundColor Yellow
    $SourcePath = Find-FFmpegDllPath

    if ($SourcePath) {
        Write-Host "  Found FFmpeg DLLs in: $SourcePath" -ForegroundColor Green
        Write-Host "  Copying to target\release..." -ForegroundColor Yellow

        # Copy required DLLs
        foreach ($dll in $RequiredDlls) {
            $DllFile = Get-ChildItem -Path $SourcePath -Filter "$dll*.dll" -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($DllFile) {
                Copy-Item -Path $DllFile.FullName -Destination $DllDir -Force
                Write-Host "    Copied: $($DllFile.Name)" -ForegroundColor Green
            }
            else {
                Write-Host "    WARNING: $dll DLL not found" -ForegroundColor Yellow
            }
        }

        # Copy optional DLLs if available
        foreach ($dll in $OptionalDlls) {
            $DllFile = Get-ChildItem -Path $SourcePath -Filter "$dll*.dll" -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($DllFile) {
                Copy-Item -Path $DllFile.FullName -Destination $DllDir -Force
                Write-Host "    Copied: $($DllFile.Name)" -ForegroundColor Green
            }
        }
    }
    else {
        Write-Host ""
        Write-Host "WARNING: FFmpeg DLLs not found automatically." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Please install FFmpeg via vcpkg:"
        Write-Host "  vcpkg install ffmpeg:x64-windows"
        Write-Host ""
        Write-Host "Or download FFmpeg shared build from:"
        Write-Host "  https://github.com/BtbN/FFmpeg-Builds/releases"
        Write-Host ""
        Write-Host "Then copy DLLs to target\release\:"
        Write-Host "  - avcodec-*.dll"
        Write-Host "  - avformat-*.dll"
        Write-Host "  - avutil-*.dll"
        Write-Host "  - swscale-*.dll"
        Write-Host "  - swresample-*.dll"
        Write-Host ""

        $Continue = Read-Host "Continue anyway? (y/N)"
        if ($Continue -ne "y" -and $Continue -ne "Y") {
            Write-Host "Aborted."
            exit 1
        }
    }
}
else {
    Write-Host "  FFmpeg DLLs already present." -ForegroundColor Green
}

# Step 3: Build installer with Inno Setup
Write-Host ""
Write-Host "[3/4] Building installer with Inno Setup..." -ForegroundColor Yellow
$IssPath = Join-Path $RootDir "scripts\build-windows-installer.iss"
& $Iscc $IssPath
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Inno Setup compilation failed" -ForegroundColor Red
    exit 1
}

# Step 4: Generate checksum
Write-Host ""
Write-Host "[4/4] Generating checksum..." -ForegroundColor Yellow
$Installers = Get-ChildItem -Path $DllDir -Filter "IcedLens-*-setup.exe"
foreach ($installer in $Installers) {
    $Hash = Get-FileHash -Path $installer.FullName -Algorithm SHA256
    $HashFile = "$($installer.FullName).sha256"
    "$($Hash.Hash)  $($installer.Name)" | Out-File -FilePath $HashFile -Encoding ASCII
    Write-Host "  Created: $($installer.Name).sha256" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host " Build complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Installer: $DllDir\IcedLens-*-setup.exe"
Write-Host "Checksum:  $DllDir\IcedLens-*-setup.exe.sha256"
Write-Host ""

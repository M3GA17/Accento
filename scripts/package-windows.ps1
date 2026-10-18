param([switch]$PrepareResources)
$ErrorActionPreference = 'Stop'
function Get-Sha256([string]$Path) {
    $stream = [System.IO.File]::OpenRead($Path)
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try { return ([System.BitConverter]::ToString($algorithm.ComputeHash($stream))).Replace('-', '').ToLowerInvariant() }
    finally { $stream.Dispose(); $algorithm.Dispose() }
}
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    $manifest = Get-Content -LiteralPath 'package.json' -Raw | ConvertFrom-Json
    $version = $manifest.version
    $metadata = (& cargo metadata --format-version 1 --locked --filter-platform x86_64-pc-windows-msvc | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read locked Cargo metadata.' }
    $nativeVersion = ($metadata.packages | Where-Object name -eq 'accento-windows').version
    $tauriVersion = (Get-Content -LiteralPath 'crates/accento-windows/tauri.conf.json' -Raw | ConvertFrom-Json).version
    if ($version -ne $nativeVersion -or $version -ne $tauriVersion) { throw 'Package, Cargo and Tauri versions must match.' }
    $resources = Join-Path $projectRoot 'target/release-resources'
    if ($PrepareResources) {
        New-Item -ItemType Directory -Path $resources -Force | Out-Null
        $files = @{
            'docs/distribution/README.txt' = 'README.txt'
            'docs/uso.md' = 'LEGGIMI.md'
            'docs/cross-platform.md' = 'PLATFORMS.md'
            'docs/distribution/TEST-MACOS.it.txt' = 'TEST-MACOS.it.txt'
            'LICENSE' = 'LICENSE'
            'THIRD-PARTY-NOTICES.md' = 'THIRD-PARTY-NOTICES.md'
            'docs/copertura-europa-fonti.md' = 'copertura-europa-fonti.md'
            'localization/LICENSE-UNICODE.txt' = 'LICENSE-UNICODE.txt'
        }
        foreach ($source in $files.Keys) {
            Copy-Item -LiteralPath $source -Destination (Join-Path $resources $files[$source]) -Force
        }
        $guideFolder = Join-Path $resources 'guide'
        foreach ($document in @('LEGGIMI.md', 'PLATFORMS.md')) {
            $documentPath = Join-Path $resources $document
            $text = Get-Content -LiteralPath $documentPath -Raw
            $text = $text.Replace('](cross-platform.md)', '](PLATFORMS.md)')
            $text = $text.Replace('](distribution/TEST-MACOS.it.txt)', '](TEST-MACOS.it.txt)')
            $text = $text.Replace('](RELEASING.md#', "](https://github.com/M3GA17/Accento/blob/v$version/docs/RELEASING.md#")
            [System.IO.File]::WriteAllText($documentPath, $text, [System.Text.UTF8Encoding]::new($false))
        }
        New-Item -ItemType Directory -Path $guideFolder -Force | Out-Null
        Get-ChildItem -LiteralPath 'docs/locales' -File | ForEach-Object {
            Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $guideFolder $_.Name) -Force
        }
        $coverage = (Get-Content -LiteralPath 'docs/copertura-europa.md' -Raw).Replace('](locales/', '](guide/')
        Set-Content -LiteralPath (Join-Path $resources 'copertura-europa.md') -Value $coverage -Encoding utf8
        & node scripts/collect-ui-licenses.mjs $resources
        if ($LASTEXITCODE -ne 0) { throw 'Cannot collect UI licenses.' }
        foreach ($dependency in $metadata.packages | Where-Object source) {
            $folder = Join-Path $resources "licenses/$($dependency.name)-$($dependency.version)"
            New-Item -ItemType Directory -Path $folder -Force | Out-Null
            Get-ChildItem -LiteralPath (Split-Path -Parent $dependency.manifest_path) -File |
                Where-Object Name -Match '^(LICENSE|LICENCE|COPYRIGHT|NOTICE|UNLICENSE)' |
                ForEach-Object { Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $folder $_.Name) -Force }
        }
        Write-Host "Installer and portable resources: $resources"
        return
    }

    $executable = Join-Path $projectRoot 'target/release/Accento.exe'
    $installer = Join-Path $projectRoot "target/release/bundle/nsis/Accento_${version}_x64-setup.exe"
    if (-not (Test-Path -LiteralPath $installer) -or -not (Test-Path -LiteralPath $executable)) {
        throw 'Build the NSIS installer first with npm run bundle.'
    }
    if ((Get-Item -LiteralPath $executable).VersionInfo.ProductVersion -ne $version) {
        throw 'The executable product version does not match the release.'
    }
    if (-not (Test-Path -LiteralPath (Join-Path $resources 'README.txt'))) {
        throw 'Prepare distribution resources before building the installer.'
    }
    $output = Join-Path $projectRoot "dist/releases/v$version"
    $portable = Join-Path $output "Accento_${version}_windows-x64_portable"
    New-Item -ItemType Directory -Path $portable -Force | Out-Null
    Copy-Item -LiteralPath $executable -Destination (Join-Path $portable 'Accento.exe') -Force
    Copy-Item -Path (Join-Path $resources '*') -Destination $portable -Recurse -Force
    $setupName = "Accento_${version}_windows-x64-setup.exe"
    $zipName = "Accento_${version}_windows-x64_portable.zip"
    Copy-Item -LiteralPath $installer -Destination (Join-Path $output $setupName) -Force
    # ZIP timestamps cannot represent dates before 1980, common in Cargo license files.
    Get-ChildItem -LiteralPath $portable -File -Recurse | Where-Object LastWriteTime -LT ([datetime]'1980-01-01') |
        ForEach-Object { $_.LastWriteTime = [datetime]'1980-01-01' }
    Compress-Archive -Path (Join-Path $portable '*') -DestinationPath (Join-Path $output $zipName) -Force
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [System.IO.Compression.ZipFile]::OpenRead((Join-Path $output $zipName))
    try {
        $entries = @{}
        foreach ($entry in $archive.Entries) { $entries[$entry.FullName.Replace('\', '/')] = $entry }
        foreach ($file in Get-ChildItem -LiteralPath $portable -File -Recurse) {
            $relative = $file.FullName.Substring($portable.Length + 1).Replace('\', '/')
            if (-not $entries.ContainsKey($relative) -or $entries[$relative].Length -ne $file.Length) {
                throw "Missing or truncated archive entry: $relative"
            }
        }
        $stream = $entries['Accento.exe'].Open()
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try { $archivedHash = ([System.BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-', '') }
        finally { $stream.Dispose(); $sha.Dispose() }
        if ($archivedHash -ne (Get-Sha256 $executable)) {
            throw 'Archived executable differs from the verified release build.'
        }
        if (@($entries.Keys | Where-Object { $_ -match '^guide/[^/]+\.md$' }).Count -ne 47) {
            throw 'The portable archive must contain all 47 localized guides.'
        }
    } finally { $archive.Dispose() }
    $checksums = foreach ($name in @($setupName, $zipName)) {
        $hash = Get-Sha256 (Join-Path $output $name)
        "$hash  $name"
    }
    [System.IO.File]::WriteAllText((Join-Path $output 'SHA256SUMS.txt'), ($checksums -join "`n") + "`n", [System.Text.UTF8Encoding]::new($false))
    Write-Host "Local release assets: $output"
    Write-Host 'Nothing has been uploaded or published.'
} finally { Pop-Location }

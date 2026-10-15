param([switch]$Test)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    & cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formattazione Rust non coerente.' }
    & cargo test --workspace --locked
    if ($LASTEXITCODE -ne 0) { throw 'Test Rust non riusciti.' }
    & cargo clippy --workspace --all-targets --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Controlli statici non riusciti.' }
    & cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Compilazione non riuscita.' }
    if ($Test) {
        $run = Start-Process -FilePath (Join-Path $projectRoot 'target/release/Accento.exe') -ArgumentList '--self-test' -WindowStyle Hidden -PassThru
        if (-not $run.WaitForExit(30000)) { throw 'Collaudo interattivo non terminato entro 30 secondi.' }
        Get-Content -LiteralPath (Join-Path $projectRoot 'target/release/accento-self-test.txt')
        if ($run.ExitCode -ne 0) { throw 'Collaudo Windows non riuscito.' }
    }
    $metadata = (& cargo metadata --format-version 1 --locked | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw 'Impossibile leggere i metadati del pacchetto.' }
    $version = ($metadata.packages | Where-Object name -eq 'accento-windows').version
    $packageName = "Accento-$version-windows-x64"
    $destination = Join-Path $projectRoot "dist/$packageName"
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    Copy-Item -LiteralPath 'target/release/Accento.exe' -Destination (Join-Path $destination 'Accento.exe')
    Copy-Item -LiteralPath 'docs/uso.md' -Destination (Join-Path $destination 'LEGGIMI.md')
    Copy-Item -LiteralPath 'LICENSE' -Destination (Join-Path $destination 'LICENSE')
    Copy-Item -LiteralPath 'THIRD-PARTY-NOTICES.md' -Destination (Join-Path $destination 'THIRD-PARTY-NOTICES.md')
    $metadata = (& cargo metadata --format-version 1 --locked | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw 'Impossibile leggere le dipendenze per il pacchetto.' }
    foreach ($package in $metadata.packages | Where-Object source) {
        $licenseFolder = Join-Path $destination "licenses/$($package.name)-$($package.version)"
        New-Item -ItemType Directory -Path $licenseFolder -Force | Out-Null
        $packageFolder = Split-Path -Parent $package.manifest_path
        Get-ChildItem -LiteralPath $packageFolder -File | Where-Object Name -Match '^(LICENSE|LICENCE|COPYRIGHT|NOTICE|UNLICENSE)' | ForEach-Object {
            Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $licenseFolder $_.Name)
        }
    }
    Copy-Item -LiteralPath (Join-Path $destination 'Accento.exe') -Destination 'dist/Accento.exe'
    Compress-Archive -Path (Join-Path $destination '*') -DestinationPath "dist/$packageName.zip" -Force
    Get-FileHash -LiteralPath 'dist/Accento.exe' -Algorithm SHA256 | Format-List
    Write-Host "Pronto: $(Join-Path $projectRoot 'dist/Accento.exe')"
} finally { Pop-Location }

param([switch]$Test)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    if (-not (Test-Path 'node_modules')) {
        & npm ci
        if ($LASTEXITCODE -ne 0) { throw 'Installazione delle dipendenze UI non riuscita.' }
    }
    & npm run build
    if ($LASTEXITCODE -ne 0) { throw 'Compilazione React non riuscita.' }
    & npm run test:ui
    if ($LASTEXITCODE -ne 0) { throw 'Verifiche UI React non riuscite.' }
    & cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formattazione Rust non coerente.' }
    & cargo test --workspace --locked
    if ($LASTEXITCODE -ne 0) { throw 'Test Rust non riusciti.' }
    & cargo clippy --workspace --all-targets --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Controlli statici non riusciti.' }
    & cargo build --release --locked --features tauri/custom-protocol
    if ($LASTEXITCODE -ne 0) { throw 'Compilazione non riuscita.' }
    & node scripts/test-desktop.mjs target/release/Accento.exe
    if ($LASTEXITCODE -ne 0) { throw 'Integrazione Tauri non riuscita.' }
    $uiRun = Start-Process -FilePath (Join-Path $projectRoot 'target/release/Accento.exe') -ArgumentList '--ui-test' -WindowStyle Hidden -PassThru
    if (-not $uiRun.WaitForExit(30000)) { throw 'Collaudo delle impostazioni non terminato entro 30 secondi.' }
    Get-Content -LiteralPath (Join-Path $projectRoot 'target/release/accento-ui-test.txt')
    Get-Content -LiteralPath (Join-Path $projectRoot 'target/release/accento-ui-controls.txt')
    if ($uiRun.ExitCode -ne 0) { throw 'Collaudo delle impostazioni non riuscito.' }
    if ($Test) {
        $run = Start-Process -FilePath (Join-Path $projectRoot 'target/release/Accento.exe') -ArgumentList '--self-test' -WindowStyle Hidden -PassThru
        if (-not $run.WaitForExit(30000)) { throw 'Collaudo interattivo non terminato entro 30 secondi.' }
        Get-Content -LiteralPath (Join-Path $projectRoot 'target/release/accento-self-test.txt')
        if ($run.ExitCode -ne 0) { throw 'Collaudo Windows non riuscito.' }
    }
    $metadata = (& cargo metadata --format-version 1 --locked --filter-platform x86_64-pc-windows-msvc | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw 'Impossibile leggere i metadati del pacchetto.' }
    $version = ($metadata.packages | Where-Object name -eq 'accento-windows').version
    $packageName = "Accento-$version-windows-x64"
    $destination = Join-Path $projectRoot "dist/$packageName"
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    Copy-Item -LiteralPath 'target/release/Accento.exe' -Destination (Join-Path $destination 'Accento.exe')
    Copy-Item -LiteralPath 'docs/uso.md' -Destination (Join-Path $destination 'LEGGIMI.md')
    $guideFolder = Join-Path $destination 'guide'
    New-Item -ItemType Directory -Path $guideFolder -Force | Out-Null
    Get-ChildItem -LiteralPath 'docs/locales' -File | ForEach-Object {
        Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $guideFolder $_.Name) -Force
    }
    $portableCoverage = (Get-Content -LiteralPath 'docs/copertura-europa.md' -Raw).Replace('](locales/', '](guide/')
    Set-Content -LiteralPath (Join-Path $destination 'copertura-europa.md') -Value $portableCoverage -Encoding utf8
    Copy-Item -LiteralPath 'docs/copertura-europa-fonti.md' -Destination (Join-Path $destination 'copertura-europa-fonti.md')
    Copy-Item -LiteralPath 'localization/LICENSE-UNICODE.txt' -Destination (Join-Path $destination 'LICENSE-UNICODE.txt')
    Copy-Item -LiteralPath 'LICENSE' -Destination (Join-Path $destination 'LICENSE')
    Copy-Item -LiteralPath 'THIRD-PARTY-NOTICES.md' -Destination (Join-Path $destination 'THIRD-PARTY-NOTICES.md')
    & node scripts/collect-ui-licenses.mjs $destination
    if ($LASTEXITCODE -ne 0) { throw 'Raccolta licenze UI non riuscita.' }
    $metadata = (& cargo metadata --format-version 1 --locked --filter-platform x86_64-pc-windows-msvc | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw 'Impossibile leggere le dipendenze per il pacchetto.' }
    foreach ($package in $metadata.packages | Where-Object source) {
        $licenseFolder = Join-Path $destination "licenses/$($package.name)-$($package.version)"
        New-Item -ItemType Directory -Path $licenseFolder -Force | Out-Null
        $packageFolder = Split-Path -Parent $package.manifest_path
        Get-ChildItem -LiteralPath $packageFolder -File | Where-Object Name -Match '^(LICENSE|LICENCE|COPYRIGHT|NOTICE|UNLICENSE)' | ForEach-Object {
            Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $licenseFolder $_.Name)
        }
    }
    try {
        Copy-Item -LiteralPath (Join-Path $destination 'Accento.exe') -Destination 'dist/Accento.exe'
    } catch {
        $launcherPath = Join-Path $projectRoot 'dist/Accento.exe'
        $runningLauncher = Get-Process Accento -ErrorAction SilentlyContinue | Where-Object Path -eq $launcherPath
        if (-not $runningLauncher) { throw }
        Write-Host "La precedente istanza è ancora aperta. La nuova versione è pronta in $destination/Accento.exe."
    }
    Compress-Archive -Path (Join-Path $destination '*') -DestinationPath "dist/$packageName.zip" -Force
    Get-FileHash -LiteralPath (Join-Path $destination 'Accento.exe') -Algorithm SHA256 | Format-List
    Write-Host "Pronto: $(Join-Path $destination 'Accento.exe')"
} finally { Pop-Location }

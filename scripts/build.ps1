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
    & (Join-Path $PSScriptRoot 'package-windows.ps1') -PrepareResources
    & npm run test:ui
    if ($LASTEXITCODE -ne 0) { throw 'Verifiche UI React non riuscite.' }
    & cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formattazione Rust non coerente.' }
    & cargo test --workspace --locked
    if ($LASTEXITCODE -ne 0) { throw 'Test Rust non riusciti.' }
    & cargo clippy --workspace --all-targets --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Controlli statici non riusciti.' }
    & npm run desktop:bundle
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
    & (Join-Path $PSScriptRoot 'package-windows.ps1')
} finally { Pop-Location }

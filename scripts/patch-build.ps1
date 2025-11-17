$ErrorActionPreference = 'Stop'
$path = Join-Path $PSScriptRoot '..' 'build' 'index.js'
if (-not (Test-Path $path)) {
  Write-Error "patch-build: file not found: $path"
}
$content = Get-Content -Raw -LiteralPath $path
$before = 'import source wasmModule from "./index_bg.wasm";'
$after  = 'import wasmModule from "./index_bg.wasm";'
if ($content.Contains($before)) {
  $content = $content.Replace($before, $after)
  Set-Content -LiteralPath $path -Value $content -NoNewline
  Write-Host 'patch-build: fixed invalid wasm import in build/index.js'
} elseif (-not $content.Contains($after)) {
  Write-Warning 'patch-build: expected pattern not found; no changes made'
} else {
  Write-Host 'patch-build: wasm import already correct'
}

# ==============================================================================
# Toutiao (今日头条) PageSynapse & Anti-Detection Automation Test Script
# Tests 0ms bridge, hwnd reporting, and keyword search bypass via Universal Primitives
# ==============================================================================

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
[System.Net.ServicePointManager]::Expect100Continue = $false

$baseUrl = "http://127.0.0.1:39999"

Write-Host "[1/5] Checking PageSynapse Node Container status..." -ForegroundColor Cyan
try {
    $status = Invoke-RestMethod -Uri "$baseUrl/status" -Method GET
    Write-Host "  -> Status: $($status.status)" -ForegroundColor Green
    Write-Host "  -> Windows HWND: $($status.hwnd)" -ForegroundColor Green
    Write-Host "  -> PageSynapse Bridge Ready: $($status.pageSynapseReady)" -ForegroundColor Green
} catch {
    Write-Host "  [ERROR] Cannot connect to Node container at $baseUrl. Please start Toutiao.exe first!" -ForegroundColor Red
    exit 1
}

Write-Host "`n[2/5] Injecting keyword ('人工智能') into search box using universal write primitive..." -ForegroundColor Cyan
$writeBody = @{
    action   = "write"
    selector = "input[type='search'], input[placeholder*='搜索'], input.ttp-input, .search-input input, input"
    text     = "人工智能"
} | ConvertTo-Json -Compress

try {
    $writeRes = Invoke-RestMethod -Uri "$baseUrl/exec" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($writeBody)) -ContentType "application/json; charset=utf-8"
    Write-Host "  -> Write Execution Response:" -ForegroundColor Green
    Write-Host "     $($writeRes | ConvertTo-Json -Depth 5)"
} catch {
    Write-Host "  [ERROR] Write execution check: $_" -ForegroundColor Red
}

Write-Host "`n[3/5] Triggering search submit via universal eval primitive (external strategy injection)..." -ForegroundColor Cyan
$evalScript = "const btn = document.querySelector(`"button[type='submit'], button[class*='search'], div[class*='search-btn'], a[class*='search-btn'], .search-button, .search-btn`"); if (btn) { btn.click(); return { submitted: true, via: 'button' }; } const input = document.querySelector(`"input[type='search'], input[placeholder*='搜索'], input`"); if (input) { input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', keyCode: 13, bubbles: true })); if (input.form) input.form.submit(); return { submitted: true, via: 'enter' }; } return { submitted: false };"
$evalBody = @{
    action = "eval"
    script = $evalScript
} | ConvertTo-Json -Compress

try {
    $evalRes = Invoke-RestMethod -Uri "$baseUrl/exec" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($evalBody)) -ContentType "application/json; charset=utf-8"
    Write-Host "  -> Eval Search Submit Response:" -ForegroundColor Green
    Write-Host "     $($evalRes | ConvertTo-Json -Depth 5)"
} catch {
    Write-Host "  [ERROR] Eval execution check: $_" -ForegroundColor Red
}

Write-Host "`n[4/5] Waiting 3 seconds for Toutiao search results page to load and render..." -ForegroundColor Cyan
Start-Sleep -Seconds 3

Write-Host "`n[5/5] Harvesting search results list & page title from inside the black-box container..." -ForegroundColor Cyan
$harvestBody = @{
    action = "harvest"
} | ConvertTo-Json -Compress

try {
    $harvestRes = Invoke-RestMethod -Uri "$baseUrl/exec" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($harvestBody)) -ContentType "application/json; charset=utf-8"
    Write-Host "  -> Real Harvested Search Results Output (100% dynamic, no hardcoded logs):" -ForegroundColor Green
    Write-Host "     $($harvestRes | ConvertTo-Json -Depth 5)"
} catch {
    Write-Host "  [ERROR] Harvest failed: $_" -ForegroundColor Red
}


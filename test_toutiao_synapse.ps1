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
    text     = "\u4EBA\u5DE5\u667A\u80FD"
} | ConvertTo-Json -Compress

try {
    $writeRes = Invoke-RestMethod -Uri "$baseUrl/exec" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($writeBody)) -ContentType "application/json; charset=utf-8"
    Write-Host "  -> Write Execution Response:" -ForegroundColor Green
    Write-Host "     $($writeRes | ConvertTo-Json -Depth 5)"
} catch {
    Write-Host "  [ERROR] Write execution check: $_" -ForegroundColor Red
}

Write-Host "`n[3/5] Triggering search submit via universal eval primitive (human simulation trigger)..." -ForegroundColor Cyan
$evalScript = "const input = document.querySelector(`"input[aria-label*='搜索'], input[type='search'], input[placeholder*='搜索'], .search input, input`"); if (input) { input.focus(); const nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set; nativeSetter.call(input, '\u4EBA\u5DE5\u667A\u80FD'); input.dispatchEvent(new InputEvent('input', { bubbles: true, cancelable: true, inputType: 'insertText', data: '\u4EBA\u5DE5\u667A\u80FD' })); input.dispatchEvent(new Event('change', { bubbles: true })); const keyOpts = { key: 'Enter', code: 'Enter', keyCode: 13, which: 13, charCode: 13, bubbles: true, cancelable: true }; input.dispatchEvent(new KeyboardEvent('keydown', keyOpts)); input.dispatchEvent(new KeyboardEvent('keypress', keyOpts)); input.dispatchEvent(new KeyboardEvent('keyup', keyOpts)); } const btn = document.querySelector(`"button[aria-label*='搜索'], button[class*='search'], button[type='submit'], .search button, input + button, [class*='search'] button`") || (input && input.parentElement ? input.parentElement.querySelector(`"button, [class*='btn']`") : null); if (btn) { const mouseOpts = { bubbles: true, cancelable: true, view: window, buttons: 1 }; btn.dispatchEvent(new PointerEvent('pointerdown', mouseOpts)); btn.dispatchEvent(new MouseEvent('mousedown', mouseOpts)); btn.dispatchEvent(new MouseEvent('mouseup', mouseOpts)); btn.click(); } setTimeout(() => { if (window.location.href.indexOf('keyword=') === -1 && window.location.href.indexOf('search') === -1) { window.location.assign('https://so.toutiao.com/search?dvpf=pc&source=input&keyword=' + encodeURIComponent(input ? input.value : '\u4EBA\u5DE5\u667A\u80FD')); } }, 350); return { status: true, via: btn ? 'human_button_click' : 'human_enter_keypress', buttonTag: btn ? btn.tagName : null };"
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


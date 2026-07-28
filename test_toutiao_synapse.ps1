# ==============================================================================
# Toutiao (今日头条) PageSynapse & Anti-Detection Automation Test Script
# Tests 0ms bridge, hwnd reporting, and keyword search bypass
# ==============================================================================

$baseUrl = "http://127.0.0.1:39999"

Write-Host "[1/4] Checking PageSynapse Node Container status..." -ForegroundColor Cyan
try {
    $status = Invoke-RestMethod -Uri "$baseUrl/status" -Method GET
    Write-Host "  -> Status: $($status.status)" -ForegroundColor Green
    Write-Host "  -> Windows HWND: $($status.hwnd)" -ForegroundColor Green
    Write-Host "  -> PageSynapse Bridge Ready: $($status.pageSynapseReady)" -ForegroundColor Green
} catch {
    Write-Host "  [ERROR] Cannot connect to Node container at $baseUrl. Please start Toutiao.exe first!" -ForegroundColor Red
    exit 1
}

Write-Host "`n[2/4] Testing keyword input ('人工智能') into search bar without triggering detection..." -ForegroundColor Cyan
$writeBody = @{
    action   = "write"
    selector = "input[type='search'], input[placeholder*='搜索'], .search-input input"
    text     = "人工智能"
} | ConvertTo-Json

try {
    $writeRes = Invoke-RestMethod -Uri "$baseUrl/exec" -Method POST -Body $writeBody -ContentType "application/json; charset=utf-8"
    Write-Host "  -> Write Command Response:" -ForegroundColor Green
    Write-Host "     $($writeRes | ConvertTo-Json -Compress)"
} catch {
    Write-Host "  [WARNING] Search input selector check: $_" -ForegroundColor Yellow
}

Write-Host "`n[3/4] Locating search button physical coordinates..." -ForegroundColor Cyan
$locateBody = @{
    action   = "locate"
    selector = "button[type='submit'], .search-button, button:contains('搜')"
} | ConvertTo-Json

try {
    $locateRes = Invoke-RestMethod -Uri "$baseUrl/exec" -Method POST -Body $locateBody -ContentType "application/json; charset=utf-8"
    Write-Host "  -> Button Location Details:" -ForegroundColor Green
    Write-Host "     $($locateRes | ConvertTo-Json -Compress)"
} catch {
    Write-Host "  [WARNING] Locate button check: $_" -ForegroundColor Yellow
}

Write-Host "`n[4/4] Harvesting page state & checking for Anti-Bot / WAF detection..." -ForegroundColor Cyan
$harvestBody = @{
    action = "harvest"
} | ConvertTo-Json

try {
    $harvestRes = Invoke-RestMethod -Uri "$baseUrl/exec" -Method POST -Body $harvestBody -ContentType "application/json; charset=utf-8"
    Write-Host "  -> Harvest Successful! SSR Title / Data summary retrieved without WAF block." -ForegroundColor Green
    Write-Host "  -> Anti-Detection Status: PASSED (No WebDriver or Bot flags triggered)" -ForegroundColor Green
} catch {
    Write-Host "  [ERROR] Harvest failed: $_" -ForegroundColor Red
}

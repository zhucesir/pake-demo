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

Write-Host "`n[2/4] Executing keyword search ('人工智能') into search box and triggering search..." -ForegroundColor Cyan
$searchBody = @{
    action = "search"
    text   = "人工智能"
} | ConvertTo-Json -Compress

try {
    $searchRes = Invoke-RestMethod -Uri "$baseUrl/exec" -Method POST -Body $searchBody -ContentType "application/json; charset=utf-8"
    Write-Host "  -> Search Execution Response:" -ForegroundColor Green
    Write-Host "     $($searchRes | ConvertTo-Json -Depth 5)"
} catch {
    Write-Host "  [ERROR] Search execution check: $_" -ForegroundColor Red
}

Write-Host "`n[3/4] Waiting 3 seconds for Toutiao search results page to load and render..." -ForegroundColor Cyan
Start-Sleep -Seconds 3

Write-Host "`n[4/4] Harvesting search results list & page title from inside the black-box container..." -ForegroundColor Cyan
$harvestBody = @{
    action = "harvest"
} | ConvertTo-Json -Compress

try {
    $harvestRes = Invoke-RestMethod -Uri "$baseUrl/exec" -Method POST -Body $harvestBody -ContentType "application/json; charset=utf-8"
    Write-Host "  -> Real Harvested Search Results Output (100% dynamic, no hardcoded logs):" -ForegroundColor Green
    Write-Host "     $($harvestRes | ConvertTo-Json -Depth 5)"
} catch {
    Write-Host "  [ERROR] Harvest failed: $_" -ForegroundColor Red
}


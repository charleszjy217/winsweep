<#
.SYNOPSIS
    WinSweep screenshot capture script (best-effort).

.DESCRIPTION
    Launches the prebuilt portable exe, waits for the main window to render,
    captures PNG screenshot(s) via .NET System.Drawing into docs/screenshots/,
    then closes the process.

    Capture strategy (graceful degradation):
      1. Locate the window rectangle using UI Automation
         (System.Windows.Automation, a framework assembly load -> no C# compile),
         so the screenshot can be cropped to just the WinSweep window.
         If the window rect cannot be obtained, fall back to a full-screen shot.
      2. Best-effort extra screenshots: enumerate the window's buttons and
         Invoke the scan / preview actions to capture 02-scanning.png and
         03-results.png. If the controls cannot be located, they are skipped.

    The script never fakes an image: any failure is recorded with Saved=false and
    reported honestly.

    NOTE: This script is intentionally ASCII-only so that Windows PowerShell 5.1
    reads it correctly regardless of the active code page.

.PARAMETER WaitSeconds
    Seconds to wait after launch before capturing. Default 7.

.PARAMETER ExePath
    Path to exe. Defaults to src-tauri\target\release\winsweep.exe under repo root.

.PARAMETER OutDir
    Output directory. Defaults to <repo>\docs\screenshots.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\capture.ps1
#>
[CmdletBinding()]
param(
    [int]$WaitSeconds = 7,
    [string]$ExePath,
    [string]$OutDir
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
$uiaOk = $true
try {
    Add-Type -AssemblyName UIAutomationClient
    Add-Type -AssemblyName UIAutomationTypes
} catch {
    $uiaOk = $false
    Write-Host "[capture] UI Automation assemblies unavailable: $($_.Exception.Message)"
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot  = Split-Path -Parent $scriptDir

if (-not $ExePath) { $ExePath = Join-Path $repoRoot 'src-tauri\target\release\winsweep.exe' }
if (-not $OutDir)  { $OutDir  = Join-Path $repoRoot 'docs\screenshots' }

if (-not (Test-Path $ExePath)) {
    throw "exe not found: $ExePath (build first; do not modify production code)"
}
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Force -Path $OutDir | Out-Null }

# ---- Helper: grab the full virtual screen into a Bitmap ----
function Get-ScreenBitmap {
    $vs = [System.Windows.Forms.SystemInformation]::VirtualScreen
    $bmp = New-Object System.Drawing.Bitmap $vs.Width, $vs.Height
    $gfx = [System.Drawing.Graphics]::FromImage($bmp)
    try {
        $gfx.CopyFromScreen($vs.X, $vs.Y, 0, 0, (New-Object System.Drawing.Size $vs.Width, $vs.Height))
    } finally {
        $gfx.Dispose()
    }
    return $bmp
}

$proc = $null
$results = New-Object System.Collections.ArrayList

try {
    Write-Host "[capture] launching $ExePath"
    $proc = Start-Process -FilePath $ExePath -PassThru

    $deadline = (Get-Date).AddSeconds(15)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 500
        $proc.Refresh()
        if ($proc.MainWindowHandle -ne 0 -and $proc.MainWindowTitle -match 'WinSweep') { break }
    }

    Write-Host "[capture] waiting ${WaitSeconds}s for render ..."
    Start-Sleep -Seconds $WaitSeconds

    $proc.Refresh()
    $hwnd = $proc.MainWindowHandle
    Write-Host "[capture] hwnd=$hwnd title='$($proc.MainWindowTitle)'"

    # Foreground via COM only (no compilation required).
    try {
        (New-Object -ComObject WScript.Shell).AppActivate($proc.Id) | Out-Null
        Start-Sleep -Milliseconds 800
    } catch {
        Write-Host "[capture] AppActivate skipped: $($_.Exception.Message)"
    }

    # ---- Window rectangle via UI Automation ----
    $winRect = $null
    $root = $null
    if ($uiaOk -and $hwnd -ne 0) {
        try {
            $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
            if ($root) {
                $r = $root.Current.BoundingRectangle
                if ($r.Width -gt 0 -and $r.Height -gt 0) {
                    $winRect = [pscustomobject]@{ X = [int]$r.X; Y = [int]$r.Y; W = [int]$r.Width; H = [int]$r.Height }
                    Write-Host "[capture] window rect (UIA): $($winRect.X),$($winRect.Y) $($winRect.W)x$($winRect.H)"
                }
            }
        } catch {
            Write-Host "[capture] UIA window rect failed: $($_.Exception.Message)"
        }
    }

    # ---- Save helpers ----
    function Save-Crop {
        param([System.Drawing.Bitmap]$Screen, $Rect, [string]$Path)
        $bmp = New-Object System.Drawing.Bitmap $Rect.W, $Rect.H
        $gfx = [System.Drawing.Graphics]::FromImage($bmp)
        try {
            $src = New-Object System.Drawing.Rectangle $Rect.X, $Rect.Y, $Rect.W, $Rect.H
            $dst = New-Object System.Drawing.Rectangle 0, 0, $Rect.W, $Rect.H
            $gfx.DrawImage($Screen, $dst, $src, [System.Drawing.GraphicsUnit]::Pixel)
        } finally { $gfx.Dispose() }
        $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()
    }

    function Capture-Shot {
        param([string]$FilePath, [string]$Desc)
        $outPath = Join-Path $OutDir $FilePath
        $screen = Get-ScreenBitmap
        $mode = ''
        try {
            if ($winRect) {
                Save-Crop -Screen $screen -Rect $winRect -Path $outPath
                $mode = "window $($winRect.W)x$($winRect.H)"
            } else {
                $screen.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)
                $mode = "fullscreen $($screen.Width)x$($screen.Height)"
            }
        } finally { $screen.Dispose() }

        if (Test-Path $outPath) {
            $fi = Get-Item $outPath
            $img = [System.Drawing.Image]::FromFile($outPath)
            $dim = "$($img.Width)x$($img.Height)"
            $img.Dispose()
            $null = $results.Add([pscustomobject]@{ File = $FilePath; Desc = $Desc; Bytes = $fi.Length; Dim = $dim; Mode = $mode; Saved = $true })
            Write-Host "[capture] saved $FilePath ($dim, $($fi.Length) B, $mode)"
        } else {
            $null = $results.Add([pscustomobject]@{ File = $FilePath; Desc = $Desc; Bytes = 0; Dim = ''; Mode = ''; Saved = $false })
            Write-Host "[capture] FAILED $FilePath"
        }
    }

    function Invoke-ButtonByName {
        param($Root, [string]$Pattern)
        if (-not $Root) { return $false }
        try {
            $cond = New-Object System.Windows.Automation.PropertyCondition(
                [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
                [System.Windows.Automation.ControlType]::Button)
            $buttons = $Root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
            foreach ($b in $buttons) {
                $name = $b.Current.Name
                if ($name -and $name -match $Pattern) {
                    $ip = $b.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
                    $ip.Invoke()
                    Write-Host "[capture] invoked button '$name'"
                    return $true
                }
            }
        } catch {
            Write-Host "[capture] invoke '$Pattern' failed: $($_.Exception.Message)"
        }
        return $false
    }

    # 1) main window
    Capture-Shot -FilePath '01-main.png' -Desc 'main screen'

    # 2) best-effort: drive the UI via UI Automation and capture extra shots.
    #    WebView2 may not expose its accessibility tree, in which case these are
    #    skipped (never faked).
    if ($root) {
        $did = $false
        $scanOk = $false
        for ($try = 0; $try -lt 4 -and -not $scanOk; $try++) {
            $scanOk = Invoke-ButtonByName -Root $root -Pattern '(\u5f00\u59cb\u626b\u63cf|scan)'
            if (-not $scanOk) { Start-Sleep -Seconds 2 }
        }
        if ($scanOk) {
            Start-Sleep -Seconds 4
            Capture-Shot -FilePath '02-scanning.png' -Desc 'scan in progress (best-effort)'
            $did = $true
        }
        if (Invoke-ButtonByName -Root $root -Pattern '\u9884\u89c8') {
            Start-Sleep -Seconds 2
            Capture-Shot -FilePath '03-results.png' -Desc 'scan results / preview (best-effort)'
            $did = $true
        }
        if (-not $did) { Write-Host "[capture] no actionable buttons found via UIA -> extra shots skipped" }
    } else {
        Write-Host "[capture] UIA root unavailable -> extra shots skipped"
    }
}
finally {
    if ($proc -and -not $proc.HasExited) {
        Write-Host "[capture] stopping PID=$($proc.Id)"
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "===== capture summary ====="
$results | Format-Table -AutoSize | Out-String | Write-Host
$results | ConvertTo-Json -Depth 4 | Set-Content -Encoding UTF8 (Join-Path $OutDir 'capture-summary.json')
Write-Host "[capture] summary written to $(Join-Path $OutDir 'capture-summary.json')"

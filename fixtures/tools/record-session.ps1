<#
.SYNOPSIS
  Record a fixture: 16 kHz mono WAV from the desk feed / USB interface, plus
  slide-change ground truth captured by pressing Enter (no key logger).

.EXAMPLE
  .\record-session.ps1 -List
  .\record-session.ps1 -Device "Line In (Behringer UMC204HD)" -Name amazing-grace -Song "Amazing Grace"

  Press Enter at every slide change, type n + Enter to move to the next song,
  q + Enter to stop. Writes <OutDir>\<Name>\audio.wav and truth.jsonl.
#>
param(
    [string]$Device,
    [string]$Name,
    [string]$Song,
    [string]$OutDir = (Join-Path $PSScriptRoot "..\recordings"),
    [switch]$List
)

$ErrorActionPreference = "Stop"
if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    throw "ffmpeg not found. Install with: winget install Gyan.FFmpeg"
}

if ($List) {
    # ffmpeg prints the DirectShow device list on stderr and exits non-zero;
    # merge it inside cmd so Windows PowerShell 5.1 does not raise on stderr.
    cmd /c "ffmpeg -hide_banner -list_devices true -f dshow -i dummy 2>&1" |
        Where-Object { $_ -match '\(audio\)' } |
        ForEach-Object { ($_ -replace '^.*?"', '"') }
    return
}
if (-not $Device -or -not $Name -or -not $Song) {
    throw "Need -Device, -Name and -Song (use -List to see devices)."
}

$dir = Join-Path $OutDir $Name
$dir = (New-Item -ItemType Directory -Force $dir).FullName   # absolute, for [IO.File] and ffmpeg
$wav = Join-Path $dir "audio.wav"
$truth = Join-Path $dir "truth.jsonl"
if (Test-Path $truth) { throw "$truth exists; pick a new -Name." }

$psi = [Diagnostics.ProcessStartInfo]::new("ffmpeg")
$psi.Arguments = "-hide_banner -loglevel error -f dshow -i `"audio=$Device`" " +
                 "-ac 1 -ar 16000 -c:a pcm_s16le -y `"$wav`""
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true   # lets us send 'q' so ffmpeg finalises the WAV header
$ff = [Diagnostics.Process]::Start($psi)
# Lines are written as they happen (t_ms from ffmpeg launch) so a crash keeps
# them; on stop they are rewritten relative to the first audio sample, because
# opening a DirectShow device takes ~1 s before ffmpeg records anything.
$clock = [Diagnostics.Stopwatch]::StartNew()
$events = [Collections.Generic.List[hashtable]]::new()

function Write-Truth($obj) {
    $events.Add($obj)
    # No BOM: Windows PowerShell's -Encoding utf8 adds one and breaks JSON parsers on line 1.
    [IO.File]::AppendAllText($truth, ($obj | ConvertTo-Json -Compress) + "`n", $utf8NoBom)
}
$utf8NoBom = [Text.UTF8Encoding]::new($false)

$slide = 0
Write-Truth @{ t_ms = 0; event = "start"; device = $Device; song = $Song }
Write-Host "Recording '$Song' -> $wav"
Write-Host "Enter = slide change | n + Enter = next song | q + Enter = stop"

try {
    while ($true) {
        $line = Read-Host "[$Song] slide $slide"
        $t = $clock.ElapsedMilliseconds
        if ($ff.HasExited) { throw "ffmpeg stopped (exit $($ff.ExitCode)); check the device name." }
        switch ($line.Trim().ToLower()) {
            "q" { return }
            "n" {
                $Song = Read-Host "Next song title"
                $slide = 0
                Write-Truth @{ t_ms = $t; song = $Song; slide = $slide }   # time of the 'n' press
            }
            default {
                $slide++
                Write-Truth @{ t_ms = $t; song = $Song; slide = $slide }
            }
        }
    }
}
finally {
    $stop = $clock.ElapsedMilliseconds
    Write-Truth @{ t_ms = $stop; event = "stop" }
    if (-not $ff.HasExited) {
        $ff.StandardInput.Write("q")
        if (-not $ff.WaitForExit(5000)) { $ff.Kill(); Write-Warning "ffmpeg killed; WAV header may need repair." }
    }
    $dur = cmd /c "ffprobe -v error -show_entries format=duration -of csv=p=0 `"$wav`" 2>nul"
    $audioMs = [double]::NaN
    if ([double]::TryParse("$dur".Trim(), [Globalization.NumberStyles]::Float,
                           [Globalization.CultureInfo]::InvariantCulture, [ref]$audioMs)) {
        $offset = [math]::Max(0, $stop - [math]::Round($audioMs * 1000))
        $lines = foreach ($e in $events) {
            $e.t_ms = [math]::Max(0, $e.t_ms - $offset)
            if ($e.event -eq "start") { $e.audio_offset_ms = $offset }
            $e | ConvertTo-Json -Compress
        }
        [IO.File]::WriteAllText($truth, (($lines -join "`n") + "`n"), $utf8NoBom)
        Write-Host "Timestamps shifted by -$offset ms to match the first audio sample."
    } else {
        Write-Warning "ffprobe could not read the WAV; truth.jsonl t_ms is still from ffmpeg launch."
    }
    $mb = [math]::Round((Get-Item $wav).Length / 1MB, 1)
    Write-Host "Saved $wav ($mb MB) and $truth"
    if ($mb -gt 25) { Write-Warning "Over 25 MB: upload to Drive Voxxa/fixtures/, do not commit." }
}

<#
.SYNOPSIS
  Make a fixture's truth.jsonl by pressing Enter while its audio.wav plays
  (Q9V, ADR-019 § 7). For a WAV made with ffmpeg, such as a private fixture:
  record-session.ps1 records from a device, this one times an existing file.

.DESCRIPTION
  Plays the WAV on the default speakers and writes one truth row per key
  press, in ms from the start of playback, next to the WAV. It writes nothing
  else, prints no slide text, and never opens a microphone.

  Enter          the next slide (the first Enter is slide 0: press it when
                 the first slide's words begin, so the first slide has a row)
  <number>       that slide of the current song (a repeated chorus, a jump)
  b              blank (the operator clears the screen, e.g. a prayer)
  n              the next song in setlist.json, slide 0
  q              stop (or let the audio end, then q)

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File fixtures\tools\tap-truth.ps1 -Wav fixtures\private\new-life\audio.wav
#>
param(
    [Parameter(Mandatory = $true)][string]$Wav
)

$ErrorActionPreference = "Stop"
$Wav = (Resolve-Path -LiteralPath $Wav).Path
$truth = Join-Path (Split-Path -Parent $Wav) "truth.jsonl"
if (Test-Path -LiteralPath $truth) { throw "$truth exists; rename or delete it first." }
# No BOM: Windows PowerShell's -Encoding utf8 adds one, and voxxa-eval's JSON reader rejects it.
$utf8NoBom = [Text.UTF8Encoding]::new($false)

$player = [Media.SoundPlayer]::new($Wav)
$player.Load()   # read the whole file first, so playback starts at once
$player.Play()
$clock = [Diagnostics.Stopwatch]::StartNew()
$song = 0
$slide = -1
Write-Host "Playing $Wav"
Write-Host "Enter = next slide (first Enter = slide 0) | <number> = that slide | b = blank | n = next song | q = stop"

try {
    while ($true) {
        $line = (Read-Host "song $song, slide $slide").Trim().ToLower()
        $t = $clock.ElapsedMilliseconds
        if ($line -eq "q") { break }
        if ($line -eq "b") {
            $row = [ordered]@{ t_ms = $t; blank = $true }
        } else {
            if ($line -eq "n") { $song++; $slide = 0 }
            elseif ($line -match '^\d+$') { $slide = [int]$line }
            elseif ($line -eq "") { $slide++ }
            else { Write-Host "Not a key: Enter, a number, b, n or q."; continue }
            $row = [ordered]@{ t_ms = $t; song = $song; slide = $slide }
        }
        [IO.File]::AppendAllText($truth, (ConvertTo-Json -InputObject $row -Compress) + "`n", $utf8NoBom)
    }
}
finally {
    $player.Stop()
    $player.Dispose()
}
Write-Host "Wrote $truth"

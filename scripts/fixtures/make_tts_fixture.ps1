# Synthetic eval fixtures from offline Windows text-to-speech (Q6, user decision 2).
#
# Writes 16 kHz mono 16-bit PCM WAVs of public-domain hymn slides (and one
# prayer written for Voxxa) to src-tauri/tests/fixtures/voxxad/tts-*.wav, then
# each fixture's audio.json and truth.jsonl under fixtures/recordings/<name>/.
# The hymn text is read from the fixture's committed setlist.json, so the
# audio speaks exactly the slide text. Speech goes to WAV files only: the
# synthesizer's output is set to a file before anything is spoken and never
# to the speakers.
#
# Truth times come from the synthesizer, not from listening: a slide row is at
# the part's offset in audio.wav plus the audio position of the part's first
# word (the earliest SpeakProgress event); a blank row is at the first word of
# a prayer, or, for a silence step marked blank, where that silence starts
# (the end of the previous part's WAV). See fixtures/README.md.
#
# Usage (Windows PowerShell 5.1, from anywhere):
#   powershell -ExecutionPolicy Bypass -File scripts/fixtures/make_tts_fixture.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/fixtures/make_tts_fixture.ps1 -Fixture refrain-it-is-well
# Then record each fixture's session.jsonl with
#   cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full --record fixtures/recordings/<name>
param(
    [string[]]$Fixture = @('refrain-it-is-well', 'jump-be-thou-my-vision', 'prayer-between-songs')
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$wavDir = Join-Path $repo 'src-tauri\tests\fixtures\voxxad'
$recDir = Join-Path $repo 'fixtures\recordings'
$wavRel = '../../../src-tauri/tests/fixtures/voxxad'

Add-Type -AssemblyName System.Speech
Add-Type -ReferencedAssemblies System.Speech -TypeDefinition @'
using System;
using System.Speech.AudioFormat;
using System.Speech.Synthesis;
using System.Threading;

public static class TtsFixture {
    // Speaks the lines (a short break between them) into a 16 kHz mono
    // 16-bit WAV. Returns where the first word starts, in samples, as the
    // synthesizer reports it.
    public static long Speak(string voice, string[] lines, int breakMs, string path) {
        long first = long.MaxValue;
        object gate = new object();
        using (var s = new SpeechSynthesizer()) {
            s.SelectVoice(voice);
            s.SetOutputToWaveFile(path, new SpeechAudioFormatInfo(16000, AudioBitsPerSample.Sixteen, AudioChannel.Mono));
            s.SpeakProgress += (o, e) => {
                lock (gate) { first = Math.Min(first, (long)Math.Round(e.AudioPosition.TotalMilliseconds * 16)); }
            };
            var p = new PromptBuilder();
            for (int i = 0; i < lines.Length; i++) {
                if (i > 0) p.AppendBreak(TimeSpan.FromMilliseconds(breakMs));
                p.AppendText(lines[i]);
            }
            s.Speak(p);
            s.SetOutputToNull();
            // Events are raised on pool threads; give them time to arrive.
            for (int i = 0; i < 50; i++) {
                lock (gate) { if (first != long.MaxValue) break; }
                Thread.Sleep(20);
            }
        }
        lock (gate) {
            if (first == long.MaxValue) throw new Exception("no word events for " + path);
            return first;
        }
    }
}
'@

$voices = @{ david = 'Microsoft David Desktop'; zira = 'Microsoft Zira Desktop' }

# A prayer written for Voxxa (no hymn text).
$prayer = @(
    'Let us pray.',
    'Father, thank you for this morning and for everyone gathered here.',
    'Quiet our hearts as we listen, and help us to love one another this week.',
    'We ask this in the name of Jesus. Amen.'
)

# Each fixture: a list of steps. A step is a silence (ms), a slide part
# (song and slide index in the fixture's setlist.json; a truth slide row at its
# first word), or a prayer part (a truth blank row at its first word).
# The WAV name says what it holds; a part reused in a fixture reuses the WAV.
$fixtures = [ordered]@{
    # Repeated chorus: v1, refrain, v2, refrain, v3, refrain (refrain = slide 1).
    'refrain-it-is-well' = @(
        @{ silence = 2000 },
        @{ wav = 'tts-it-is-well-v1-zira'; voice = 'zira'; song = 0; slide = 0 },
        @{ silence = 1000 },
        @{ wav = 'tts-it-is-well-refrain-zira'; voice = 'zira'; song = 0; slide = 1 },
        @{ silence = 1500 },
        @{ wav = 'tts-it-is-well-v2-zira'; voice = 'zira'; song = 0; slide = 2 },
        @{ silence = 1000 },
        @{ wav = 'tts-it-is-well-refrain-zira'; voice = 'zira'; song = 0; slide = 1 },
        @{ silence = 1500 },
        @{ wav = 'tts-it-is-well-v3-zira'; voice = 'zira'; song = 0; slide = 3 },
        @{ silence = 1000 },
        @{ wav = 'tts-it-is-well-refrain-zira'; voice = 'zira'; song = 0; slide = 1 },
        @{ silence = 4000 }
    )
    # Jumps: v1, v3 (v2 skipped), v4, then back to v1.
    'jump-be-thou-my-vision' = @(
        @{ silence = 2000 },
        @{ wav = 'tts-be-thou-my-vision-v1-david'; voice = 'david'; song = 0; slide = 0 },
        @{ silence = 1500 },
        @{ wav = 'tts-be-thou-my-vision-v3-david'; voice = 'david'; song = 0; slide = 2 },
        @{ silence = 1500 },
        @{ wav = 'tts-be-thou-my-vision-v4-david'; voice = 'david'; song = 0; slide = 3 },
        @{ silence = 1500 },
        @{ wav = 'tts-be-thou-my-vision-v1-david'; voice = 'david'; song = 0; slide = 0 },
        @{ silence = 4000 }
    )
    # Two songs with a spoken prayer between them (no long pause before it).
    'prayer-between-songs' = @(
        @{ silence = 2000 },
        @{ wav = 'tts-holy-holy-holy-v1-david'; voice = 'david'; song = 0; slide = 0 },
        @{ silence = 1500 },
        @{ wav = 'tts-holy-holy-holy-v2-david'; voice = 'david'; song = 0; slide = 1 },
        @{ silence = 1000 },
        @{ wav = 'tts-prayer-zira'; voice = 'zira'; prayer = $true },
        @{ silence = 2000 },
        @{ wav = 'tts-come-thou-fount-v1-zira'; voice = 'zira'; song = 1; slide = 0 },
        @{ silence = 1500 },
        @{ wav = 'tts-come-thou-fount-v2-zira'; voice = 'zira'; song = 1; slide = 1 },
        @{ silence = 4000 }
    )
}

function Write-Lf([string]$path, [string[]]$lines) {
    $text = ($lines -join "`n") + "`n"
    [IO.File]::WriteAllText($path, $text, (New-Object Text.UTF8Encoding $false))
}

foreach ($name in $Fixture) {
    if (-not $fixtures.Contains($name)) { throw "unknown fixture: $name" }
    $dir = Join-Path $recDir $name
    $setlist = (Get-Content -Raw -Encoding UTF8 (Join-Path $dir 'setlist.json') | ConvertFrom-Json).setlist
    $made = @{}
    $offset = [long]0        # samples into audio.wav
    $recipe = @()
    $truth = @()
    foreach ($step in $fixtures[$name]) {
        if ($step.ContainsKey('silence')) {
            $recipe += "  { `"silence_ms`": $($step.silence) }"
            if ($step.blank) { $truth += "{`"t_ms`":$([Math]::Round($offset / 16)),`"blank`":true}" }
            $offset += [long]$step.silence * 16
            continue
        }
        $wav = Join-Path $wavDir ($step.wav + '.wav')
        if (-not $made.ContainsKey($step.wav)) {
            if ($step.prayer) { $lines = $prayer }
            else { $lines = $setlist[$step.song].slides[$step.slide].text -split "`n" }
            $made[$step.wav] = [TtsFixture]::Speak($voices[$step.voice], $lines, 300, $wav)
        }
        $first = $made[$step.wav]
        $t = [Math]::Round(($offset + $first) / 16)
        if ($step.prayer) { $truth += "{`"t_ms`":$t,`"blank`":true}" }
        else { $truth += "{`"t_ms`":$t,`"song`":$($step.song),`"slide`":$($step.slide)}" }
        $recipe += "  { `"wav`": `"$wavRel/$($step.wav).wav`" }"
        # The length of the data chunk, as audio.rs joins it.
        $samples = [long]-1
        $bytes = [IO.File]::ReadAllBytes($wav)
        for ($at = 12; $at + 8 -le $bytes.Length; ) {
            $len = [BitConverter]::ToUInt32($bytes, $at + 4)
            if ([Text.Encoding]::ASCII.GetString($bytes, $at, 4) -eq 'data') { $samples = [long]($len / 2); break }
            $at += 8 + $len + ($len % 2)
        }
        if ($samples -lt 0) { throw "$wav has no data chunk" }
        Write-Host ("{0,-24} {1,-34} first word {2,6} ms  length {3,6} ms  truth {4,6} ms" -f `
                $name, $step.wav, [Math]::Round($first / 16), [Math]::Round($samples / 16), $t)
        $offset += $samples
    }
    Write-Lf (Join-Path $dir 'audio.json') (@('[', ($recipe -join ",`n"), ']'))
    Write-Lf (Join-Path $dir 'truth.jsonl') $truth
    Write-Host ("{0}: {1:n1} s of audio, {2} truth rows" -f $name, ($offset / 16000), $truth.Count)
}

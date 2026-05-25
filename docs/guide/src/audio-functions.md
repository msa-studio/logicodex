# Jadual Fungsi Audio Lengkap

Senarai lengkap 22 fungsi audio Raylib yang tersedia dalam Logicodex.

---

## Fungsi Audio Device (4)

| Fungsi | Parameter | Return | Safety |
|---|---|---|---|
| `InitAudioDevice` | — | `Void` | Safe |
| `CloseAudioDevice` | — | `Void` | Safe |
| `IsAudioDeviceReady` | — | `Bool` | Safe |
| `SetMasterVolume` | `volume: F32` | `Void` | Safe |

## Fungsi Sound (5)

| Fungsi | Parameter | Return | Safety |
|---|---|---|---|
| `LoadSound` | `namaFail: Text` | `Sound` | Safe |
| `UnloadSound` | `bunyi: Sound` | `Void` | Safe |
| `PlaySound` | `bunyi: Sound` | `Void` | Safe |
| `StopSound` | `bunyi: Sound` | `Void` | Safe |
| `IsSoundPlaying` | `bunyi: Sound` | `Bool` | Safe |

## Fungsi Music (8)

| Fungsi | Parameter | Return | Safety |
|---|---|---|---|
| `LoadMusicStream` | `namaFail: Text` | `Music` | Safe |
| `UnloadMusicStream` | `muzik: Music` | `Void` | Safe |
| `PlayMusicStream` | `muzik: Music` | `Void` | Safe |
| `StopMusicStream` | `muzik: Music` | `Void` | Safe |
| `IsMusicStreamPlaying` | `muzik: Music` | `Bool` | Safe |
| `UpdateMusicStream` | `muzik: Music` | `Void` | Safe |
| `SetMusicVolume` | `muzik: Music, volume: F32` | `Void` | Safe |
| `SeekMusicStream` | `muzik: Music, posisi: F32` | `Void` | Safe |

## Fungsi Audio Stream (5)

| Fungsi | Parameter | Return | Safety |
|---|---|---|---|
| `LoadAudioStream` | `sampleRate: U32, sampleSize: U32, channels: U32` | `AudioStream` | Safe |
| `UnloadAudioStream` | `stream: AudioStream` | `Void` | Safe |
| `PlayAudioStream` | `stream: AudioStream` | `Void` | Safe |
| `StopAudioStream` | `stream: AudioStream` | `Void` | Safe |
| `IsAudioStreamPlaying` | `stream: AudioStream` | `Bool` | Safe |
| `SetAudioStreamCallback` | `stream: AudioStream, callback: AudioCallback` | `Void` | StrictAudioContext |

## Tipe Audio

| Tipe | Medan | Keterangan |
|---|---|---|
| `Sound` | `i64 handle` | Bunyi pendek, dimuat sepenuhnya ke memori |
| `Music` | `i64 handle` | Audio panjang, di-stream semasa main |
| `AudioStream` | `i64 handle` | Stream audio real-time dengan callback |
| `Wave` | `i64 handle` | Data audio mentah (samples + format) |
| `AudioCallback` | — | Function signature untuk callback: `fn(&mut [F32], U32) -> Void` |

## Format Audio yang Disokong

| Format | Sound | Music | Keterangan |
|---|---|---|---|
| WAV | ✅ | ❌ | Uncompressed, dimuat ke memori |
| OGG | ❌ | ✅ | Compressed, streaming |
| MP3 | ❌ | ✅ | Compressed, streaming |
| FLAC | ❌ | ✅ | Lossless, streaming |
| MOD | ❌ | ✅ | Tracker music |
| XM | ❌ | ✅ | Extended Module |

## StrictAudioContext Check

Fungsi `SetAudioStreamCallback` memicu semakan StrictAudioContext. Callback **mesti** mematuhi 4 peraturan:

| Peraturan | Kod Ralat | Apa Dihalang |
|---|---|---|
| Tiada I/O | `AudioViolationIo` | `print`, `DrawText`, `InitWindow` |
| Tiada Rekursi | `AudioViolationRecursion` | Panggilan diri sendiri |
| Tiada Unbounded Loop | `AudioViolationUnboundedLoop` | `loop { }` tanpa henti |
| Tiada Forbidden Call | `AudioViolationForbiddenCall` | `malloc`, `free`, `spawn` |

## Contoh Penggunaan Lengkap

### Mainkan Sound Effect

```logicodex
InitAudioDevice()
BINA bunyi SEBAGAI Sound = LoadSound("jump.wav")
PlaySound(bunyi)
-- ... tunggu selesai ...
UnloadSound(bunyi)
CloseAudioDevice()
```

### Mainkan Music

```logicodex
InitAudioDevice()
BINA muzik SEBAGAI Music = LoadMusicStream("background.mp3")
SetMusicVolume(muzik, 0.5)
PlayMusicStream(muzik)

SEMENTARA !WindowShouldClose()
    UpdateMusicStream(muzik)   -- PENTING: panggil setiap frame
    -- ... render frame ...
TAMAT_SEMENTARA

StopMusicStream(muzik)
UnloadMusicStream(muzik)
CloseAudioDevice()
```

### Audio Stream dengan Callback

```logicodex
InitAudioDevice()
BINA stream SEBAGAI AudioStream = LoadAudioStream(44100, 16, 1)
SetAudioStreamCallback(stream, my_callback)
PlayAudioStream(stream)

SEMENTARA BENAR
    -- stream berjalan di latar belakang
TAMAT_SEMENTARA

StopAudioStream(stream)
UnloadAudioStream(stream)
CloseAudioDevice()
```

# Chapter 14: Audio Programming

Logicodex menyokong 22 fungsi audio Raylib dengan integrasi StrictAudioContext.

---

## Audio Device {#device}

### InitAudioDevice
```logicodex
InitAudioDevice() -> Void
-- Mesti dipanggil sebelum fungsi audio lain
-- Contoh:
InitAudioDevice()
```

### CloseAudioDevice
```logicodex
CloseAudioDevice() -> Void
-- Membersihkan semua sumber audio
-- Panggil sebelum program tamat
```

### IsAudioDeviceReady
```logicodex
IsAudioDeviceReady() -> Bool
-- Semak sama ada audio device sedia
-- Contoh:
JIKA !IsAudioDeviceReady()
    PAPAR "Audio device tidak tersedia!"
    PULANG 1
TAMAT_JIKA
```

### SetMasterVolume
```logicodex
SetMasterVolume(volume: F32) -> Void
-- Tetapkan volume master (0.0 hingga 1.0)
-- Contoh:
SetMasterVolume(0.8)   -- 80% volume
```

---

## Sound (Efek Pendek) {#sound}

### LoadSound
```logicodex
LoadSound(namaFail: Text) -> Sound
-- Memuat fail audio pendek (WAV)
-- Data dimuat sepenuhnya ke memori
-- Contoh:
BINA bunyi_lompat SEBAGAI Sound = LoadSound("jump.wav")
```

### UnloadSound
```logicodex
UnloadSound(bunyi: Sound) -> Void
-- Bebaskan memori bunyi
-- Contoh:
UnloadSound(bunyi_lompat)
```

### PlaySound
```logicodex
PlaySound(bunyi: Sound) -> Void
-- Main bunyi sekali
-- Contoh:
PlaySound(bunyi_lompat)
```

### StopSound
```logicodex
StopSound(bunyi: Sound) -> Void
-- Hentikan bunyi yang sedang dimain
```

### IsSoundPlaying
```logicodex
IsSoundPlaying(bunyi: Sound) -> Bool
-- Semak sama ada bunyi sedang dimain
```

---

## Music (Audio Panjang) {#music}

### LoadMusicStream
```logicodex
LoadMusicStream(namaFail: Text) -> Music
-- Memuat fail audio panjang (MP3, OGG, FLAC)
-- Data di-stream (tidak dimuat sepenuhnya)
-- Contoh:
BINA muzik_latar SEBAGAI Music = LoadMusicStream("background.mp3")
```

### UnloadMusicStream
```logicodex
UnloadMusicStream(muzik: Music) -> Void
-- Bebaskan sumber muzik
```

### PlayMusicStream
```logicodex
PlayMusicStream(muzik: Music) -> Void
-- Mula main muzik
-- Contoh:
PlayMusicStream(muzik_latar)
```

### StopMusicStream
```logicodex
StopMusicStream(muzik: Music) -> Void
-- Hentikan muzik
```

### IsMusicStreamPlaying
```logicodex
IsMusicStreamPlaying(muzik: Music) -> Bool
-- Semak sama ada muzik sedang dimain
```

### UpdateMusicStream
```logicodex
UpdateMusicStream(muzik: Music) -> Void
-- Kemas kini buffer muzik (panggil setiap frame)
-- Contoh:
SEMENTARA !WindowShouldClose()
    UpdateMusicStream(muzik_latar)
    -- draw frame
TAMAT_SEMENTARA
```

### SetMusicVolume / SeekMusicStream
```logicodex
SetMusicVolume(muzik: Music, volume: F32) -> Void  -- 0.0 hingga 1.0
SeekMusicStream(muzik: Music, posisi: F32) -> Void  -- Dalam saat
```

---

## Audio Stream (Real-time) {#stream}

### LoadAudioStream
```logicodex
LoadAudioStream(sampleRate: U32, sampleSize: U32, channels: U32) -> AudioStream
-- Membuat audio stream untuk data real-time
-- Contoh:
BINA stream SEBAGAI AudioStream = LoadAudioStream(44100, 16, 1)
```

### UnloadAudioStream / PlayAudioStream / StopAudioStream / IsAudioStreamPlaying
```logicodex
UnloadAudioStream(stream: AudioStream) -> Void
PlayAudioStream(stream: AudioStream) -> Void
StopAudioStream(stream: AudioStream) -> Void
IsAudioStreamPlaying(stream: AudioStream) -> Bool
```

### SetAudioStreamCallback
```logicodex
SetAudioStreamCallback(stream: AudioStream, callback: AudioCallback) -> Void
-- Tetapkan fungsi callback untuk mengisi buffer audio
-- ⚠️ Mesti patuh StrictAudioContext!
-- Contoh:
SetAudioStreamCallback(stream, audio_callback)
```

---

## StrictAudioContext: 4 Peraturan {#strict}

Callback audio **mesti** mematuhi 4 peraturan berikut:

### Peraturan 1: Tiada I/O dalam Callback

```logicodex
-- ❌ SALAH — melanggar AudioViolationIo
FUNGSI callback_jahat(buffer: &mut [F32], frames: U32) -> Void
MULA
    PAPAR "Callback dipanggil!"   -- ❌ Tiada print dalam ISR!
    InitWindow(800, 600, "X")     -- ❌ Tiada InitWindow!
    DrawText("X", 0, 0, 20, RED)  -- ❌ Tiada DrawText!
TAMAT
```

```logicodex
-- ✅ BETUL — tiada I/O
FUNGSI callback_betul(buffer: &mut [F32], frames: U32) -> Void
MULA
    UNTUK i DARI 0 HINGGA frames
        buffer[i] = 0.0   -- hanya proses audio
    TAMAT_UNTUK
TAMAT
```

### Peraturan 2: Tiada Rekursi

```logicodex
-- ❌ SALAH — melanggar AudioViolationRecursion
FUNGSI callback_rekursif(buffer: &mut [F32], frames: U32) -> Void
MULA
    callback_rekursif(buffer, frames)   -- ❌ Panggil diri sendiri!
TAMAT
```

### Peraturan 3: Tiada Unbounded Loop

```logicodex
-- ❌ SALAH — melanggar AudioViolationUnboundedLoop
FUNGSI callback_loop(buffer: &mut [F32], frames: U32) -> Void
MULA
    SEMENTARA BENAR   -- ❌ Loop tanpa henti!
        buffer[0] = 0.0
    TAMAT_SEMENTARA
TAMAT
```

```logicodex
-- ✅ BETUL — loop dengan batas
FUNGSI callback_betul(buffer: &mut [F32], frames: U32) -> Void
MULA
    UNTUK i DARI 0 HINGGA frames   -- ✅ Loop terbatas
        buffer[i] = 0.0
    TAMAT_UNTUK
TAMAT
```

### Peraturan 4: Tiada malloc/free/spawn

```logicodex
-- ❌ SALAH — melanggar AudioViolationForbiddenCall
FUNGSI callback_alokasi(buffer: &mut [F32], frames: U32) -> Void
MULA
    BINA temp SEBAGAI [F32; 100] = [0.0; 100]  -- ❌ Alokasi dalam ISR!
    spawn pelakon_lain()                          -- ❌ Spawn thread!
TAMAT
```

### Mematuhi Semua Peraturan

```logicodex
-- ✅ Contoh callback yang betul sepenuhnya
FUNGSI audio_callback(buffer: &mut [F32], frames: U32) -> Void
MULA
    -- Hanya gunakan variable local dan pre-allocated state
    -- Tiada I/O, tiada alokasi, tiada rekursi, loop terbatas
    
    UNTUK i DARI 0 HINGGA frames
        -- Gelombang sinus 440Hz (A4)
        BINA t SEBAGAI F64 = (fasa + i SEBAGAI U64) SEBAGAI F64 / 44100.0
        buffer[i] = sin(2.0 * PI * 440.0 * t) SEBAGAI F32
    TAMAT_UNTUK
    
    fasa = fasa + frames SEBAGAI U64
TAMAT
```

### Ralat Jika Melanggar

Jika anda melanggar mana-mana peraturan, compiler akan memberi ralat:

```
error[E007]: AudioViolationIo
  --> audio.ldx:15:5
   |
15 |     PAPAR "Callback dipanggil!"
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^ Audio callback tidak boleh melakukan I/O
   |
   = bantuan: Pindahkan operasi I/O ke luar callback audio
   = help: Move I/O operations outside the audio callback
```

---

## Contoh Program Audio Lengkap

```logicodex
PROGRAM audio_demo

GUNA_JENIS I32
GUNA_JENIS F32
GUNA_JENIS U32
GUNA_JENIS F64
GUNA_JENIS U64

BINA fasa SEBAGAI U64 = 0

FUNGSI audio_callback(buffer: &mut [F32], frames: U32) -> Void
MULA
    UNTUK i DARI 0 HINGGA frames
        BINA t SEBAGAI F64 = (fasa + i SEBAGAI U64) SEBAGAI F64 / 44100.0
        buffer[i] = sin(2.0 * PI * 440.0 * t) SEBAGAI F32
    TAMAT_UNTUK
    fasa = fasa + frames SEBAGAI U64
TAMAT

FUNGSI utama() -> I32
MULA
    InitWindow(400, 200, "Audio Demo")
    InitAudioDevice()
    
    JIKA !IsAudioDeviceReady()
        PAPAR "Audio tidak tersedia!"
        PULANG 1
    TAMAT_JIKA
    
    BINA stream SEBAGAI AudioStream = LoadAudioStream(44100, 16, 1)
    SetAudioStreamCallback(stream, audio_callback)
    PlayAudioStream(stream)
    
    SEMENTARA !WindowShouldClose()
    MULA
        BeginDrawing()
        ClearBackground(RAYWHITE)
        DrawText("Main nada A4 (440Hz)...", 80, 80, 20, DARKGRAY)
        DrawText("Tutup tetingkap untuk berhenti.", 60, 120, 16, GRAY)
        EndDrawing()
    TAMAT_SEMENTARA
    
    StopAudioStream(stream)
    UnloadAudioStream(stream)
    CloseAudioDevice()
    CloseWindow()
    
    PULANG 0
TAMAT

TAMAT PROGRAM
```

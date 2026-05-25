# Chapter 18: Resepi dan Contoh

Koleksi resepi lengkap untuk pelbagai senario penggunaan.

---

## HTTP Server Sederhana {#http}

```logicodex
PROGRAM http_server

GUNA_JENIS I32
GUNA_JENIS Text

PERKHIDMATAN HttpServer {
    port: 8080,
    keperluan: [Net.Admin, Storage.Read("/www")],
    pengendali: handle_request,
    dasar: Halang,
}

FUNGSI halaman_utama() -> Text
MULA
    PULANG "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n"
         + "<html><body><h1>Logicodex Server</h1>"
         + "<p>Server berjalan!</p></body></html>"
TAMAT

FUNGSI halaman_tidak_jumpa() -> Text
MULA
    PULANG "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\n404 - Halaman tidak dijumpai"
TAMAT

FUNGSI halaman_status() -> Text
MULA
    PULANG "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n"
         + "{ \"status\": \"ok\", \"server\": \"Logicodex\" }"
TAMAT

FUNGSI handle_request(sambungan: Connection) -> Void
    PERLUKAN Net.Admin
MULA
    BINA buffer SEBAGAI [U8; 4096] = [0; 4096]
    BINA n SEBAGAI I32 = sambungan.baca(&mut buffer)
    
    JIKA n <= 0
        PULANG
    TAMAT_JIKA
    
    -- Parse request path (simplifikasi)
    BINA request SEBAGAI Text = buffer SEBAGAI Text
    
    BINA response SEBAGAI Text
    JIKA request.mengandung("GET / ")
        response = halaman_utama()
    LAIN_JIKA request.mengandung("GET /status")
        response = halaman_status()
    LAIN
        response = halaman_tidak_jumpa()
    TAMAT_JIKA
    
    sambungan.tulis(response SEBAGAI &[U8])
TAMAT

FUNGSI utama() -> I32
MULA
    PAPAR "Memulakan HTTP server pada port 8080..."
    -- Server akan berjalan sehingga diberhentikan
    PULANG 0
TAMAT

TAMAT PROGRAM
```

---

## Aplikasi Grafik Interaktif {#grafik}

```logicodex
PROGRAM grafik_interaktif

GUNA_JENIS I32
GUNA_JENIS F32
GUNA_JENIS F64

STRUKTUR Player {
    x: F32,
    y: F32,
    kelajuan: F32,
    saiz: F32,
    warna: Color,
}

FUNGSI utama() -> I32
MULA
    InitWindow(800, 600, "Grafik Interaktif")
    SetTargetFPS(60)
    
    BINA pemain SEBAGAI Player = Player {
        x: 400.0,
        y: 300.0,
        kelajuan: 5.0,
        saiz: 20.0,
        warna: BLUE,
    }
    
    BINA skor SEBAGAI I32 = 0
    
    SEMENTARA !WindowShouldClose()
    MULA
        -- Input
        JIKA IsKeyDown(KEY_UP)    || IsKeyDown(KEY_W)
            pemain.y = pemain.y - pemain.kelajuan
        JIKA IsKeyDown(KEY_DOWN)  || IsKeyDown(KEY_S)
            pemain.y = pemain.y + pemain.kelajuan
        JIKA IsKeyDown(KEY_LEFT)  || IsKeyDown(KEY_A)
            pemain.x = pemain.x - pemain.kelajuan
        JIKA IsKeyDown(KEY_RIGHT) || IsKeyDown(KEY_D)
            pemain.x = pemain.x + pemain.kelajuan
        
        -- Boundary check
        pemain.x = Clamp(pemain.x, pemain.saiz, 800.0 - pemain.saiz)
        pemain.y = Clamp(pemain.y, pemain.saiz, 600.0 - pemain.saiz)
        
        -- Draw
        BeginDrawing()
        ClearBackground(RAYWHITE)
        
        -- Draw player
        DrawCircle(pemain.x SEBAGAI I32, pemain.y SEBAGAI I32, pemain.saiz, pemain.warna)
        
        -- Draw score
        DrawText("Skor: " + skor, 10, 10, 20, DARKGRAY)
        
        -- Draw instructions
        DrawText("Gunakan WASD atau anak panah untuk bergerak", 10, 570, 16, GRAY)
        
        EndDrawing()
    TAMAT_SEMENTARA
    
    CloseWindow()
    PULANG 0
TAMAT

TAMAT PROGRAM
```

---

## Pemain Audio {#audio-player}

```logicodex
PROGRAM pemain_audio

GUNA_JENIS I32
GUNA_JENIS F32
GUNA_JENIS Text

FUNGSI tunjuk_menu() -> Void
MULA
    PAPAR "=== Pemain Audio Logicodex ==="
    PAPAR "1. Main muzik"
    PAPAR "2. Hentikan muzik"
    PAPAR "3. Set volume"
    PAPAR "4. Keluar"
    PAPAR "Pilihan anda: "
TAMAT

FUNGSI utama() -> I32
MULA
    InitWindow(400, 300, "Pemain Audio")
    InitAudioDevice()
    
    JIKA !IsAudioDeviceReady()
        PAPAR "Audio device tidak tersedia!"
        CloseWindow()
        PULANG 1
    TAMAT_JIKA
    
    BINA muzik SEBAGAI Music = LoadMusicStream("lagu.mp3")
    BINA volume SEBAGAI F32 = 1.0
    
    SEMENTARA !WindowShouldClose()
    MULA
        UpdateMusicStream(muzik)
        
        BeginDrawing()
        ClearBackground(DARKGRAY)
        
        DrawText("Pemain Audio", 120, 50, 24, RAYWHITE)
        
        -- Status
        JIKA IsMusicStreamPlaying(muzik)
            DrawText("Status: SEDANG MAIN", 130, 120, 18, GREEN)
        LAIN
            DrawText("Status: BERHENTI", 140, 120, 18, RED)
        TAMAT_JIKA
        
        DrawText("Volume: " + (volume * 100.0) SEBAGAI I32 + "%", 150, 160, 18, RAYWHITE)
        DrawText("[SPACE] Main/Berhenti", 110, 220, 16, GRAY)
        DrawText("[UP/DOWN] Volume", 120, 240, 16, GRAY)
        DrawText("[ESC] Keluar", 150, 260, 16, GRAY)
        
        EndDrawing()
        
        -- Input
        JIKA IsKeyPressed(KEY_SPACE)
            JIKA IsMusicStreamPlaying(muzik)
                StopMusicStream(muzik)
            LAIN
                PlayMusicStream(muzik)
            TAMAT_JIKA
        JAMAT_JIKA
        
        JIKA IsKeyDown(KEY_UP)
            volume = Clamp(volume + 0.01, 0.0, 1.0)
            SetMusicVolume(muzik, volume)
        JAMAT_JIKA
        
        JIKA IsKeyDown(KEY_DOWN)
            volume = Clamp(volume - 0.01, 0.0, 1.0)
            SetMusicVolume(muzik, volume)
        JAMAT_JIKA
    TAMAT_SEMENTARA
    
    StopMusicStream(muzik)
    UnloadMusicStream(muzik)
    CloseAudioDevice()
    CloseWindow()
    
    PULANG 0
TAMAT

TAMAT PROGRAM
```

---

## Aplikasi Bare Metal (Freestanding) {#baremetal}

```logicodex
PROGRAM bare_metal

GUNA_JENIS I32
GUNA_JENIS PTR<U8>
GUNA_JENIS U8
GUNA_JENIS PTR<U16>
GUNA_JENIS U16
GUNA_JENIS U32
GUNA_JENIS U64

-- MMIO addresses
TANDA KAWASAN_PERKAKAS UART0 SEBAGAI PTR<U8>  = ALAMAT 0x09000000   -- QEMU UART
TANDA KAWASAN_PERKAKAS VGA_TEXT SEBAGAI PTR<U16> = ALAMAT 0xB8000

-- UART functions
FUNGSI uart_hantar(c: U8) -> Void
MULA
    -- QEMU UART: tulis ke 0x09000000
    SEMENTARA (BACA_VOLATIL(UART0 + 0x18) & 0x20) == 0
        -- tunggu TX buffer kosong
    TAMAT_SEMENTARA
    TULIS_VOLATIL(UART0, c)
TAMAT

FUNGSI uart_hantar_string(str: &[U8]) -> Void
MULA
    UNTUK c DARI str
        uart_hantar(c)
    TAMAT_UNTUK
TAMAT

-- VGA functions
FUNGSI vga_tulis_baris(barisan: I32, str: &[U8]) -> Void
MULA
    BINA offset SEBAGAI U64 = barisan SEBAGAI U64 * 80
    UNTUK i DARI 0 HINGGA str.panjang()
        BINA c SEBAGAI U16 = str[i] SEBAGAI U16
        BINA sel SEBAGAI U16 = (0x0F SEBAGAI U16 << 8) | c  -- putih pada hitam
        TULIS_VOLATIL(VGA_TEXT + offset + i, sel)
    TAMAT_UNTUK
TAMAT

-- Main
FUNGSI mula_sistem() -> I32
MULA
    -- Output ke UART
    uart_hantar_string("Logicodex Bare Metal\r\n")
    uart_hantar_string("====================\r\n")
    uart_hantar_string("Sistem dimulakan!\r\n")
    
    -- Output ke VGA
    vga_tulis_baris(0, "Logicodex Bare Metal v1.45")
    vga_tulis_baris(2, "CPU: x86_64-unknown-none")
    vga_tulis_baris(3, "Mem: 2MB")
    vga_tulis_baris(5, "Status: Running...")
    
    -- Loop
    BINA count SEBAGAI U32 = 0
    SEMENTARA BENAR
        count = count + 1
        JIKA count % 10000000 == 0
            uart_hantar_string("." SEBAGAI &[U8])
        TAMAT_JIKA
    TAMAT_SEMENTARA
    
    PULANG 0
TAMAT

TAMAT PROGRAM
```

---

## Latihan

1. Tambah collision detection kepada aplikasi grafik interaktif
2. Tambah multiple track kepada pemain audio
3. Tulis bootloader GRUB untuk program bare metal

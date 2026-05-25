# Chapter 10: Actor — Unit Berkomputasi

Actor adalah unit komputasi terpencil dalam Logicodex — setiap actor berjalan secara bebas dan berkomunikasi melalui channel.

---

## Mendefinisikan Actor {#define}

### Sintaks Asas

```logicodex
PELAKON <Nama> {
    -- State (variable milik actor)
    <variable_declarations>
    
    -- Channel (untuk komunikasi)
    saluran: Channel<Tipe>,
    
    -- Handler function (dipanggil apabila menerima mesej)
    FUNGSI <nama_handler>(msg: <Tipe>) -> Void
    MULA
        -- proses mesej
    TAMAT
}
```

### Contoh: Counter Actor

```logicodex
PELAKON Counter {
    BINA nilai SEBAGAI I32 = 0
    saluran: Channel<CounterMsg>
}

ENUMERASI CounterMsg {
    Tambah(I32),
    Kurang(I32),
    Dapatkan,
    Reset,
}

FUNGSI handle_counter(akt: &Counter, msg: CounterMsg) -> Void
MULA
    PADAN msg {
        CounterMsg::Tambah(n) => { akt.nilai = akt.nilai + n; }
        CounterMsg::Kurang(n) => { akt.nilai = akt.nilai - n; }
        CounterMsg::Dapatkan   => { PAPAR "Nilai: " + akt.nilai; }
        CounterMsg::Reset      => { akt.nilai = 0; }
    }
TAMAT
```

---

## Spawn dan Lifecycle {#spawn}

### Spawn Actor

```logicodex
FUNGSI utama() -> I32
MULA
    -- Buat channel
    BINA ch SEBAGAI Channel<CounterMsg> = Channel::baru(100)
    
    -- Spawn actor (menjadi task/concurrent unit)
    BINA counter SEBAGAI Counter = Counter { nilai: 0, saluran: ch }
    HIDUPKAN counter             -- spawn di shard yang tersedia
    
    -- Hantar mesej kepada actor
    ch.hantar(CounterMsg::Tambah(5))
    ch.hantar(CounterMsg::Tambah(3))
    ch.hantar(CounterMsg::Dapatkan)   -- Output: Nilai: 8
    ch.hantar(CounterMsg::Kurang(2))
    ch.hantar(CounterMsg::Dapatkan)   -- Output: Nilai: 6
    ch.hantar(CounterMsg::Reset)
    
    PULANG 0
TAMAT
```

### Lifecycle Actor

```text
┌──────────┐   spawn()    ┌──────────┐    stop()    ┌──────────┐
│  Dibuat  │─────────────▶│  Berjalan│─────────────▶│  Berhenti│
│ (new)    │              │ (running)│              │ (stopped)│
└──────────┘              └──────────┘              └──────────┘
                              │
                              │ channel penuh
                              ▼
                          ┌──────────┐
                          │  Tersekat│
                          │ (blocked)│
                          └──────────┘
```

### Menghentikan Actor

```logicodex
PELAKON Worker {
    saluran: Channel<WorkMsg>,
    BINA hidup SEBAGAI Bool = BENAR
}

ENUMERASI WorkMsg {
    Kerja(Data),
    Berhenti,       -- Signal untuk hentikan
}

FUNGSI handle_worker(akt: &Worker, msg: WorkMsg) -> Void
MULA
    PADAN msg {
        WorkMsg::Kerja(data) => {
            JIKA akt.hidup
                proses(data)
            TAMAT_JIKA
        }
        WorkMsg::Berhenti => {
            akt.hidup = PALSU
            PAPAR "Worker berhenti"
        }
    }
TAMAT

-- Penggunaan
ch.hantar(WorkMsg::Kerja(data1))
ch.hantar(WorkMsg::Kerja(data2))
ch.hantar(WorkMsg::Berhenti)   -- signal hentikan
```

---

## Komunikasi Antara Actor {#komunikasi}

### Request-Response Pattern

```logicodex
-- Actor A (Requester) hantar request kepada Actor B (Responder)
-- dan tunggu response

STRUKTUR Request {
    id: U64,
    data: Text,
}

STRUKTUR Response {
    id: U64,
    result: Result<Text, Text>,
}

PELAKON Requester {
    req_ch: Channel<Request>,
    resp_ch: Channel<Response>,
}

PELAKON Responder {
    req_ch: Channel<Request>,
    resp_ch: Channel<Response>,
}

FUNGSI handle_responder(akt: &Responder, req: Request) -> Void
MULA
    BINA hasil SEBAGAI Text = proses_request(req.data)
    akt.resp_ch.hantar(Response {
        id: req.id,
        result: Result::Ok(hasil),
    })
TAMAT

FUNGSI handle_requester(akt: &Requester, resp: Response) -> Void
MULA
    PADAN resp.result {
        Result::Ok(data)  => PAPAR "Response: " + data,
        Result::Err(err)  => PAPAR "Error: " + err,
    }
TAMAT
```

### Pub-Sub Pattern

```logicodex
-- Multiple subscribers menerima mesej yang sama

PELAKON Publisher {
    subscribers: Vec<Channel<Event>>,
}

STRUKTUR Event {
    jenis: Text,
    data: Text,
}

FUNGSI publish(akt: &Publisher, event: Event) -> Void
MULA
    UNTUK sub DARI akt.subscribers
        sub.hantar(event)   -- hantar ke setiap subscriber
    TAMAT_UNTUK
TAMAT
```

### Pipeline Pattern

```logicodex
-- Data mengalir melalui beberapa actor secara berurutan

-- Actor 1: Baca file → Actor 2: Parse → Actor 3: Simpan

PELAKON FileReader {
    output: Channel<Text>,
}

PELAKON Parser {
    input: Channel<Text>,
    output: Channel<StructuredData>,
}

PELAKON DatabaseWriter {
    input: Channel<StructuredData>,
}

-- Rangkaian pipeline:
-- FileReader.output ──▶ Parser.input
-- Parser.output     ──▶ DatabaseWriter.input
```

---

## Latihan

1. Tulis actor `Timer` yang menghantar ping setiap 1 saat
2. Tulis actor `Logger` yang menerima log dan menulis ke fail
3. Tulis sistem pub-sub dengan 1 publisher dan 3 subscribers

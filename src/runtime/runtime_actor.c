/* =========================================================================
 * Logicodex Actor Runtime — C backend (pthread)
 * -------------------------------------------------------------------------
 * SECURITY / TRUST MODEL (read before editing):
 *   This file is a FIXED, AUDITED artifact. It is NOT generated from user
 *   input and user .ldx programs can never inject C here. The attack surface
 *   is exactly this reviewed code, nothing more.
 *
 *   This runtime is a C-specific BACKEND for one OS primitive: threading.
 *   It does NOT define Logicodex semantics. Actor ownership / capability /
 *   lifetime rules live in Logicodex (L1/L2); this file only does the
 *   mechanical pthread work. The backend is swappable (e.g. a future native
 *   Logicodex threading layer) without changing any semantics.
 *
 *   MINIMAL BOUNDARY — this runtime may ONLY:
 *     - create/join OS threads (pthread)
 *     - (later) channel buffers via mutex + condvar
 *   It must NEVER do file I/O, networking, process exec, or anything else.
 *
 * ERROR PROVENANCE (for the future Logicodex error-code system):
 *   Return values encode WHERE a failure originated so callers / the future
 *   error-code mapper can classify it. Negative = failure.
 *     >= 0                : success
 *     LX_ERR_C_RUNTIME    : failure inside this C runtime (e.g. bad args)
 *     LX_ERR_OS           : failure from the OS/pthread layer (errno-backed)
 *   (Logicodex-semantic and link/build origins are signalled elsewhere, not
 *   here — this file is the C-runtime/OS boundary only.)
 *
 * OBSERVABILITY:
 *   If the environment variable LOGICODEX_RUNTIME_TRACE is set (any value),
 *   spawn/join operations print a short trace line to stderr. Off by default.
 * ========================================================================= */

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Error provenance codes (negative). Kept small and explicit; the full
 * error-code system will map these to richer codes later. Each names the
 * LAYER the failure came from so the future mapper can classify it. */
#define LX_ERR_C_RUNTIME      (-101) /* origin: this C runtime (generic)     */
#define LX_ERR_OS             (-102) /* origin: OS/pthread (pthread_* failed) */
#define LX_ERR_INVALID_ENTRY  (-103) /* origin: C runtime — spawn(NULL)      */
#define LX_ERR_INVALID_HANDLE (-104) /* origin: C runtime — join(<=0)        */
#define LX_ERR_INVALID_ARG    (-105) /* origin: C runtime — bad arg (capacity/out) */

/* Opaque actor handle. ABI-1 PLATFORM ASSUMPTION (Linux x86_64): a pthread_t
 * fits in an int64_t and is reinterpreted as this handle. The static assert
 * makes a violating platform fail to compile rather than silently truncate.
 * A future portable design may replace this with an opaque numeric actor-id +
 * a runtime id->pthread_t table (deferred: adds global state/locking). */
typedef int64_t lx_actor_handle;

_Static_assert(sizeof(pthread_t) <= sizeof(int64_t),
               "pthread_t does not fit in int64_t (ABI-1 Linux x86_64 "
               "assumption violated; an opaque handle table would be required)");

/* Channel handle = a malloc'd channel struct pointer reinterpreted as i64.
 * This static assert guarantees the pointer fits the handle width. */
_Static_assert(sizeof(void *) <= sizeof(int64_t),
               "pointer does not fit in i64 channel handle");

/* An actor entry is a plain niladic function: `void __actor_<name>(void)`.
 * Logicodex lowers each actor body to exactly this shape, so the runtime
 * needs no knowledge of names, signatures, or program structure. */
typedef void (*lx_actor_entry)(void);

/* Trace helper — no-op unless LOGICODEX_RUNTIME_TRACE is set. Checked once. */
static int lx_trace_enabled(void) {
    static int cached = -1;
    if (cached == -1) {
        cached = getenv("LOGICODEX_RUNTIME_TRACE") != NULL ? 1 : 0;
    }
    return cached;
}

/* pthread's start_routine signature is `void *(*)(void *)`. We adapt the
 * Logicodex niladic entry through a tiny trampoline so the C runtime stays
 * the only place that knows about pthread's calling convention. */
static void *lx_actor_trampoline(void *arg) {
    lx_actor_entry entry = (lx_actor_entry)arg;
    if (entry != NULL) {
        entry();
    }
    return NULL;
}

/* logicodex_spawn(entry) -> i64
 *   entry : pointer to a `void (*)(void)` actor function (ABI-1).
 *   return: >= 0  an opaque actor handle (the pthread_t reinterpreted),
 *           < 0   a provenance-coded error (LX_ERR_*).
 *
 * NOTE: returning the thread handle as i64 keeps join simple and avoids a
 * global table (zero-trust: no shared mutable registry to corrupt). */
lx_actor_handle logicodex_spawn(void *entry) {
    if (entry == NULL) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] spawn: NULL entry -> INVALID_ENTRY\n");
        return LX_ERR_INVALID_ENTRY; /* provenance: C runtime */
    }
    pthread_t tid;
    int rc = pthread_create(&tid, NULL, lx_actor_trampoline, entry);
    if (rc != 0) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] spawn: pthread_create rc=%d -> OS error\n", rc);
        return LX_ERR_OS; /* provenance: OS/pthread */
    }
    if (lx_trace_enabled())
        fprintf(stderr, "[lx-rt] spawn: started actor, handle=%lu\n",
                (unsigned long)tid);
    /* Reinterpret the opaque pthread_t as the handle (see _Static_assert). */
    return (lx_actor_handle)tid;
}

/* logicodex_join(handle) -> i64
 *   handle: a value previously returned by logicodex_spawn (>= 0).
 *   return: 0 on success, or a provenance-coded error (LX_ERR_*). */
int64_t logicodex_join(lx_actor_handle handle) {
    /* join(0) and join(negative) are invalid: 0 is never a valid returned
     * handle in practice and negatives are error codes. No UB — fail cleanly. */
    if (handle <= 0) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] join: invalid handle %ld -> INVALID_HANDLE\n",
                    (long)handle);
        return LX_ERR_INVALID_HANDLE; /* provenance: C runtime */
    }
    pthread_t tid = (pthread_t)(uintptr_t)handle;
    int rc = pthread_join(tid, NULL);
    if (rc != 0) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] join: pthread_join rc=%d -> OS error\n", rc);
        return LX_ERR_OS; /* provenance: OS/pthread */
    }
    if (lx_trace_enabled())
        fprintf(stderr, "[lx-rt] join: actor handle=%lu joined\n",
                (unsigned long)tid);
    return 0;
}

/* =========================================================================
 * Channel B.1 — SPSC bounded blocking channel (i64 messages)
 * -------------------------------------------------------------------------
 * SCOPE (B.1, deliberately minimal — channel is where runtimes rot):
 *   - single producer, single consumer
 *   - bounded ring buffer of i64
 *   - BLOCKING send (waits while full), BLOCKING recv (waits while empty)
 *   NOT in B.1: timeout, close/drop/shutdown, select, MPSC, broadcast.
 *
 * MEMORY: create() malloc's the channel; B.1 has NO free/close (documented
 * leak — the OS reclaims at process exit). A half-baked free risks more races
 * than a small leak at exit, so it is deferred until a real close/drop design.
 *
 * HANDLE: the channel pointer is reinterpreted as an i64 handle (see the
 * _Static_assert above). The runtime never sees a channel name — codegen owns
 * the name->handle mapping (a channel handle lives in an ordinary variable).
 *
 * BOUNDARY: this code may only malloc + use pthread mutex/condvar + an i64
 * buffer. No file/network/exec.
 * ========================================================================= */

typedef struct {
    int64_t *buf;          /* ring buffer of capacity slots                 */
    int64_t capacity;      /* number of slots                               */
    int64_t count;         /* current number of queued items                */
    int64_t head;          /* next index to read                            */
    int64_t tail;          /* next index to write                           */
    pthread_mutex_t mutex; /* guards all fields above                       */
    pthread_cond_t not_full;  /* signalled when an item is removed          */
    pthread_cond_t not_empty; /* signalled when an item is added            */
} lx_channel;

/* logicodex_channel_create(capacity) -> i64 handle
 *   capacity : number of i64 slots (> 0).
 *   return   : >= 0 opaque handle (channel pointer as i64),
 *              < 0  provenance-coded error (LX_ERR_*). */
int64_t logicodex_channel_create(int64_t capacity) {
    if (capacity <= 0) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] channel_create: capacity %ld <= 0 -> INVALID_ARG\n",
                    (long)capacity);
        return LX_ERR_INVALID_ARG; /* provenance: C runtime */
    }
    /* Guard against absurd capacities that would overflow the allocation. */
    if ((uint64_t)capacity > (SIZE_MAX / sizeof(int64_t))) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] channel_create: capacity too large -> INVALID_ARG\n");
        return LX_ERR_INVALID_ARG; /* provenance: C runtime */
    }
    lx_channel *ch = (lx_channel *)malloc(sizeof(lx_channel));
    if (ch == NULL) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] channel_create: malloc failed -> C_RUNTIME\n");
        return LX_ERR_C_RUNTIME; /* provenance: C runtime */
    }
    ch->buf = (int64_t *)malloc((size_t)capacity * sizeof(int64_t));
    if (ch->buf == NULL) {
        free(ch);
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] channel_create: buffer malloc failed -> C_RUNTIME\n");
        return LX_ERR_C_RUNTIME; /* provenance: C runtime */
    }
    ch->capacity = capacity;
    ch->count = 0;
    ch->head = 0;
    ch->tail = 0;
    if (pthread_mutex_init(&ch->mutex, NULL) != 0
        || pthread_cond_init(&ch->not_full, NULL) != 0
        || pthread_cond_init(&ch->not_empty, NULL) != 0) {
        free(ch->buf);
        free(ch);
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] channel_create: pthread init failed -> OS\n");
        return LX_ERR_OS; /* provenance: OS/pthread */
    }
    if (lx_trace_enabled())
        fprintf(stderr, "[lx-rt] channel_create: capacity=%ld handle=%p\n",
                (long)capacity, (void *)ch);
    return (int64_t)(intptr_t)ch;
}

/* logicodex_channel_send(handle, value) -> i64 status
 *   Blocks while the channel is full. Returns 0 on success, or LX_ERR_*. */
int64_t logicodex_channel_send(int64_t handle, int64_t value) {
    if (handle <= 0) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] channel_send: invalid handle -> INVALID_HANDLE\n");
        return LX_ERR_INVALID_HANDLE; /* provenance: C runtime */
    }
    lx_channel *ch = (lx_channel *)(intptr_t)handle;
    pthread_mutex_lock(&ch->mutex);
    while (ch->count == ch->capacity) {
        pthread_cond_wait(&ch->not_full, &ch->mutex); /* blocking send */
    }
    ch->buf[ch->tail] = value;
    ch->tail = (ch->tail + 1) % ch->capacity;
    ch->count += 1;
    pthread_cond_signal(&ch->not_empty);
    pthread_mutex_unlock(&ch->mutex);
    if (lx_trace_enabled())
        fprintf(stderr, "[lx-rt] channel_send: value=%ld handle=%p\n",
                (long)value, (void *)ch);
    return 0;
}

/* logicodex_channel_recv(handle, out) -> i64 status
 *   Blocks while the channel is empty. Writes the received value to *out.
 *   Returns 0 on success, or LX_ERR_*. */
int64_t logicodex_channel_recv(int64_t handle, int64_t *out) {
    if (handle <= 0) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] channel_recv: invalid handle -> INVALID_HANDLE\n");
        return LX_ERR_INVALID_HANDLE; /* provenance: C runtime */
    }
    if (out == NULL) {
        if (lx_trace_enabled())
            fprintf(stderr, "[lx-rt] channel_recv: NULL out -> INVALID_ARG\n");
        return LX_ERR_INVALID_ARG; /* provenance: C runtime */
    }
    lx_channel *ch = (lx_channel *)(intptr_t)handle;
    pthread_mutex_lock(&ch->mutex);
    while (ch->count == 0) {
        pthread_cond_wait(&ch->not_empty, &ch->mutex); /* blocking recv */
    }
    int64_t value = ch->buf[ch->head];
    ch->head = (ch->head + 1) % ch->capacity;
    ch->count -= 1;
    pthread_cond_signal(&ch->not_full);
    pthread_mutex_unlock(&ch->mutex);
    *out = value;
    if (lx_trace_enabled())
        fprintf(stderr, "[lx-rt] channel_recv: value=%ld handle=%p\n",
                (long)value, (void *)ch);
    return 0;
}

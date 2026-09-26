/*
 * common.h — GPU-COBOL shared infrastructure used by the build:
 * version string, arena allocator, diagnostics.
 *
 * NOTE: include/gpu_cobol.h declares an overlapping diagnostic API
 * (GcDiagList, gc_diag_*). The build uses this header instead; no
 * translation unit includes both, so the definitions never collide.
 */
#ifndef GPU_COBOL_COMMON_H
#define GPU_COBOL_COMMON_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>

/* ------------------------------------------------------------------ */
/* Version */
/* ------------------------------------------------------------------ */
#define GC_VERSION_MAJOR 0
#define GC_VERSION_MINOR 1
#define GC_VERSION_PATCH 0
#define GC_VERSION_STRING "0.1.0"

/* ------------------------------------------------------------------ */
/* Arena allocator: all AST / IR / token text lives here. */
/* ------------------------------------------------------------------ */
typedef struct GcArenaChunk {
    struct GcArenaChunk *next;
    size_t size;
    size_t used;
    unsigned char data[];
} GcArenaChunk;

typedef struct GcArena {
    GcArenaChunk *head;
    size_t chunk_size;
    size_t total_allocated;
} GcArena;

GcArena *gc_arena_create(size_t chunk_size);
void gc_arena_destroy(GcArena *a);
/* Returns zero-initialised, 16-byte aligned memory. Aborts on OOM. */
void *gc_arena_alloc(GcArena *a, size_t size);
char *gc_arena_strdup(GcArena *a, const char *s);
char *gc_arena_strndup(GcArena *a, const char *s, size_t n);

/* ------------------------------------------------------------------ */
/* Diagnostics */
/* ------------------------------------------------------------------ */
typedef enum {
    GC_DIAG_NOTE,
    GC_DIAG_WARNING,
    GC_DIAG_ERROR,
    GC_DIAG_FATAL
} GcDiagLevel;

typedef struct {
    GcDiagLevel level;
    const char *filename;
    int line;
    int column;
    char message[512];
    char context[256];
} GcDiagnostic;

#define GC_MAX_DIAGNOSTICS 256

typedef struct {
    GcDiagnostic items[GC_MAX_DIAGNOSTICS];
    int count;
    int error_count;
    int warning_count;
} GcDiagList;

void gc_diag_init(GcDiagList *list);
/* list may be NULL (diagnostic is dropped, e.g. during lexer peek). */
void gc_diag_emit(GcDiagList *list, GcDiagLevel level,
                  const char *file, int line, int col,
                  const char *fmt, ...);
void gc_diag_print_all(const GcDiagList *list, FILE *out);
bool gc_diag_has_errors(const GcDiagList *list);

/* ------------------------------------------------------------------ */
/* Small string helpers */
/* ------------------------------------------------------------------ */
/* ASCII case-insensitive equality of s[0..n) with NUL-terminated word. */
bool gc_str_ieq_n(const char *s, size_t n, const char *word);

#endif /* GPU_COBOL_COMMON_H */

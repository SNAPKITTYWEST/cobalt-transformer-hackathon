#include "common.h"

#include <ctype.h>

/* ------------------------------------------------------------------ */
/* Arena */
/* ------------------------------------------------------------------ */
#define GC_ARENA_ALIGN 16

static GcArenaChunk *chunk_new(size_t size) {
    GcArenaChunk *c = (GcArenaChunk *)malloc(sizeof(GcArenaChunk) + size);
    if (!c) {
        fprintf(stderr, "gpu-cobol: out of memory\n");
        abort();
    }
    c->next = NULL;
    c->size = size;
    c->used = 0;
    return c;
}

GcArena *gc_arena_create(size_t chunk_size) {
    GcArena *a = (GcArena *)calloc(1, sizeof(GcArena));
    if (!a) {
        fprintf(stderr, "gpu-cobol: out of memory\n");
        abort();
    }
    a->chunk_size = chunk_size ? chunk_size : (1u << 16);
    a->head = chunk_new(a->chunk_size);
    return a;
}

void gc_arena_destroy(GcArena *a) {
    if (!a) return;
    GcArenaChunk *c = a->head;
    while (c) {
        GcArenaChunk *n = c->next;
        free(c);
        c = n;
    }
    free(a);
}

void *gc_arena_alloc(GcArena *a, size_t size) {
    if (size == 0) size = 1;
    size = (size + GC_ARENA_ALIGN - 1) & ~(size_t)(GC_ARENA_ALIGN - 1);
    GcArenaChunk *c = a->head;
    if (!c || c->used + size > c->size) {
        size_t csize = size > a->chunk_size ? size : a->chunk_size;
        GcArenaChunk *nc = chunk_new(csize);
        nc->next = a->head;
        a->head = nc;
        c = nc;
    }
    void *p = c->data + c->used;
    c->used += size;
    a->total_allocated += size;
    memset(p, 0, size);
    return p;
}

char *gc_arena_strndup(GcArena *a, const char *s, size_t n) {
    char *d = (char *)gc_arena_alloc(a, n + 1);
    if (n) memcpy(d, s, n);
    d[n] = '\0';
    return d;
}

char *gc_arena_strdup(GcArena *a, const char *s) {
    if (!s) return NULL;
    return gc_arena_strndup(a, s, strlen(s));
}

/* ------------------------------------------------------------------ */
/* Diagnostics */
/* ------------------------------------------------------------------ */
void gc_diag_init(GcDiagList *list) {
    memset(list, 0, sizeof(*list));
}

void gc_diag_emit(GcDiagList *list, GcDiagLevel level,
                  const char *file, int line, int col,
                  const char *fmt, ...) {
    if (!list) return;
    if (level >= GC_DIAG_ERROR) list->error_count++;
    else if (level == GC_DIAG_WARNING) list->warning_count++;
    if (list->count >= GC_MAX_DIAGNOSTICS) return;
    GcDiagnostic *d = &list->items[list->count++];
    d->level = level;
    d->filename = file;
    d->line = line;
    d->column = col;
    d->context[0] = '\0';
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(d->message, sizeof(d->message), fmt, ap);
    va_end(ap);
}

static const char *level_name(GcDiagLevel l) {
    switch (l) {
        case GC_DIAG_NOTE: return "note";
        case GC_DIAG_WARNING: return "warning";
        case GC_DIAG_ERROR: return "error";
        case GC_DIAG_FATAL: return "fatal error";
    }
    return "diagnostic";
}

void gc_diag_print_all(const GcDiagList *list, FILE *out) {
    if (!list) return;
    for (int i = 0; i < list->count; i++) {
        const GcDiagnostic *d = &list->items[i];
        fprintf(out, "%s:%d:%d: %s: %s\n",
                d->filename ? d->filename : "<unknown>",
                d->line, d->column, level_name(d->level), d->message);
        if (d->context[0])
            fprintf(out, "    %s\n", d->context);
    }
    if (list->count >= GC_MAX_DIAGNOSTICS)
        fprintf(out, "(diagnostic limit of %d reached; further ones suppressed)\n",
                GC_MAX_DIAGNOSTICS);
}

bool gc_diag_has_errors(const GcDiagList *list) {
    return list && list->error_count > 0;
}

/* ------------------------------------------------------------------ */
/* Strings */
/* ------------------------------------------------------------------ */
bool gc_str_ieq_n(const char *s, size_t n, const char *word) {
    size_t i = 0;
    for (; i < n; i++) {
        if (!word[i]) return false;
        if (toupper((unsigned char)s[i]) != toupper((unsigned char)word[i]))
            return false;
    }
    return word[i] == '\0';
}

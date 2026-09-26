/*
 * lexer.h — GPU-COBOL free-form lexer
 */
#ifndef GPU_COBOL_LEXER_H
#define GPU_COBOL_LEXER_H

#include "token.h"
#include "common.h"

typedef struct GcLexer {
    const char *src;
    size_t length;
    size_t pos;
    int line;
    int column;
    const char *filename;
    GcArena *arena;       /* token text / string values are copied here */
    GcDiagList *diags;
    /* keyword hash table (simple linear for now) */
    bool initialized;
    bool at_line_start;   /* next token is first on its line (level numbers) */
    bool expect_picture;  /* previous word was PIC/PICTURE [IS] */
} GcLexer;

void gc_lexer_init(GcLexer *lex, const char *src, size_t len,
                   const char *filename, GcArena *arena, GcDiagList *diags);
void gc_lexer_free(GcLexer *lex);

/* Advance and return next token. Caller may keep the token; string
 * values that need ownership are copied into token.u.str_val. */
GcToken gc_lexer_next(GcLexer *lex);

/* Peek without consuming */
GcToken gc_lexer_peek(GcLexer *lex);

/* Utility: dump all tokens (debug) */
void gc_lexer_dump_all(GcLexer *lex, FILE *out);

#endif /* GPU_COBOL_LEXER_H */

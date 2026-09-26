/*
 * parser.h — GPU-COBOL recursive-descent parser (two-token lookahead).
 */
#ifndef GPU_COBOL_PARSER_H
#define GPU_COBOL_PARSER_H

#include "common.h"
#include "lexer.h"
#include "ast.h"

typedef struct GcParser {
    GcLexer *lex;
    GcArena *arena;
    GcDiagList *diags;
    GcToken cur;   /* current token */
    GcToken next;  /* one-token lookahead */
    int errors;
} GcParser;

void gc_parser_init(GcParser *p, GcLexer *lex, GcArena *arena,
                    GcDiagList *diags);

/* Parse a whole compilation unit. Returns an AST_PROGRAM node (never NULL);
 * check the diagnostic list for errors. */
GcAstNode *gc_parser_parse(GcParser *p);

#endif /* GPU_COBOL_PARSER_H */

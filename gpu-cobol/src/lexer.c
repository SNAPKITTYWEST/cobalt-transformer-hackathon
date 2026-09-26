/*
 * lexer.c — GPU-COBOL free-form lexer.
 *
 *  - case-insensitive reserved words (COBOL + GPU extensions, see token.h)
 *  - identifiers may contain hyphens (never leading/trailing): BLOCK-SIZE
 *  - "*>" starts a comment that runs to end of line
 *  - integer, decimal (1.5) and string ('..' / "..", doubled-quote escape)
 *    literals; a trailing '.' followed by non-digit is a sentence period
 *  - an integer that is the first token on its line and is a valid level
 *    number (01-49, 66, 77, 88) is returned as TOK_LEVEL
 *  - after PIC / PICTURE [IS] the next blank-delimited run of characters is
 *    returned verbatim as TOK_PICTURE (e.g. "9(9)", "S9(4)V99")
 */
#include "lexer.h"

#include <ctype.h>

void gc_lexer_init(GcLexer *lex, const char *src, size_t len,
                   const char *filename, GcArena *arena, GcDiagList *diags) {
    memset(lex, 0, sizeof(*lex));
    lex->src = src;
    lex->length = len;
    lex->pos = 0;
    lex->line = 1;
    lex->column = 1;
    lex->filename = filename ? filename : "<input>";
    lex->arena = arena;
    lex->diags = diags;
    lex->initialized = true;
    lex->at_line_start = true;
    lex->expect_picture = false;
}

void gc_lexer_free(GcLexer *lex) {
    /* All token storage lives in the arena; nothing to release here. */
    lex->initialized = false;
}

static int cur(const GcLexer *lex) {
    return lex->pos < lex->length ? (unsigned char)lex->src[lex->pos] : -1;
}

static int at(const GcLexer *lex, size_t off) {
    size_t p = lex->pos + off;
    return p < lex->length ? (unsigned char)lex->src[p] : -1;
}

static void adv(GcLexer *lex) {
    if (lex->pos >= lex->length) return;
    if (lex->src[lex->pos] == '\n') {
        lex->line++;
        lex->column = 1;
        lex->at_line_start = true;
    } else {
        lex->column++;
    }
    lex->pos++;
}

static void skip_trivia(GcLexer *lex) {
    for (;;) {
        int c = cur(lex);
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == '\f' ||
            c == '\v') {
            adv(lex);
        } else if (c == '*' && at(lex, 1) == '>') {
            while (cur(lex) != -1 && cur(lex) != '\n') adv(lex);
        } else {
            return;
        }
    }
}

static bool is_word_start(int c) { return c != -1 && isalpha(c); }
static bool is_word_char(int c) { return c != -1 && (isalnum(c) || c == '_'); }

static GcToken make_tok(GcLexer *lex, GcTokenKind k, size_t start,
                        int line, int col) {
    GcToken t;
    memset(&t, 0, sizeof(t));
    t.kind = k;
    t.line = line;
    t.column = col;
    t.length = lex->pos - start;
    t.text = lex->arena
        ? gc_arena_strndup(lex->arena, lex->src + start, t.length)
        : lex->src + start;
    return t;
}

static bool is_level_value(int64_t v) {
    return (v >= 1 && v <= 49) || v == 66 || v == 77 || v == 88;
}

static GcToken lex_picture(GcLexer *lex, int line, int col) {
    size_t start = lex->pos;
    while (cur(lex) != -1 && !isspace(cur(lex))) {
        /* A '.' or ',' followed by blank/EOF ends the clause, not the PIC. */
        if ((cur(lex) == '.' || cur(lex) == ',') &&
            (at(lex, 1) == -1 || isspace(at(lex, 1))))
            break;
        adv(lex);
    }
    GcToken t = make_tok(lex, TOK_PICTURE, start, line, col);
    t.u.str_val = t.text;
    if (t.length == 0)
        gc_diag_emit(lex->diags, GC_DIAG_ERROR, lex->filename, line, col,
                     "expected picture string after PIC");
    return t;
}

static GcToken lex_word(GcLexer *lex, int line, int col) {
    size_t start = lex->pos;
    for (;;) {
        int c = cur(lex);
        if (is_word_char(c)) {
            adv(lex);
        } else if (c == '-' && is_word_char(at(lex, 1))) {
            /* hyphen inside a word: part of the name (BLOCK-SIZE) */
            adv(lex);
        } else {
            break;
        }
    }
    size_t len = lex->pos - start;
    GcTokenKind kind = gc_token_keyword_lookup(lex->src + start, len);
    GcToken t = make_tok(lex, kind, start, line, col);
    if (lex->arena) {
        char *up = (char *)t.text;
        for (size_t i = 0; i < len; i++)
            up[i] = (char)toupper((unsigned char)up[i]);
    }
    return t;
}

static GcToken lex_number(GcLexer *lex, int line, int col, bool first_on_line) {
    size_t start = lex->pos;
    while (cur(lex) != -1 && isdigit(cur(lex))) adv(lex);
    bool is_float = false;
    if (cur(lex) == '.' && at(lex, 1) != -1 && isdigit(at(lex, 1))) {
        is_float = true;
        adv(lex);
        while (cur(lex) != -1 && isdigit(cur(lex))) adv(lex);
    }
    if ((cur(lex) == 'e' || cur(lex) == 'E') &&
        (isdigit(at(lex, 1)) ||
         ((at(lex, 1) == '+' || at(lex, 1) == '-') && isdigit(at(lex, 2))))) {
        is_float = true;
        adv(lex);
        if (cur(lex) == '+' || cur(lex) == '-') adv(lex);
        while (cur(lex) != -1 && isdigit(cur(lex))) adv(lex);
    }
    if (is_word_start(cur(lex)) || cur(lex) == '_') {
        /* e.g. "3D-ARRAY": COBOL names may start with a digit */
        while (is_word_char(cur(lex)) ||
               (cur(lex) == '-' && is_word_char(at(lex, 1))))
            adv(lex);
        GcToken t = make_tok(lex, TOK_IDENT, start, line, col);
        char *up = (char *)t.text;
        if (lex->arena)
            for (size_t i = 0; i < t.length; i++)
                up[i] = (char)toupper((unsigned char)up[i]);
        return t;
    }
    GcToken t = make_tok(lex, is_float ? TOK_FLOAT : TOK_INT, start, line, col);
    if (is_float) {
        t.u.float_val = strtod(t.text, NULL);
    } else {
        t.u.int_val = (int64_t)strtoll(t.text, NULL, 10);
        if (first_on_line && t.length <= 2 && is_level_value(t.u.int_val))
            t.kind = TOK_LEVEL;
    }
    return t;
}

static GcToken lex_string(GcLexer *lex, int line, int col) {
    size_t start = lex->pos;
    int q = cur(lex);
    adv(lex);
    size_t cap = 64, n = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) abort();
    bool closed = false;
    while (cur(lex) != -1 && cur(lex) != '\n') {
        int c = cur(lex);
        if (c == q) {
            if (at(lex, 1) == q) { /* doubled quote = literal quote */
                adv(lex);
            } else {
                adv(lex);
                closed = true;
                break;
            }
        }
        if (n + 1 >= cap) {
            cap *= 2;
            char *nb = (char *)realloc(buf, cap);
            if (!nb) abort();
            buf = nb;
        }
        buf[n++] = (char)c;
        adv(lex);
    }
    if (!closed)
        gc_diag_emit(lex->diags, GC_DIAG_ERROR, lex->filename, line, col,
                     "unterminated string literal");
    GcToken t = make_tok(lex, TOK_STRING, start, line, col);
    t.u.str_val = lex->arena ? gc_arena_strndup(lex->arena, buf, n) : "";
    free(buf);
    return t;
}

GcToken gc_lexer_next(GcLexer *lex) {
    skip_trivia(lex);
    bool first_on_line = lex->at_line_start;
    lex->at_line_start = false;
    int line = lex->line, col = lex->column;
    size_t start = lex->pos;
    int c = cur(lex);

    if (c == -1) {
        GcToken t;
        memset(&t, 0, sizeof(t));
        t.kind = TOK_EOF;
        t.text = "";
        t.line = line;
        t.column = col;
        return t;
    }

    if (lex->expect_picture) {
        /* "PIC IS 9(4)": IS is still a keyword */
        if (is_word_start(c)) {
            size_t p = lex->pos;
            size_t e = p;
            while (e < lex->length && isalpha((unsigned char)lex->src[e])) e++;
            if (gc_str_ieq_n(lex->src + p, e - p, "IS") &&
                (e >= lex->length || isspace((unsigned char)lex->src[e]))) {
                return lex_word(lex, line, col); /* keep expect_picture */
            }
        }
        lex->expect_picture = false;
        return lex_picture(lex, line, col);
    }

    if (is_word_start(c)) {
        GcToken t = lex_word(lex, line, col);
        if (t.kind == TOK_PIC || t.kind == TOK_PICTURE_KW)
            lex->expect_picture = true;
        return t;
    }
    if (isdigit(c))
        return lex_number(lex, line, col, first_on_line);
    if (c == '\'' || c == '"')
        return lex_string(lex, line, col);

    GcTokenKind k = TOK_ERROR;
    switch (c) {
        case '.': k = TOK_PERIOD; break;
        case ',': k = TOK_COMMA; break;
        case ':': k = TOK_COLON; break;
        case ';': /* COBOL separator: treat like whitespace */
            adv(lex);
            return gc_lexer_next(lex);
        case '(': k = TOK_LPAREN; break;
        case ')': k = TOK_RPAREN; break;
        case '+': k = TOK_PLUS; break;
        case '-': k = TOK_MINUS; break;
        case '/': k = TOK_SLASH; break;
        case '=': k = TOK_EQ; break;
        case '*':
            if (at(lex, 1) == '*') { adv(lex); k = TOK_POWER; }
            else k = TOK_STAR;
            break;
        case '<':
            if (at(lex, 1) == '=') { adv(lex); k = TOK_LE; }
            else if (at(lex, 1) == '>') { adv(lex); k = TOK_NE; }
            else k = TOK_LT;
            break;
        case '>':
            if (at(lex, 1) == '=') { adv(lex); k = TOK_GE; }
            else k = TOK_GT;
            break;
        default:
            break;
    }
    adv(lex);
    GcToken t = make_tok(lex, k, start, line, col);
    if (k == TOK_ERROR)
        gc_diag_emit(lex->diags, GC_DIAG_ERROR, lex->filename, line, col,
                     "unexpected character '%c' (0x%02X)",
                     isprint(c) ? c : '?', (unsigned)c);
    return t;
}

GcToken gc_lexer_peek(GcLexer *lex) {
    GcLexer saved = *lex;
    lex->diags = NULL; /* do not report errors twice */
    GcToken t = gc_lexer_next(lex);
    *lex = saved;
    return t;
}

void gc_lexer_dump_all(GcLexer *lex, FILE *out) {
    for (;;) {
        GcToken t = gc_lexer_next(lex);
        gc_token_print(&t, out);
        if (t.kind == TOK_EOF) break;
    }
}

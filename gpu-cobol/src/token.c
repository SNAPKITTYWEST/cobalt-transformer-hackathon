#include "token.h"

static const char *const kind_names[TOK_COUNT__] = {
#define GC_X_NAME(k, s) [k] = s,
    GC_TOKEN_BASIC_LIST(GC_X_NAME)
    GC_KEYWORD_LIST(GC_X_NAME)
#undef GC_X_NAME
};

typedef struct {
    const char *word;
    GcTokenKind kind;
} KeywordEntry;

static const KeywordEntry keyword_table[] = {
#define GC_X_KW(k, s) { s, k },
    GC_KEYWORD_LIST(GC_X_KW)
#undef GC_X_KW
};

const char *gc_token_kind_name(GcTokenKind kind) {
    if ((int)kind < 0 || kind >= TOK_COUNT__ || !kind_names[kind])
        return "<?>";
    return kind_names[kind];
}

bool gc_token_is_keyword(GcTokenKind kind) {
    return kind > TOK_KEYWORD_BEFORE__ && kind < TOK_COUNT__;
}

GcTokenKind gc_token_keyword_lookup(const char *word, size_t len) {
    size_t n = sizeof(keyword_table) / sizeof(keyword_table[0]);
    for (size_t i = 0; i < n; i++) {
        if (gc_str_ieq_n(word, len, keyword_table[i].word))
            return keyword_table[i].kind;
    }
    return TOK_IDENT;
}

void gc_token_print(const GcToken *tok, FILE *out) {
    fprintf(out, "%4d:%-3d %-16s", tok->line, tok->column,
            gc_token_kind_name(tok->kind));
    switch (tok->kind) {
        case TOK_INT:
        case TOK_LEVEL:
            fprintf(out, " %lld", (long long)tok->u.int_val);
            break;
        case TOK_FLOAT:
            fprintf(out, " %g", tok->u.float_val);
            break;
        case TOK_STRING:
            fprintf(out, " \"%s\"", tok->u.str_val ? tok->u.str_val : "");
            break;
        case TOK_PICTURE:
            fprintf(out, " %s", tok->u.str_val ? tok->u.str_val : "");
            break;
        case TOK_IDENT:
            fprintf(out, " %s", tok->text ? tok->text : "");
            break;
        default:
            break;
    }
    fputc('\n', out);
}

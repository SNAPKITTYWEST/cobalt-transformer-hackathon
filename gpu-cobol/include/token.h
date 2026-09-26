/*
 * token.h — GPU-COBOL token kinds and token structure.
 *
 * Token kinds are generated from two X-macro lists:
 *   GC_TOKEN_BASIC_LIST  — literals, identifiers, punctuation, operators
 *   GC_KEYWORD_LIST      — reserved words (COBOL + GPU extensions);
 *                          the second column is the canonical spelling
 *                          used both by the lexer's keyword table and by
 *                          gc_token_kind_name().
 */
#ifndef GPU_COBOL_TOKEN_H
#define GPU_COBOL_TOKEN_H

#include "common.h"

#define GC_TOKEN_BASIC_LIST(X)          \
    X(TOK_EOF,        "<eof>")          \
    X(TOK_ERROR,      "<error>")        \
    X(TOK_IDENT,      "<identifier>")   \
    X(TOK_INT,        "<integer>")      \
    X(TOK_FLOAT,      "<float>")        \
    X(TOK_STRING,     "<string>")       \
    X(TOK_LEVEL,      "<level-number>") \
    X(TOK_PICTURE,    "<picture>")      \
    X(TOK_PERIOD,     ".")              \
    X(TOK_COMMA,      ",")              \
    X(TOK_COLON,      ":")              \
    X(TOK_LPAREN,     "(")              \
    X(TOK_RPAREN,     ")")              \
    X(TOK_PLUS,       "+")              \
    X(TOK_MINUS,      "-")              \
    X(TOK_STAR,       "*")              \
    X(TOK_SLASH,      "/")              \
    X(TOK_POWER,      "**")             \
    X(TOK_EQ,         "=")              \
    X(TOK_NE,         "<>")             \
    X(TOK_LT,         "<")              \
    X(TOK_LE,         "<=")             \
    X(TOK_GT,         ">")              \
    X(TOK_GE,         ">=")

#define GC_KEYWORD_LIST(X)                          \
    /* --- divisions / sections / paragraphs --- */ \
    X(TOK_IDENTIFICATION,  "IDENTIFICATION")        \
    X(TOK_ID,              "ID")                    \
    X(TOK_DIVISION,        "DIVISION")              \
    X(TOK_PROGRAM_ID,      "PROGRAM-ID")            \
    X(TOK_ENVIRONMENT,     "ENVIRONMENT")           \
    X(TOK_CONFIGURATION,   "CONFIGURATION")         \
    X(TOK_DATA,            "DATA")                  \
    X(TOK_WORKING_STORAGE, "WORKING-STORAGE")       \
    X(TOK_LOCAL_STORAGE,   "LOCAL-STORAGE")         \
    X(TOK_LINKAGE,         "LINKAGE")               \
    X(TOK_SECTION,         "SECTION")               \
    X(TOK_PROCEDURE,       "PROCEDURE")             \
    X(TOK_USING,           "USING")                 \
    /* --- data description --- */                 \
    X(TOK_PIC,             "PIC")                   \
    X(TOK_PICTURE_KW,      "PICTURE")               \
    X(TOK_IS,              "IS")                    \
    X(TOK_USAGE,           "USAGE")                 \
    X(TOK_COMP,            "COMP")                  \
    X(TOK_COMP_1,          "COMP-1")                \
    X(TOK_COMP_2,          "COMP-2")                \
    X(TOK_COMP_3,          "COMP-3")                \
    X(TOK_COMP_4,          "COMP-4")                \
    X(TOK_COMP_5,          "COMP-5")                \
    X(TOK_COMPUTATIONAL,   "COMPUTATIONAL")         \
    X(TOK_BINARY,          "BINARY")                \
    X(TOK_VALUE,           "VALUE")                 \
    X(TOK_OCCURS,          "OCCURS")                \
    X(TOK_TIMES,           "TIMES")                 \
    X(TOK_FILLER,          "FILLER")                \
    /* --- procedure statements --- */              \
    X(TOK_COMPUTE,         "COMPUTE")               \
    X(TOK_ROUNDED,         "ROUNDED")               \
    X(TOK_SET,             "SET")                   \
    X(TOK_TO,              "TO")                    \
    X(TOK_MOVE,            "MOVE")                  \
    X(TOK_IF,              "IF")                    \
    X(TOK_THEN,            "THEN")                  \
    X(TOK_ELSE,            "ELSE")                  \
    X(TOK_END_IF,          "END-IF")                \
    X(TOK_CONTINUE,        "CONTINUE")              \
    X(TOK_GOBACK,          "GOBACK")                \
    X(TOK_STOP,            "STOP")                  \
    X(TOK_RUN,             "RUN")                   \
    X(TOK_EXIT,            "EXIT")                  \
    X(TOK_PROGRAM,         "PROGRAM")               \
    X(TOK_END,             "END")                   \
    X(TOK_AND,             "AND")                   \
    X(TOK_OR,              "OR")                    \
    X(TOK_NOT,             "NOT")                   \
    /* --- GPU extensions: kernel declaration --- */ \
    X(TOK_KERNEL,          "KERNEL")                \
    X(TOK_PARAMETER,       "PARAMETER")             \
    X(TOK_AS,              "AS")                    \
    X(TOK_GPU_POINTER,     "GPU-POINTER")           \
    X(TOK_GPU_I32,         "GPU-I32")               \
    X(TOK_GPU_I64,         "GPU-I64")               \
    X(TOK_GPU_F16,         "GPU-F16")               \
    X(TOK_GPU_F32,         "GPU-F32")               \
    X(TOK_GPU_F64,         "GPU-F64")               \
    X(TOK_GPU_MASK,        "GPU-MASK")              \
    X(TOK_INT32,           "INT32")                 \
    X(TOK_INT64,           "INT64")                 \
    X(TOK_FLOAT16,         "FLOAT16")               \
    X(TOK_FLOAT32,         "FLOAT32")               \
    X(TOK_FLOAT64,         "FLOAT64")               \
    /* --- GPU extensions: memory / compute --- */  \
    X(TOK_GPU_LOAD,        "GPU-LOAD")              \
    X(TOK_GPU_STORE,       "GPU-STORE")             \
    X(TOK_GPU_DOT,         "GPU-DOT")               \
    X(TOK_GPU_REDUCE_SUM,  "GPU-REDUCE-SUM")        \
    X(TOK_GPU_REDUCE_MAX,  "GPU-REDUCE-MAX")        \
    X(TOK_GPU_REDUCE_MIN,  "GPU-REDUCE-MIN")        \
    X(TOK_GPU_SYNC,        "GPU-SYNC")              \
    X(TOK_PROGRAM_ID_X,    "PROGRAM-ID-X")          \
    X(TOK_PROGRAM_ID_Y,    "PROGRAM-ID-Y")          \
    X(TOK_PROGRAM_ID_Z,    "PROGRAM-ID-Z")          \
    X(TOK_LANE_ID,         "LANE-ID")               \
    X(TOK_MASK,            "MASK")                  \
    X(TOK_OFFSETS,         "OFFSETS")               \
    X(TOK_SHAPE,           "SHAPE")                 \
    X(TOK_OTHER,           "OTHER")                 \
    X(TOK_INTO,            "INTO")                  \
    X(TOK_WITH,            "WITH")                  \
    X(TOK_AXIS,            "AXIS")

typedef enum {
#define GC_X_ENUM(k, s) k,
    GC_TOKEN_BASIC_LIST(GC_X_ENUM)
    TOK_KEYWORD_FIRST__,
    TOK_KEYWORD_BEFORE__ = TOK_KEYWORD_FIRST__ - 1,
    GC_KEYWORD_LIST(GC_X_ENUM)
#undef GC_X_ENUM
    TOK_COUNT__
} GcTokenKind;

typedef struct GcToken {
    GcTokenKind kind;
    /* Source text of the token (arena copy). Words (keywords and
     * identifiers) are canonicalised to upper case. */
    const char *text;
    size_t length;
    int line;
    int column;
    union {
        int64_t int_val;      /* TOK_INT, TOK_LEVEL */
        double float_val;     /* TOK_FLOAT */
        const char *str_val;  /* TOK_STRING (unquoted), TOK_PICTURE */
    } u;
} GcToken;

const char *gc_token_kind_name(GcTokenKind kind);
bool gc_token_is_keyword(GcTokenKind kind);
/* Look up a reserved word (case-insensitive); returns TOK_IDENT if none. */
GcTokenKind gc_token_keyword_lookup(const char *word, size_t len);
void gc_token_print(const GcToken *tok, FILE *out);

#endif /* GPU_COBOL_TOKEN_H */

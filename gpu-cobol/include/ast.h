/*
 * ast.h — GPU-COBOL abstract syntax tree.
 *
 * Tree shape produced by the parser (consumed by sema.c):
 *
 *   AST_PROGRAM                 u.program {program_name, data_div,
 *                                          kernel_sec, proc_div}
 *     AST_DATA_DIV              children: AST_DATA_ITEM
 *     AST_KERNEL_SECTION        children: AST_KERNEL_DEF
 *       AST_KERNEL_DEF          u.kernel {name, params[], n_params}
 *         AST_PARAM             u.param {name, type_name, is_pointer}
 *     AST_PROC_DIV              children: statements
 *
 *   Statements:
 *     AST_STMT_COMPUTE/SET/MOVE u.assign {lhs, rhs}
 *     AST_STMT_GPU_LOAD         children[0] = ptr;
 *                               u.memop {ptr, offsets, shape, mask, other,
 *                                        dest = INTO target}
 *     AST_STMT_GPU_STORE        children[0] = ptr;
 *                               u.memop {ptr, offsets, mask,
 *                                        dest = VALUE expression}
 *     AST_STMT_GPU_DOT          children {a, b, dest}
 *     AST_STMT_GPU_REDUCE       children {value[, dest]}; u.reduce {op, axis}
 *     AST_STMT_GPU_SYNC, AST_STMT_GOBACK
 *     AST_STMT_IF               u.ifstmt {cond, then_body, else_body}
 *                               (bodies are AST_STMT_LIST)
 *     AST_STMT_LIST             children: statements
 */
#ifndef GPU_COBOL_AST_H
#define GPU_COBOL_AST_H

#include "common.h"
#include "token.h"

typedef enum {
    AST_PROGRAM,
    AST_DATA_DIV,
    AST_DATA_ITEM,
    AST_KERNEL_SECTION,
    AST_KERNEL_DEF,
    AST_PARAM,
    AST_PROC_DIV,

    AST_STMT_LIST,
    AST_STMT_COMPUTE,
    AST_STMT_SET,
    AST_STMT_MOVE,
    AST_STMT_IF,
    AST_STMT_GPU_LOAD,
    AST_STMT_GPU_STORE,
    AST_STMT_GPU_DOT,
    AST_STMT_GPU_REDUCE,
    AST_STMT_GPU_SYNC,
    AST_STMT_GOBACK,

    AST_EXPR_LITERAL,
    AST_EXPR_IDENT,
    AST_EXPR_PROGRAM_ID,
    AST_EXPR_LANE_ID,
    AST_EXPR_BINARY,
    AST_EXPR_UNARY,

    AST_KIND_COUNT__
} GcAstKind;

typedef struct GcAstNode GcAstNode;

struct GcAstNode {
    GcAstKind kind;
    int line;
    int column;

    GcAstNode **children;
    int n_children;
    int cap_children;

    union {
        struct {
            GcTokenKind lit_kind;   /* TOK_INT, TOK_FLOAT, TOK_STRING */
            int64_t int_val;
            double float_val;
            const char *str_val;
        } literal;
        struct {
            const char *name;
        } ident;
        struct {
            int axis;               /* 0 = X, 1 = Y, 2 = Z */
        } program_id;
        struct {
            GcTokenKind op;
            GcAstNode *left;
            GcAstNode *right;
        } binary;
        struct {
            GcTokenKind op;         /* TOK_MINUS or TOK_NOT */
            GcAstNode *operand;
        } unary;
        struct {
            GcAstNode *ptr;
            GcAstNode *offsets;
            GcAstNode *shape;
            GcAstNode *mask;
            GcAstNode *other;
            GcAstNode *dest;        /* LOAD: INTO target; STORE: VALUE expr */
        } memop;
        struct {
            GcTokenKind op;         /* TOK_GPU_REDUCE_SUM/MAX/MIN */
            int axis;
        } reduce;
        struct {
            GcAstNode *lhs;
            GcAstNode *rhs;
        } assign;
        struct {
            GcAstNode *cond;
            GcAstNode *then_body;
            GcAstNode *else_body;
        } ifstmt;
        struct {
            const char *name;
            GcAstNode **params;
            int n_params;
        } kernel;
        struct {
            const char *name;
            const char *type_name;  /* canonical upper-case type keyword */
            int is_pointer;
        } param;
        struct {
            int level;
            const char *name;       /* NULL for FILLER */
            const char *pic;
            const char *usage;
            GcAstNode *value;       /* AST_EXPR_LITERAL or NULL */
            int occurs;
        } data_item;
        struct {
            const char *program_name;
            GcAstNode *data_div;
            GcAstNode *kernel_sec;
            GcAstNode *proc_div;
        } program;
    } u;
};

GcAstNode *gc_ast_new(GcArena *a, GcAstKind kind, int line, int column);
void gc_ast_add_child(GcArena *a, GcAstNode *parent, GcAstNode *child);
const char *gc_ast_kind_name(GcAstKind kind);
void gc_ast_print(const GcAstNode *node, int indent, FILE *out);

#endif /* GPU_COBOL_AST_H */

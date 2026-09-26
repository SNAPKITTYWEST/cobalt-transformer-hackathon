/*
 * parser.c — GPU-COBOL recursive-descent parser.
 *
 * Grammar (free-form, periods terminate sentences):
 *
 *   program      := [IDENTIFICATION DIVISION . PROGRAM-ID . name . ...]
 *                   [ENVIRONMENT DIVISION . ...]
 *                   { DATA DIVISION .
 *                   | (WORKING-STORAGE|LOCAL-STORAGE|LINKAGE) SECTION .
 *                   | data-item
 *                   | KERNEL SECTION . { kernel-def } }
 *                   [PROCEDURE DIVISION [USING ...] . { statement [.] }]
 *                   [END PROGRAM name .]
 *   data-item    := LEVEL (name|FILLER) { PIC pic | USAGE [IS] usage | usage
 *                   | VALUE [IS] literal | OCCURS n [TIMES] } .
 *   kernel-def   := KERNEL name { PARAMETER name AS type-spec [,] } .
 *   type-spec    := GPU-POINTER [elem-type] | elem-type
 *   statement    := COMPUTE target [ROUNDED] = expr
 *                 | SET target TO expr
 *                 | MOVE expr TO target
 *                 | GPU-LOAD ptr {OFFSETS e | SHAPE e | MASK e | OTHER e}
 *                            INTO target
 *                 | GPU-STORE ptr {OFFSETS e | VALUE e | MASK e}
 *                 | GPU-DOT e [WITH|,] e INTO target
 *                 | GPU-REDUCE-(SUM|MAX|MIN) e [AXIS int] [INTO target]
 *                 | GPU-SYNC
 *                 | IF expr [THEN] stmts [ELSE stmts] (END-IF | .)
 *                 | GOBACK | STOP RUN | EXIT PROGRAM | CONTINUE
 *
 *   expr precedence (low -> high):
 *     OR < AND < NOT < comparison (= <> < <= > >=, NOT =)
 *        < + - < * / < ** (right assoc) < unary +/- < primary
 */
#include "parser.h"

/* ------------------------------------------------------------------ */
/* Token plumbing */
/* ------------------------------------------------------------------ */
static void advance(GcParser *p) {
    p->cur = p->next;
    if (p->cur.kind != TOK_EOF)
        p->next = gc_lexer_next(p->lex);
}

static bool check(const GcParser *p, GcTokenKind k) { return p->cur.kind == k; }

static bool accept(GcParser *p, GcTokenKind k) {
    if (p->cur.kind == k) {
        advance(p);
        return true;
    }
    return false;
}

static const char *tok_desc(const GcToken *t) {
    if (t->kind == TOK_IDENT || t->kind == TOK_INT || t->kind == TOK_FLOAT ||
        t->kind == TOK_LEVEL || t->kind == TOK_PICTURE)
        return t->text ? t->text : gc_token_kind_name(t->kind);
    return gc_token_kind_name(t->kind);
}

static void error_at(GcParser *p, const GcToken *t, const char *fmt,
                     const char *arg) {
    p->errors++;
    char msg[400];
    snprintf(msg, sizeof(msg), fmt, arg);
    gc_diag_emit(p->diags, GC_DIAG_ERROR, p->lex->filename, t->line, t->column,
                 "%s (found '%s')", msg, tok_desc(t));
}

static bool expect(GcParser *p, GcTokenKind k, const char *context) {
    if (accept(p, k)) return true;
    char buf[200];
    snprintf(buf, sizeof(buf), "expected '%s' %s", gc_token_kind_name(k),
             context ? context : "");
    error_at(p, &p->cur, "%s", buf);
    return false;
}

/* Skip to just past the next sentence period (error recovery). */
static void sync_to_period(GcParser *p) {
    while (!check(p, TOK_EOF) && !check(p, TOK_PERIOD)) advance(p);
    accept(p, TOK_PERIOD);
}

static bool is_division_start(const GcParser *p) {
    return (p->cur.kind == TOK_IDENTIFICATION || p->cur.kind == TOK_ID ||
            p->cur.kind == TOK_ENVIRONMENT || p->cur.kind == TOK_DATA ||
            p->cur.kind == TOK_PROCEDURE) &&
           p->next.kind == TOK_DIVISION;
}

static bool is_kernel_section(const GcParser *p) {
    return p->cur.kind == TOK_KERNEL && p->next.kind == TOK_SECTION;
}

/* Keywords that may also be used as data names in expression / target
 * position (e.g. "SET MASK TO ...", "MASK MASK"). */
static bool is_soft_keyword(GcTokenKind k) {
    switch (k) {
        case TOK_MASK:
        case TOK_SHAPE:
        case TOK_OTHER:
        case TOK_AXIS:
        case TOK_DATA:
        case TOK_ID:
            return true;
        default:
            return false;
    }
}

static GcAstNode *node(GcParser *p, GcAstKind k, const GcToken *at) {
    return gc_ast_new(p->arena, k, at->line, at->column);
}

/* ------------------------------------------------------------------ */
/* Expressions */
/* ------------------------------------------------------------------ */
static GcAstNode *parse_expr(GcParser *p);

static GcAstNode *make_ident(GcParser *p, const GcToken *t) {
    GcAstNode *n = node(p, AST_EXPR_IDENT, t);
    n->u.ident.name = t->text;
    return n;
}

static GcAstNode *parse_target(GcParser *p, const char *context) {
    if (check(p, TOK_IDENT) || is_soft_keyword(p->cur.kind)) {
        GcToken t = p->cur;
        advance(p);
        return make_ident(p, &t);
    }
    error_at(p, &p->cur, "expected identifier %s", context);
    return NULL;
}

static GcAstNode *parse_primary(GcParser *p) {
    GcToken t = p->cur;
    switch (t.kind) {
        case TOK_INT: {
            advance(p);
            GcAstNode *n = node(p, AST_EXPR_LITERAL, &t);
            n->u.literal.lit_kind = TOK_INT;
            n->u.literal.int_val = t.u.int_val;
            n->u.literal.float_val = (double)t.u.int_val;
            return n;
        }
        case TOK_FLOAT: {
            advance(p);
            GcAstNode *n = node(p, AST_EXPR_LITERAL, &t);
            n->u.literal.lit_kind = TOK_FLOAT;
            n->u.literal.float_val = t.u.float_val;
            n->u.literal.int_val = (int64_t)t.u.float_val;
            return n;
        }
        case TOK_STRING: {
            advance(p);
            GcAstNode *n = node(p, AST_EXPR_LITERAL, &t);
            n->u.literal.lit_kind = TOK_STRING;
            n->u.literal.str_val = t.u.str_val;
            return n;
        }
        case TOK_PROGRAM_ID_X:
        case TOK_PROGRAM_ID_Y:
        case TOK_PROGRAM_ID_Z: {
            advance(p);
            GcAstNode *n = node(p, AST_EXPR_PROGRAM_ID, &t);
            n->u.program_id.axis = (int)(t.kind - TOK_PROGRAM_ID_X);
            return n;
        }
        case TOK_LANE_ID:
            advance(p);
            return node(p, AST_EXPR_LANE_ID, &t);
        case TOK_LPAREN: {
            advance(p);
            GcAstNode *e = parse_expr(p);
            expect(p, TOK_RPAREN, "to close parenthesised expression");
            return e;
        }
        case TOK_IDENT:
            advance(p);
            return make_ident(p, &t);
        default:
            if (is_soft_keyword(t.kind)) {
                advance(p);
                return make_ident(p, &t);
            }
            error_at(p, &t, "%s", "expected expression");
            return NULL;
    }
}

static GcAstNode *make_binary(GcParser *p, const GcToken *op, GcAstNode *l,
                              GcAstNode *r) {
    GcAstNode *n = node(p, AST_EXPR_BINARY, op);
    n->u.binary.op = op->kind;
    n->u.binary.left = l;
    n->u.binary.right = r;
    return n;
}

static GcAstNode *parse_unary(GcParser *p) {
    if (check(p, TOK_MINUS) || check(p, TOK_PLUS)) {
        GcToken op = p->cur;
        advance(p);
        GcAstNode *operand = parse_unary(p);
        if (op.kind == TOK_PLUS) return operand;
        /* fold negative numeric literals */
        if (operand && operand->kind == AST_EXPR_LITERAL &&
            operand->u.literal.lit_kind != TOK_STRING) {
            operand->u.literal.int_val = -operand->u.literal.int_val;
            operand->u.literal.float_val = -operand->u.literal.float_val;
            return operand;
        }
        GcAstNode *n = node(p, AST_EXPR_UNARY, &op);
        n->u.unary.op = TOK_MINUS;
        n->u.unary.operand = operand;
        return n;
    }
    return parse_primary(p);
}

static GcAstNode *parse_power(GcParser *p) {
    GcAstNode *l = parse_unary(p);
    if (check(p, TOK_POWER)) {
        GcToken op = p->cur;
        advance(p);
        GcAstNode *r = parse_power(p); /* right associative */
        return make_binary(p, &op, l, r);
    }
    return l;
}

static GcAstNode *parse_mul(GcParser *p) {
    GcAstNode *l = parse_power(p);
    while (check(p, TOK_STAR) || check(p, TOK_SLASH)) {
        GcToken op = p->cur;
        advance(p);
        l = make_binary(p, &op, l, parse_power(p));
    }
    return l;
}

static GcAstNode *parse_add(GcParser *p) {
    GcAstNode *l = parse_mul(p);
    while (check(p, TOK_PLUS) || check(p, TOK_MINUS)) {
        GcToken op = p->cur;
        advance(p);
        l = make_binary(p, &op, l, parse_mul(p));
    }
    return l;
}

static bool is_relop(GcTokenKind k) {
    return k == TOK_EQ || k == TOK_NE || k == TOK_LT || k == TOK_LE ||
           k == TOK_GT || k == TOK_GE;
}

static GcAstNode *parse_compare(GcParser *p) {
    GcAstNode *l = parse_add(p);
    if (is_relop(p->cur.kind)) {
        GcToken op = p->cur;
        advance(p);
        return make_binary(p, &op, l, parse_add(p));
    }
    /* "a NOT = b", "a NOT < b", "a NOT > b" */
    if (check(p, TOK_NOT) && is_relop(p->next.kind)) {
        advance(p);
        GcToken op = p->cur;
        advance(p);
        GcAstNode *r = parse_add(p);
        switch (op.kind) {
            case TOK_EQ: op.kind = TOK_NE; break;
            case TOK_NE: op.kind = TOK_EQ; break;
            case TOK_LT: op.kind = TOK_GE; break;
            case TOK_LE: op.kind = TOK_GT; break;
            case TOK_GT: op.kind = TOK_LE; break;
            case TOK_GE: op.kind = TOK_LT; break;
            default: break;
        }
        return make_binary(p, &op, l, r);
    }
    return l;
}

static GcAstNode *parse_not(GcParser *p) {
    if (check(p, TOK_NOT)) {
        GcToken op = p->cur;
        advance(p);
        GcAstNode *n = node(p, AST_EXPR_UNARY, &op);
        n->u.unary.op = TOK_NOT;
        n->u.unary.operand = parse_not(p);
        return n;
    }
    return parse_compare(p);
}

static GcAstNode *parse_and(GcParser *p) {
    GcAstNode *l = parse_not(p);
    while (check(p, TOK_AND)) {
        GcToken op = p->cur;
        advance(p);
        l = make_binary(p, &op, l, parse_not(p));
    }
    return l;
}

static GcAstNode *parse_or(GcParser *p) {
    GcAstNode *l = parse_and(p);
    while (check(p, TOK_OR)) {
        GcToken op = p->cur;
        advance(p);
        l = make_binary(p, &op, l, parse_and(p));
    }
    return l;
}

static GcAstNode *parse_expr(GcParser *p) { return parse_or(p); }

/* ------------------------------------------------------------------ */
/* Statements */
/* ------------------------------------------------------------------ */
static GcAstNode *parse_statement(GcParser *p);

static bool is_stmt_list_end(const GcParser *p) {
    GcTokenKind k = p->cur.kind;
    return k == TOK_EOF || k == TOK_PERIOD || k == TOK_ELSE || k == TOK_END_IF;
}

static GcAstNode *parse_stmt_list_until_end(GcParser *p, const GcToken *at) {
    GcAstNode *list = node(p, AST_STMT_LIST, at);
    while (!is_stmt_list_end(p)) {
        int before_errors = p->errors;
        GcToken start = p->cur;
        GcAstNode *s = parse_statement(p);
        if (s) gc_ast_add_child(p->arena, list, s);
        if (p->errors != before_errors &&
            start.line == p->cur.line && start.column == p->cur.column)
            advance(p); /* guarantee progress */
    }
    return list;
}

static GcAstNode *parse_assign(GcParser *p, GcAstKind kind, GcToken kw) {
    GcAstNode *n = node(p, kind, &kw);
    if (kind == AST_STMT_MOVE) {
        n->u.assign.rhs = parse_expr(p);
        expect(p, TOK_TO, "in MOVE statement");
        n->u.assign.lhs = parse_target(p, "after MOVE ... TO");
    } else if (kind == AST_STMT_SET) {
        n->u.assign.lhs = parse_target(p, "after SET");
        expect(p, TOK_TO, "in SET statement");
        n->u.assign.rhs = parse_expr(p);
    } else {
        n->u.assign.lhs = parse_target(p, "after COMPUTE");
        accept(p, TOK_ROUNDED);
        expect(p, TOK_EQ, "in COMPUTE statement");
        n->u.assign.rhs = parse_expr(p);
    }
    return n;
}

static GcAstNode *parse_gpu_load(GcParser *p, GcToken kw) {
    GcAstNode *n = node(p, AST_STMT_GPU_LOAD, &kw);
    n->u.memop.ptr = parse_target(p, "(pointer) after GPU-LOAD");
    if (n->u.memop.ptr) gc_ast_add_child(p->arena, n, n->u.memop.ptr);
    for (;;) {
        if (accept(p, TOK_OFFSETS)) n->u.memop.offsets = parse_expr(p);
        else if (accept(p, TOK_SHAPE)) n->u.memop.shape = parse_expr(p);
        else if (accept(p, TOK_MASK)) n->u.memop.mask = parse_expr(p);
        else if (accept(p, TOK_OTHER)) n->u.memop.other = parse_expr(p);
        else break;
    }
    if (expect(p, TOK_INTO, "in GPU-LOAD statement"))
        n->u.memop.dest = parse_target(p, "after INTO");
    return n;
}

static GcAstNode *parse_gpu_store(GcParser *p, GcToken kw) {
    GcAstNode *n = node(p, AST_STMT_GPU_STORE, &kw);
    n->u.memop.ptr = parse_target(p, "(pointer) after GPU-STORE");
    if (n->u.memop.ptr) gc_ast_add_child(p->arena, n, n->u.memop.ptr);
    for (;;) {
        if (accept(p, TOK_OFFSETS)) n->u.memop.offsets = parse_expr(p);
        else if (accept(p, TOK_VALUE)) n->u.memop.dest = parse_expr(p);
        else if (accept(p, TOK_MASK)) n->u.memop.mask = parse_expr(p);
        else break;
    }
    if (!n->u.memop.dest)
        error_at(p, &kw, "%s", "GPU-STORE requires a VALUE clause");
    return n;
}

static GcAstNode *parse_gpu_dot(GcParser *p, GcToken kw) {
    GcAstNode *n = node(p, AST_STMT_GPU_DOT, &kw);
    gc_ast_add_child(p->arena, n, parse_expr(p));
    if (!accept(p, TOK_WITH)) accept(p, TOK_COMMA);
    gc_ast_add_child(p->arena, n, parse_expr(p));
    if (expect(p, TOK_INTO, "in GPU-DOT statement"))
        gc_ast_add_child(p->arena, n, parse_target(p, "after INTO"));
    return n;
}

static GcAstNode *parse_gpu_reduce(GcParser *p, GcToken kw) {
    GcAstNode *n = node(p, AST_STMT_GPU_REDUCE, &kw);
    n->u.reduce.op = kw.kind;
    gc_ast_add_child(p->arena, n, parse_expr(p));
    if (accept(p, TOK_AXIS)) {
        if (check(p, TOK_INT)) {
            n->u.reduce.axis = (int)p->cur.u.int_val;
            advance(p);
        } else {
            error_at(p, &p->cur, "%s", "expected integer after AXIS");
        }
    }
    if (accept(p, TOK_INTO))
        gc_ast_add_child(p->arena, n, parse_target(p, "after INTO"));
    return n;
}

static GcAstNode *parse_if(GcParser *p, GcToken kw) {
    GcAstNode *n = node(p, AST_STMT_IF, &kw);
    n->u.ifstmt.cond = parse_expr(p);
    accept(p, TOK_THEN);
    GcToken then_at = p->cur;
    n->u.ifstmt.then_body = parse_stmt_list_until_end(p, &then_at);
    if (check(p, TOK_ELSE)) {
        GcToken e = p->cur;
        advance(p);
        n->u.ifstmt.else_body = parse_stmt_list_until_end(p, &e);
    }
    if (!accept(p, TOK_END_IF) && !check(p, TOK_PERIOD))
        error_at(p, &p->cur, "%s", "expected END-IF or '.' to close IF");
    return n;
}

static GcAstNode *parse_statement(GcParser *p) {
    GcToken kw = p->cur;
    switch (kw.kind) {
        case TOK_COMPUTE: advance(p); return parse_assign(p, AST_STMT_COMPUTE, kw);
        case TOK_SET: advance(p); return parse_assign(p, AST_STMT_SET, kw);
        case TOK_MOVE: advance(p); return parse_assign(p, AST_STMT_MOVE, kw);
        case TOK_GPU_LOAD: advance(p); return parse_gpu_load(p, kw);
        case TOK_GPU_STORE: advance(p); return parse_gpu_store(p, kw);
        case TOK_GPU_DOT: advance(p); return parse_gpu_dot(p, kw);
        case TOK_GPU_REDUCE_SUM:
        case TOK_GPU_REDUCE_MAX:
        case TOK_GPU_REDUCE_MIN: advance(p); return parse_gpu_reduce(p, kw);
        case TOK_GPU_SYNC: advance(p); return node(p, AST_STMT_GPU_SYNC, &kw);
        case TOK_IF: advance(p); return parse_if(p, kw);
        case TOK_GOBACK: advance(p); return node(p, AST_STMT_GOBACK, &kw);
        case TOK_STOP:
            advance(p);
            expect(p, TOK_RUN, "after STOP");
            return node(p, AST_STMT_GOBACK, &kw);
        case TOK_EXIT:
            advance(p);
            expect(p, TOK_PROGRAM, "after EXIT");
            return node(p, AST_STMT_GOBACK, &kw);
        case TOK_CONTINUE:
            advance(p);
            return NULL;
        default:
            error_at(p, &kw, "%s", "expected a statement");
            sync_to_period(p);
            return NULL;
    }
}

/* ------------------------------------------------------------------ */
/* Divisions */
/* ------------------------------------------------------------------ */
static const char *parse_name(GcParser *p, const char *context) {
    if (check(p, TOK_IDENT) || check(p, TOK_STRING)) {
        const char *s = check(p, TOK_STRING) ? p->cur.u.str_val : p->cur.text;
        advance(p);
        return s;
    }
    error_at(p, &p->cur, "expected name %s", context);
    return NULL;
}

static void parse_identification(GcParser *p, GcAstNode *prog) {
    advance(p); /* IDENTIFICATION | ID */
    expect(p, TOK_DIVISION, "after IDENTIFICATION");
    expect(p, TOK_PERIOD, "after IDENTIFICATION DIVISION");
    if (accept(p, TOK_PROGRAM_ID)) {
        expect(p, TOK_PERIOD, "after PROGRAM-ID");
        prog->u.program.program_name = parse_name(p, "after PROGRAM-ID.");
        sync_to_period(p); /* optional "IS INITIAL" etc. */
    }
    /* skip AUTHOR. / DATE-WRITTEN. / ... paragraphs */
    while (!check(p, TOK_EOF) && !is_division_start(p) && !is_kernel_section(p))
        advance(p);
}

static void skip_environment(GcParser *p) {
    advance(p); /* ENVIRONMENT */
    advance(p); /* DIVISION */
    while (!check(p, TOK_EOF) && !is_division_start(p) && !is_kernel_section(p))
        advance(p);
}

static bool is_usage_kw(GcTokenKind k) {
    return k == TOK_COMP || k == TOK_COMP_1 || k == TOK_COMP_2 ||
           k == TOK_COMP_3 || k == TOK_COMP_4 || k == TOK_COMP_5 ||
           k == TOK_COMPUTATIONAL || k == TOK_BINARY;
}

static GcAstNode *parse_data_item(GcParser *p) {
    GcToken lvl = p->cur;
    advance(p);
    GcAstNode *n = node(p, AST_DATA_ITEM, &lvl);
    n->u.data_item.level = (int)lvl.u.int_val;
    if (accept(p, TOK_FILLER)) {
        n->u.data_item.name = NULL;
    } else if (check(p, TOK_IDENT) || is_soft_keyword(p->cur.kind)) {
        n->u.data_item.name = p->cur.text;
        advance(p);
    } else {
        error_at(p, &p->cur, "%s", "expected data name after level number");
        sync_to_period(p);
        return n;
    }
    while (!check(p, TOK_PERIOD) && !check(p, TOK_EOF)) {
        if (accept(p, TOK_PIC) || accept(p, TOK_PICTURE_KW)) {
            accept(p, TOK_IS);
            if (check(p, TOK_PICTURE)) {
                n->u.data_item.pic = p->cur.u.str_val;
                advance(p);
            }
        } else if (accept(p, TOK_USAGE)) {
            accept(p, TOK_IS);
            if (is_usage_kw(p->cur.kind)) {
                n->u.data_item.usage = p->cur.text;
                advance(p);
            } else {
                error_at(p, &p->cur, "%s", "expected usage after USAGE");
            }
        } else if (is_usage_kw(p->cur.kind)) {
            n->u.data_item.usage = p->cur.text;
            advance(p);
        } else if (accept(p, TOK_VALUE)) {
            accept(p, TOK_IS);
            GcAstNode *v = parse_unary(p);
            if (v && v->kind != AST_EXPR_LITERAL)
                error_at(p, &p->cur, "%s", "VALUE clause requires a literal");
            n->u.data_item.value = v;
        } else if (accept(p, TOK_OCCURS)) {
            if (check(p, TOK_INT)) {
                n->u.data_item.occurs = (int)p->cur.u.int_val;
                advance(p);
            }
            accept(p, TOK_TIMES);
        } else {
            error_at(p, &p->cur, "%s", "unexpected token in data description");
            sync_to_period(p);
            return n;
        }
    }
    expect(p, TOK_PERIOD, "to end data description entry");
    return n;
}

static bool is_type_kw(GcTokenKind k) {
    switch (k) {
        case TOK_GPU_I32: case TOK_GPU_I64: case TOK_GPU_F16:
        case TOK_GPU_F32: case TOK_GPU_F64: case TOK_GPU_MASK:
        case TOK_INT32: case TOK_INT64: case TOK_FLOAT16:
        case TOK_FLOAT32: case TOK_FLOAT64: case TOK_MASK:
            return true;
        default:
            return false;
    }
}

static GcAstNode *parse_param(GcParser *p) {
    GcToken kw = p->cur;
    advance(p); /* PARAMETER */
    GcAstNode *n = node(p, AST_PARAM, &kw);
    if (check(p, TOK_IDENT) || is_soft_keyword(p->cur.kind)) {
        n->u.param.name = p->cur.text;
        advance(p);
    } else {
        error_at(p, &p->cur, "%s", "expected parameter name");
    }
    if (!expect(p, TOK_AS, "after parameter name")) return n;
    if (accept(p, TOK_GPU_POINTER)) {
        n->u.param.is_pointer = 1;
        if (is_type_kw(p->cur.kind)) {
            n->u.param.type_name = gc_token_kind_name(p->cur.kind);
            advance(p);
        } else {
            n->u.param.type_name = "GPU-POINTER";
        }
    } else if (is_type_kw(p->cur.kind)) {
        n->u.param.type_name = gc_token_kind_name(p->cur.kind);
        advance(p);
    } else if (check(p, TOK_IDENT)) {
        n->u.param.type_name = p->cur.text;
        advance(p);
    } else {
        error_at(p, &p->cur, "%s", "expected parameter type after AS");
    }
    return n;
}

static GcAstNode *parse_kernel_def(GcParser *p) {
    GcToken kw = p->cur;
    advance(p); /* KERNEL */
    GcAstNode *k = node(p, AST_KERNEL_DEF, &kw);
    k->u.kernel.name = parse_name(p, "after KERNEL");

    GcAstNode *tmp = node(p, AST_STMT_LIST, &kw); /* param collector */
    while (check(p, TOK_PARAMETER)) {
        gc_ast_add_child(p->arena, tmp, parse_param(p));
        accept(p, TOK_COMMA);
    }
    k->u.kernel.params = tmp->children;
    k->u.kernel.n_params = tmp->n_children;
    if (!expect(p, TOK_PERIOD, "to end KERNEL declaration"))
        sync_to_period(p);
    return k;
}

static GcAstNode *parse_kernel_section(GcParser *p, GcAstNode *existing) {
    GcToken kw = p->cur;
    advance(p); /* KERNEL */
    advance(p); /* SECTION */
    expect(p, TOK_PERIOD, "after KERNEL SECTION");
    GcAstNode *sec = existing ? existing : node(p, AST_KERNEL_SECTION, &kw);
    while (check(p, TOK_KERNEL) && !is_kernel_section(p))
        gc_ast_add_child(p->arena, sec, parse_kernel_def(p));
    return sec;
}

static GcAstNode *parse_procedure(GcParser *p) {
    GcToken kw = p->cur;
    advance(p); /* PROCEDURE */
    advance(p); /* DIVISION */
    GcAstNode *proc = node(p, AST_PROC_DIV, &kw);
    if (accept(p, TOK_USING)) {
        while (check(p, TOK_IDENT) || check(p, TOK_COMMA)) advance(p);
    }
    expect(p, TOK_PERIOD, "after PROCEDURE DIVISION");

    while (!check(p, TOK_EOF)) {
        if (accept(p, TOK_PERIOD)) continue;
        /* END PROGRAM name. */
        if (check(p, TOK_END) && p->next.kind == TOK_PROGRAM) {
            sync_to_period(p);
            break;
        }
        /* paragraph / section labels: "NAME." or "NAME SECTION." */
        if (check(p, TOK_IDENT) &&
            (p->next.kind == TOK_PERIOD || p->next.kind == TOK_SECTION)) {
            sync_to_period(p);
            continue;
        }
        int before_errors = p->errors;
        GcToken start = p->cur;
        GcAstNode *s = parse_statement(p);
        if (s) gc_ast_add_child(p->arena, proc, s);
        if (p->errors != before_errors &&
            start.line == p->cur.line && start.column == p->cur.column)
            advance(p);
        if (p->errors > 50) break;
    }
    return proc;
}

/* ------------------------------------------------------------------ */
/* Entry points */
/* ------------------------------------------------------------------ */
void gc_parser_init(GcParser *p, GcLexer *lex, GcArena *arena,
                    GcDiagList *diags) {
    memset(p, 0, sizeof(*p));
    p->lex = lex;
    p->arena = arena;
    p->diags = diags;
    p->next = gc_lexer_next(lex);
    advance(p); /* cur = first token, next = second */
}

GcAstNode *gc_parser_parse(GcParser *p) {
    GcAstNode *prog = node(p, AST_PROGRAM, &p->cur);

    if ((check(p, TOK_IDENTIFICATION) || check(p, TOK_ID)) &&
        p->next.kind == TOK_DIVISION)
        parse_identification(p, prog);
    if (check(p, TOK_ENVIRONMENT) && p->next.kind == TOK_DIVISION)
        skip_environment(p);

    while (!check(p, TOK_EOF) &&
           !(check(p, TOK_PROCEDURE) && p->next.kind == TOK_DIVISION)) {
        if (check(p, TOK_DATA) && p->next.kind == TOK_DIVISION) {
            advance(p);
            advance(p);
            expect(p, TOK_PERIOD, "after DATA DIVISION");
            if (!prog->u.program.data_div)
                prog->u.program.data_div = node(p, AST_DATA_DIV, &p->cur);
        } else if ((check(p, TOK_WORKING_STORAGE) || check(p, TOK_LOCAL_STORAGE) ||
                    check(p, TOK_LINKAGE)) && p->next.kind == TOK_SECTION) {
            advance(p);
            advance(p);
            expect(p, TOK_PERIOD, "after SECTION");
            if (!prog->u.program.data_div)
                prog->u.program.data_div = node(p, AST_DATA_DIV, &p->cur);
        } else if (check(p, TOK_LEVEL)) {
            if (!prog->u.program.data_div)
                prog->u.program.data_div = node(p, AST_DATA_DIV, &p->cur);
            gc_ast_add_child(p->arena, prog->u.program.data_div,
                             parse_data_item(p));
        } else if (is_kernel_section(p)) {
            prog->u.program.kernel_sec =
                parse_kernel_section(p, prog->u.program.kernel_sec);
        } else {
            error_at(p, &p->cur, "%s",
                     "expected data description, KERNEL SECTION or "
                     "PROCEDURE DIVISION");
            sync_to_period(p);
        }
        if (p->errors > 50) break;
    }

    if (check(p, TOK_PROCEDURE) && p->next.kind == TOK_DIVISION)
        prog->u.program.proc_div = parse_procedure(p);

    if (!check(p, TOK_EOF) && p->errors == 0)
        error_at(p, &p->cur, "%s", "unexpected tokens after end of program");
    return prog;
}

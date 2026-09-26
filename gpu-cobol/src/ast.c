#include "ast.h"

GcAstNode *gc_ast_new(GcArena *a, GcAstKind kind, int line, int column) {
    GcAstNode *n = (GcAstNode *)gc_arena_alloc(a, sizeof(GcAstNode));
    n->kind = kind;
    n->line = line;
    n->column = column;
    return n;
}

void gc_ast_add_child(GcArena *a, GcAstNode *parent, GcAstNode *child) {
    if (!parent || !child) return;
    if (parent->n_children >= parent->cap_children) {
        int ncap = parent->cap_children ? parent->cap_children * 2 : 4;
        GcAstNode **nc = (GcAstNode **)gc_arena_alloc(
            a, sizeof(GcAstNode *) * (size_t)ncap);
        if (parent->n_children)
            memcpy(nc, parent->children,
                   sizeof(GcAstNode *) * (size_t)parent->n_children);
        parent->children = nc;
        parent->cap_children = ncap;
    }
    parent->children[parent->n_children++] = child;
}

const char *gc_ast_kind_name(GcAstKind kind) {
    switch (kind) {
        case AST_PROGRAM: return "Program";
        case AST_DATA_DIV: return "DataDivision";
        case AST_DATA_ITEM: return "DataItem";
        case AST_KERNEL_SECTION: return "KernelSection";
        case AST_KERNEL_DEF: return "Kernel";
        case AST_PARAM: return "Param";
        case AST_PROC_DIV: return "ProcedureDivision";
        case AST_STMT_LIST: return "StmtList";
        case AST_STMT_COMPUTE: return "Compute";
        case AST_STMT_SET: return "Set";
        case AST_STMT_MOVE: return "Move";
        case AST_STMT_IF: return "If";
        case AST_STMT_GPU_LOAD: return "GpuLoad";
        case AST_STMT_GPU_STORE: return "GpuStore";
        case AST_STMT_GPU_DOT: return "GpuDot";
        case AST_STMT_GPU_REDUCE: return "GpuReduce";
        case AST_STMT_GPU_SYNC: return "GpuSync";
        case AST_STMT_GOBACK: return "Goback";
        case AST_EXPR_LITERAL: return "Literal";
        case AST_EXPR_IDENT: return "Ident";
        case AST_EXPR_PROGRAM_ID: return "ProgramId";
        case AST_EXPR_LANE_ID: return "LaneId";
        case AST_EXPR_BINARY: return "Binary";
        case AST_EXPR_UNARY: return "Unary";
        default: return "?";
    }
}

static void pad(int indent, FILE *out) {
    for (int i = 0; i < indent; i++) fputs("  ", out);
}

static void print_labeled(const char *label, const GcAstNode *n, int indent,
                          FILE *out) {
    if (!n) return;
    pad(indent, out);
    fprintf(out, "%s:\n", label);
    gc_ast_print(n, indent + 1, out);
}

static void print_children(const GcAstNode *n, int indent, FILE *out) {
    for (int i = 0; i < n->n_children; i++)
        gc_ast_print(n->children[i], indent, out);
}

void gc_ast_print(const GcAstNode *n, int indent, FILE *out) {
    if (!out) out = stdout;
    if (!n) {
        pad(indent, out);
        fputs("<null>\n", out);
        return;
    }
    pad(indent, out);
    fputs(gc_ast_kind_name(n->kind), out);

    switch (n->kind) {
        case AST_PROGRAM:
            fprintf(out, " %s\n",
                    n->u.program.program_name ? n->u.program.program_name : "<unnamed>");
            if (n->u.program.data_div) gc_ast_print(n->u.program.data_div, indent + 1, out);
            if (n->u.program.kernel_sec) gc_ast_print(n->u.program.kernel_sec, indent + 1, out);
            if (n->u.program.proc_div) gc_ast_print(n->u.program.proc_div, indent + 1, out);
            return;

        case AST_DATA_ITEM:
            fprintf(out, " %02d %s", n->u.data_item.level,
                    n->u.data_item.name ? n->u.data_item.name : "FILLER");
            if (n->u.data_item.pic) fprintf(out, " PIC %s", n->u.data_item.pic);
            if (n->u.data_item.usage) fprintf(out, " USAGE %s", n->u.data_item.usage);
            if (n->u.data_item.occurs) fprintf(out, " OCCURS %d", n->u.data_item.occurs);
            fputc('\n', out);
            print_labeled("value", n->u.data_item.value, indent + 1, out);
            return;

        case AST_KERNEL_DEF:
            fprintf(out, " %s (%d params)\n",
                    n->u.kernel.name ? n->u.kernel.name : "<unnamed>",
                    n->u.kernel.n_params);
            for (int i = 0; i < n->u.kernel.n_params; i++)
                gc_ast_print(n->u.kernel.params[i], indent + 1, out);
            return;

        case AST_PARAM:
            fprintf(out, " %s AS %s%s\n", n->u.param.name ? n->u.param.name : "?",
                    n->u.param.is_pointer ? "GPU-POINTER " : "",
                    n->u.param.type_name ? n->u.param.type_name : "?");
            return;

        case AST_STMT_COMPUTE:
        case AST_STMT_SET:
        case AST_STMT_MOVE:
            fputc('\n', out);
            print_labeled("target", n->u.assign.lhs, indent + 1, out);
            print_labeled("value", n->u.assign.rhs, indent + 1, out);
            return;

        case AST_STMT_IF:
            fputc('\n', out);
            print_labeled("cond", n->u.ifstmt.cond, indent + 1, out);
            print_labeled("then", n->u.ifstmt.then_body, indent + 1, out);
            print_labeled("else", n->u.ifstmt.else_body, indent + 1, out);
            return;

        case AST_STMT_GPU_LOAD:
            fputc('\n', out);
            print_labeled("ptr", n->u.memop.ptr, indent + 1, out);
            print_labeled("offsets", n->u.memop.offsets, indent + 1, out);
            print_labeled("shape", n->u.memop.shape, indent + 1, out);
            print_labeled("mask", n->u.memop.mask, indent + 1, out);
            print_labeled("other", n->u.memop.other, indent + 1, out);
            print_labeled("into", n->u.memop.dest, indent + 1, out);
            return;

        case AST_STMT_GPU_STORE:
            fputc('\n', out);
            print_labeled("ptr", n->u.memop.ptr, indent + 1, out);
            print_labeled("offsets", n->u.memop.offsets, indent + 1, out);
            print_labeled("value", n->u.memop.dest, indent + 1, out);
            print_labeled("mask", n->u.memop.mask, indent + 1, out);
            return;

        case AST_STMT_GPU_REDUCE:
            fprintf(out, " %s axis=%d\n", gc_token_kind_name(n->u.reduce.op),
                    n->u.reduce.axis);
            print_children(n, indent + 1, out);
            return;

        case AST_EXPR_LITERAL:
            switch (n->u.literal.lit_kind) {
                case TOK_FLOAT: fprintf(out, " %g\n", n->u.literal.float_val); break;
                case TOK_STRING:
                    fprintf(out, " \"%s\"\n",
                            n->u.literal.str_val ? n->u.literal.str_val : "");
                    break;
                default:
                    fprintf(out, " %lld\n", (long long)n->u.literal.int_val);
                    break;
            }
            return;

        case AST_EXPR_IDENT:
            fprintf(out, " %s\n", n->u.ident.name ? n->u.ident.name : "?");
            return;

        case AST_EXPR_PROGRAM_ID:
            fprintf(out, " %c\n", "XYZ"[n->u.program_id.axis % 3]);
            return;

        case AST_EXPR_BINARY:
            fprintf(out, " %s\n", gc_token_kind_name(n->u.binary.op));
            gc_ast_print(n->u.binary.left, indent + 1, out);
            gc_ast_print(n->u.binary.right, indent + 1, out);
            return;

        case AST_EXPR_UNARY:
            fprintf(out, " %s\n", gc_token_kind_name(n->u.unary.op));
            gc_ast_print(n->u.unary.operand, indent + 1, out);
            return;

        default:
            fputc('\n', out);
            print_children(n, indent + 1, out);
            return;
    }
}

#include "common.h"
#include "lexer.h"
#include "parser.h"
#include "ast.h"
#include "sema.h"
#include "ir.h"
#include "ptx.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *read_file(const char *path, size_t *out_len) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (sz < 0) { fclose(f); return NULL; }
    char *buf = (char *)malloc((size_t)sz + 1);
    if (!buf) { fclose(f); return NULL; }
    size_t n = fread(buf, 1, (size_t)sz, f);
    fclose(f);
    buf[n] = '\0';
    if (out_len) *out_len = n;
    return buf;
}

static void usage(const char *argv0) {
    fprintf(stderr,
        "GPU-COBOL compiler %s\n"
        "Usage: %s [options] <file.cbl|file.cob>\n"
        "Options:\n"
        " --emit-ast Print AST and exit\n"
        " --emit-ir Print GPU-COBOL IR and exit\n"
        " --emit-ptx Emit PTX to stdout\n"
        " --arch=sm_XX Target architecture (default sm_86)\n"
        " --check Parse + semantic check only\n"
        " -h, --help Show this help\n",
        GC_VERSION_STRING, argv0);
}

int main(int argc, char **argv) {
    const char *input = NULL;
    const char *arch = "sm_86";
    int emit_ast = 0, emit_ir = 0, emit_ptx = 0, check_only = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            usage(argv[0]);
            return 0;
        } else if (strcmp(argv[i], "--emit-ast") == 0) {
            emit_ast = 1;
        } else if (strcmp(argv[i], "--emit-ir") == 0) {
            emit_ir = 1;
        } else if (strcmp(argv[i], "--emit-ptx") == 0) {
            emit_ptx = 1;
        } else if (strcmp(argv[i], "--check") == 0) {
            check_only = 1;
        } else if (strncmp(argv[i], "--arch=", 7) == 0) {
            arch = argv[i] + 7;
        } else if (argv[i][0] != '-') {
            input = argv[i];
        } else {
            fprintf(stderr, "unknown option: %s\n", argv[i]);
            usage(argv[0]);
            return 1;
        }
    }

    if (!input) {
        usage(argv[0]);
        return 1;
    }

    size_t src_len = 0;
    char *src = read_file(input, &src_len);
    if (!src) {
        fprintf(stderr, "cannot read '%s'\n", input);
        return 1;
    }

    GcArena *arena = gc_arena_create(1 << 20);
    GcDiagList diags;
    gc_diag_init(&diags);

    GcLexer lexer;
    gc_lexer_init(&lexer, src, src_len, input, arena, &diags);

    GcParser parser;
    gc_parser_init(&parser, &lexer, arena, &diags);
    GcAstNode *ast = gc_parser_parse(&parser);

    if (gc_diag_has_errors(&diags)) {
        gc_diag_print_all(&diags, stderr);
        free(src);
        gc_arena_destroy(arena);
        return 1;
    }

    if (emit_ast) {
        gc_ast_print(ast, 0, stdout);
        free(src);
        gc_arena_destroy(arena);
        return 0;
    }

    GcSema sema;
    gc_sema_init(&sema, arena, &diags);
    GcIrModule *mod = gc_sema_analyze(&sema, ast);

    if (gc_diag_has_errors(&diags)) {
        gc_diag_print_all(&diags, stderr);
        free(src);
        gc_arena_destroy(arena);
        return 1;
    }

    if (emit_ir || check_only) {
        if (emit_ir && mod)
            gc_ir_dump(mod, stdout);
        free(src);
        gc_arena_destroy(arena);
        return 0;
    }

    if (emit_ptx || !check_only) {
        GcPtxEmitter ptx;
        gc_ptx_init(&ptx, stdout, arch, &diags);
        if (mod)
            gc_ptx_emit_module(&ptx, mod);
    }

    free(src);
    gc_arena_destroy(arena);
    return gc_diag_has_errors(&diags) ? 1 : 0;
}

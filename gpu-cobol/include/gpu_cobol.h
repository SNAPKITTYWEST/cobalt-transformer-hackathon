/*
 * GPU-COBOL — Hand-rolled COBOL-oriented GPU DSL Compiler
 * Public header: core types, error handling, pipeline entry points.
 *
 * Semantic model reverse-engineered from Triton-style GPU programming
 * abstractions, expressed in a COBOL-compatible surface syntax.
 * No Python runtime. No Triton dependency.
 */
#ifndef GPU_COBOL_H
#define GPU_COBOL_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ------------------------------------------------------------------ */
/* Version & target architecture */
/* ------------------------------------------------------------------ */
#define GPUCOBOL_VERSION_MAJOR 0
#define GPUCOBOL_VERSION_MINOR 1
#define GPUCOBOL_VERSION_PATCH 0
#define GPUCOBOL_VERSION_STRING "0.1.0"

typedef enum {
    GC_ARCH_SM_70 = 70,
    GC_ARCH_SM_75 = 75,
    GC_ARCH_SM_80 = 80,
    GC_ARCH_SM_86 = 86, /* default Ampere */
    GC_ARCH_SM_89 = 89,
    GC_ARCH_SM_90 = 90,
    GC_ARCH_CPU_INTERPRETER = 0
} GcTargetArch;

/* ------------------------------------------------------------------ */
/* Diagnostic / error system */
/* ------------------------------------------------------------------ */
typedef enum {
    GC_DIAG_NOTE,
    GC_DIAG_WARNING,
    GC_DIAG_ERROR,
    GC_DIAG_FATAL
} GcDiagLevel;

typedef struct {
    GcDiagLevel level;
    const char *filename;
    int line;
    int column;
    char message[512];
    char context[256];
} GcDiagnostic;

#define GC_MAX_DIAGNOSTICS 256

typedef struct {
    GcDiagnostic items[GC_MAX_DIAGNOSTICS];
    int count;
    int error_count;
    int warning_count;
} GcDiagList;

void gc_diag_init(GcDiagList *list);
void gc_diag_emit(GcDiagList *list, GcDiagLevel level,
                  const char *file, int line, int col,
                  const char *fmt, ...);
void gc_diag_print_all(const GcDiagList *list, FILE *out);
bool gc_diag_has_errors(const GcDiagList *list);

/* ------------------------------------------------------------------ */
/* Source buffer */
/* ------------------------------------------------------------------ */
typedef struct {
    char *data;
    size_t length;
    size_t capacity;
    char *filename;
} GcSource;

int gc_source_load(GcSource *src, const char *path);
void gc_source_free(GcSource *src);
const char *gc_source_line(const GcSource *src, int line, int *out_len);

/* ------------------------------------------------------------------ */
/* Forward declarations of major pipeline objects */
/* ------------------------------------------------------------------ */
typedef struct GcToken GcToken;
typedef struct GcLexer GcLexer;
typedef struct GcAstNode GcAstNode;
typedef struct GcParser GcParser;
typedef struct GcType GcType;
typedef struct GcSymbol GcSymbol;
typedef struct GcScope GcScope;
typedef struct GcSemantic GcSemantic;
typedef struct GcIrModule GcIrModule;
typedef struct GcIrValue GcIrValue;
typedef struct GcIrInst GcIrInst;
typedef struct GcIrBlock GcIrBlock;
typedef struct GcIrFunc GcIrFunc;
typedef struct GcPtxModule GcPtxModule;
typedef struct GcCompiler GcCompiler;

/* ------------------------------------------------------------------ */
/* Compiler driver options */
/* ------------------------------------------------------------------ */
typedef struct {
    GcTargetArch arch;
    bool emit_ir;
    bool emit_ptx;
    bool emit_ast;
    bool run_cpu;
    bool verbose;
    bool check_only;
    const char *output_path;
    const char *input_path;
} GcOptions;

void gc_options_init(GcOptions *opt);

/* ------------------------------------------------------------------ */
/* Top-level compile entry */
/* ------------------------------------------------------------------ */
typedef enum {
    GC_OK = 0,
    GC_ERR_IO,
    GC_ERR_LEX,
    GC_ERR_PARSE,
    GC_ERR_SEMANTIC,
    GC_ERR_IR,
    GC_ERR_PTX,
    GC_ERR_INTERNAL
} GcStatus;

GcStatus gc_compile(const GcOptions *opt, GcDiagList *diags);

#ifdef __cplusplus
}
#endif

#endif /* GPU_COBOL_H */

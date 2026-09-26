#ifndef GPU_COBOL_SEMA_H
#define GPU_COBOL_SEMA_H

#include "ast.h"
#include "ir.h"
#include "common.h"

typedef struct {
    char *name;
    GcType type;
    GcIrValue *ir_value;
    int is_param;
    int is_gpu;
} GcSymbol;

#define GC_SYMTAB_SIZE 256

typedef struct {
    GcSymbol table[GC_SYMTAB_SIZE];
    int count;
} GcSymTab;

typedef struct {
    GcArena *arena;
    GcDiagList *diags;
    GcSymTab globals;
    GcSymTab locals;
    GcIrModule *module;
    GcIrKernel *current_kernel;
    GcIrBlock *current_block;
} GcSema;

void gc_sema_init(GcSema *s, GcArena *a, GcDiagList *d);
GcIrModule *gc_sema_analyze(GcSema *s, GcAstNode *ast);

#endif

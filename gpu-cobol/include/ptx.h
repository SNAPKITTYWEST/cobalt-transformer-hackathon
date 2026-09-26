#ifndef GPU_COBOL_PTX_H
#define GPU_COBOL_PTX_H

#include "ir.h"
#include "common.h"

typedef struct {
    char *arch; /* e.g. "sm_86" */
    int version_major;
    int version_minor;
    FILE *out;
    int indent;
    GcDiagList *diags;
} GcPtxEmitter;

void gc_ptx_init(GcPtxEmitter *e, FILE *out, const char *arch, GcDiagList *d);
int gc_ptx_emit_module(GcPtxEmitter *e, const GcIrModule *m);

#endif

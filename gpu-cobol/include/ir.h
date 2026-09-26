/*
 * ir.h — GPU-COBOL SSA-style intermediate representation.
 *
 *   GcIrModule  -> list of GcIrKernel (next)
 *   GcIrKernel  -> params[], list of GcIrBlock (blocks / entry)
 *   GcIrBlock   -> list of GcIrInst (first .. last)
 *   GcIrInst    -> op, type, operands[], optional result GcIrValue
 */
#ifndef GPU_COBOL_IR_H
#define GPU_COBOL_IR_H

#include "common.h"
#include "token.h"

/* ------------------------------------------------------------------ */
/* Types */
/* ------------------------------------------------------------------ */
typedef enum {
    GC_TYPE_VOID,
    GC_TYPE_INT1,
    GC_TYPE_INT32,
    GC_TYPE_INT64,
    GC_TYPE_FLOAT16,
    GC_TYPE_FLOAT32,
    GC_TYPE_FLOAT64,
    GC_TYPE_POINTER,
    GC_TYPE_MASK
} GcTypeKind;

typedef struct GcType {
    GcTypeKind kind;
    GcTypeKind elem_kind; /* pointee kind for GC_TYPE_POINTER */
} GcType;

GcType gc_type_void(void);
GcType gc_type_int32(void);
GcType gc_type_int64(void);
GcType gc_type_float32(void);
GcType gc_type_float64(void);
GcType gc_type_pointer(GcType elem);
GcType gc_type_mask(void);
const char *gc_type_kind_name(GcTypeKind k);
/* Writes e.g. "i32", "f32", "ptr<f32>", "mask" into buf. */
const char *gc_type_str(GcType t, char *buf, size_t n);

/* ------------------------------------------------------------------ */
/* IR objects */
/* ------------------------------------------------------------------ */
typedef enum {
    IR_CONST,
    IR_PROGRAM_ID,
    IR_LANE_ID,
    IR_LOAD,
    IR_STORE,
    IR_BINOP,
    IR_UNOP,
    IR_DOT,
    IR_REDUCE,
    IR_BARRIER,
    IR_RETURN
} GcIrOp;

#define GC_MAX_PARAMS 32
#define GC_IR_MAX_OPERANDS 8

typedef struct GcIrValue GcIrValue;
typedef struct GcIrInst GcIrInst;
typedef struct GcIrBlock GcIrBlock;
typedef struct GcIrKernel GcIrKernel;
typedef struct GcIrModule GcIrModule;

struct GcIrValue {
    int id;
    GcType type;
    const char *name; /* source-level name, may be NULL */
    GcIrInst *def;    /* defining instruction, NULL for params */
};

struct GcIrInst {
    GcIrOp op;
    GcType type;
    GcIrValue *result; /* NULL for STORE / BARRIER / RETURN */
    GcIrValue *operands[GC_IR_MAX_OPERANDS];
    int n_operands;
    int64_t imm_i;       /* IR_CONST (integer) */
    double imm_f;        /* IR_CONST (float) */
    int axis;            /* IR_PROGRAM_ID / IR_REDUCE */
    GcTokenKind bin_op;  /* IR_BINOP / IR_UNOP operator token */
    GcIrBlock *block;
    GcIrInst *next;
};

struct GcIrBlock {
    int id;
    GcIrInst *first;
    GcIrInst *last;
    GcIrBlock *next;
};

struct GcIrKernel {
    const char *name;
    GcIrModule *module;
    GcIrValue *params[GC_MAX_PARAMS];
    int n_params;
    GcIrBlock *entry;
    GcIrBlock *blocks;
    int next_value_id;
    int next_block_id;
    GcIrKernel *next;
};

struct GcIrModule {
    const char *name;
    GcArena *arena;
    GcIrKernel *kernels;
    int n_kernels;
};

GcIrModule *gc_ir_module_create(GcArena *arena, const char *name);
/* Creates a kernel with an empty entry block and appends it to the module.
 * The name is sanitised to a C/PTX identifier ('-' -> '_'). */
GcIrKernel *gc_ir_kernel_create(GcIrModule *m, const char *name);
GcIrBlock *gc_ir_block_create(GcIrKernel *k);
GcIrValue *gc_ir_value_create(GcIrKernel *k, GcType type, const char *name);
/* Appends a new instruction to block b. A result value of `type` is created
 * for every op except IR_STORE, IR_BARRIER and IR_RETURN. */
GcIrInst *gc_ir_inst_create(GcIrKernel *k, GcIrBlock *b, GcIrOp op,
                            GcType type);
void gc_ir_inst_add_operand(GcIrInst *inst, GcIrValue *v);

const char *gc_ir_op_name(GcIrOp op);
/* Structural checks; reports problems to diags. Returns number of errors. */
int gc_ir_validate(const GcIrModule *m, GcDiagList *diags);
void gc_ir_dump(const GcIrModule *m, FILE *out);

#endif /* GPU_COBOL_IR_H */

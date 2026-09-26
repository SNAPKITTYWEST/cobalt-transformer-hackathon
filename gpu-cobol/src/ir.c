#include "ir.h"

#include <ctype.h>

/* ------------------------------------------------------------------ */
/* Types */
/* ------------------------------------------------------------------ */
static GcType mk(GcTypeKind k) {
    GcType t;
    t.kind = k;
    t.elem_kind = GC_TYPE_VOID;
    return t;
}

GcType gc_type_void(void) { return mk(GC_TYPE_VOID); }
GcType gc_type_int32(void) { return mk(GC_TYPE_INT32); }
GcType gc_type_int64(void) { return mk(GC_TYPE_INT64); }
GcType gc_type_float32(void) { return mk(GC_TYPE_FLOAT32); }
GcType gc_type_float64(void) { return mk(GC_TYPE_FLOAT64); }
GcType gc_type_mask(void) { return mk(GC_TYPE_MASK); }

GcType gc_type_pointer(GcType elem) {
    GcType t = mk(GC_TYPE_POINTER);
    t.elem_kind = elem.kind;
    return t;
}

const char *gc_type_kind_name(GcTypeKind k) {
    switch (k) {
        case GC_TYPE_VOID: return "void";
        case GC_TYPE_INT1: return "i1";
        case GC_TYPE_INT32: return "i32";
        case GC_TYPE_INT64: return "i64";
        case GC_TYPE_FLOAT16: return "f16";
        case GC_TYPE_FLOAT32: return "f32";
        case GC_TYPE_FLOAT64: return "f64";
        case GC_TYPE_POINTER: return "ptr";
        case GC_TYPE_MASK: return "mask";
    }
    return "?";
}

const char *gc_type_str(GcType t, char *buf, size_t n) {
    if (t.kind == GC_TYPE_POINTER)
        snprintf(buf, n, "ptr<%s>", gc_type_kind_name(t.elem_kind));
    else
        snprintf(buf, n, "%s", gc_type_kind_name(t.kind));
    return buf;
}

/* ------------------------------------------------------------------ */
/* Construction */
/* ------------------------------------------------------------------ */
GcIrModule *gc_ir_module_create(GcArena *arena, const char *name) {
    GcIrModule *m = (GcIrModule *)gc_arena_alloc(arena, sizeof(GcIrModule));
    m->arena = arena;
    m->name = gc_arena_strdup(arena, name ? name : "module");
    return m;
}

GcIrBlock *gc_ir_block_create(GcIrKernel *k) {
    GcIrBlock *b = (GcIrBlock *)gc_arena_alloc(k->module->arena, sizeof(GcIrBlock));
    b->id = k->next_block_id++;
    if (!k->blocks) {
        k->blocks = b;
    } else {
        GcIrBlock *t = k->blocks;
        while (t->next) t = t->next;
        t->next = b;
    }
    return b;
}

GcIrKernel *gc_ir_kernel_create(GcIrModule *m, const char *name) {
    GcIrKernel *k = (GcIrKernel *)gc_arena_alloc(m->arena, sizeof(GcIrKernel));
    k->module = m;
    char *nm = gc_arena_strdup(m->arena, name ? name : "kernel");
    for (char *c = nm; *c; c++)
        if (!isalnum((unsigned char)*c) && *c != '_') *c = '_';
    k->name = nm;
    k->entry = gc_ir_block_create(k);
    if (!m->kernels) {
        m->kernels = k;
    } else {
        GcIrKernel *t = m->kernels;
        while (t->next) t = t->next;
        t->next = k;
    }
    m->n_kernels++;
    return k;
}

GcIrValue *gc_ir_value_create(GcIrKernel *k, GcType type, const char *name) {
    GcIrValue *v = (GcIrValue *)gc_arena_alloc(k->module->arena, sizeof(GcIrValue));
    v->id = k->next_value_id++;
    v->type = type;
    v->name = name ? gc_arena_strdup(k->module->arena, name) : NULL;
    return v;
}

static bool op_has_result(GcIrOp op) {
    return !(op == IR_STORE || op == IR_BARRIER || op == IR_RETURN);
}

GcIrInst *gc_ir_inst_create(GcIrKernel *k, GcIrBlock *b, GcIrOp op,
                            GcType type) {
    GcIrInst *i = (GcIrInst *)gc_arena_alloc(k->module->arena, sizeof(GcIrInst));
    i->op = op;
    i->type = type;
    i->block = b;
    if (op_has_result(op)) {
        i->result = gc_ir_value_create(k, type, NULL);
        i->result->def = i;
    }
    if (b) {
        if (!b->first) b->first = i;
        else b->last->next = i;
        b->last = i;
    }
    return i;
}

void gc_ir_inst_add_operand(GcIrInst *inst, GcIrValue *v) {
    if (!inst || inst->n_operands >= GC_IR_MAX_OPERANDS) return;
    inst->operands[inst->n_operands++] = v;
}

const char *gc_ir_op_name(GcIrOp op) {
    switch (op) {
        case IR_CONST: return "const";
        case IR_PROGRAM_ID: return "program_id";
        case IR_LANE_ID: return "lane_id";
        case IR_LOAD: return "load";
        case IR_STORE: return "store";
        case IR_BINOP: return "binop";
        case IR_UNOP: return "unop";
        case IR_DOT: return "dot";
        case IR_REDUCE: return "reduce";
        case IR_BARRIER: return "barrier";
        case IR_RETURN: return "return";
    }
    return "?";
}

/* ------------------------------------------------------------------ */
/* Validation */
/* ------------------------------------------------------------------ */
int gc_ir_validate(const GcIrModule *m, GcDiagList *diags) {
    int errs = 0;
    if (!m) return 0;
#define IR_ERR(...)                                                      \
    do {                                                                 \
        gc_diag_emit(diags, GC_DIAG_ERROR, "ir", 0, 0, __VA_ARGS__);     \
        errs++;                                                          \
    } while (0)

    for (const GcIrKernel *k = m->kernels; k; k = k->next) {
        if (!k->entry) IR_ERR("kernel '%s' has no entry block", k->name);
        const GcIrInst *last = NULL;
        for (const GcIrBlock *b = k->blocks; b; b = b->next) {
            for (const GcIrInst *i = b->first; i; i = i->next) {
                last = i;
                for (int o = 0; o < i->n_operands; o++)
                    if (!i->operands[o])
                        IR_ERR("kernel '%s': %s has null operand %d",
                               k->name, gc_ir_op_name(i->op), o);
                if (op_has_result(i->op) && !i->result)
                    IR_ERR("kernel '%s': %s has no result",
                           k->name, gc_ir_op_name(i->op));
                switch (i->op) {
                    case IR_LOAD:
                    case IR_STORE:
                        if (i->n_operands < (i->op == IR_LOAD ? 1 : 2))
                            IR_ERR("kernel '%s': %s has too few operands",
                                   k->name, gc_ir_op_name(i->op));
                        else if (i->operands[0] &&
                                 i->operands[0]->type.kind != GC_TYPE_POINTER)
                            IR_ERR("kernel '%s': %s base operand %%%d is not "
                                   "a pointer", k->name, gc_ir_op_name(i->op),
                                   i->operands[0]->id);
                        break;
                    case IR_BINOP:
                        if (i->n_operands != 2)
                            IR_ERR("kernel '%s': binop needs 2 operands, has %d",
                                   k->name, i->n_operands);
                        break;
                    case IR_UNOP:
                        if (i->n_operands != 1)
                            IR_ERR("kernel '%s': unop needs 1 operand, has %d",
                                   k->name, i->n_operands);
                        break;
                    default:
                        break;
                }
            }
        }
        if (!last || last->op != IR_RETURN)
            gc_diag_emit(diags, GC_DIAG_WARNING, "ir", 0, 0,
                         "kernel '%s' does not end with a return", k->name);
    }
#undef IR_ERR
    return errs;
}

/* ------------------------------------------------------------------ */
/* Dump */
/* ------------------------------------------------------------------ */
static void dump_value(const GcIrValue *v, FILE *out) {
    if (!v) {
        fputs("<null>", out);
        return;
    }
    fprintf(out, "%%%d", v->id);
    if (v->name) fprintf(out, "(%s)", v->name);
}

void gc_ir_dump(const GcIrModule *m, FILE *out) {
    if (!m) return;
    char tb[32];
    fprintf(out, "; GPU-COBOL IR, version %s\n", GC_VERSION_STRING);
    fprintf(out, "module %s\n\n", m->name);
    for (const GcIrKernel *k = m->kernels; k; k = k->next) {
        fprintf(out, "kernel %s(", k->name);
        for (int p = 0; p < k->n_params; p++) {
            const GcIrValue *v = k->params[p];
            fprintf(out, "%s%%%d: %s %s", p ? ", " : "", v->id,
                    gc_type_str(v->type, tb, sizeof tb), v->name ? v->name : "");
        }
        fputs(") {\n", out);
        for (const GcIrBlock *b = k->blocks; b; b = b->next) {
            fprintf(out, "bb%d:\n", b->id);
            for (const GcIrInst *i = b->first; i; i = i->next) {
                fputs("    ", out);
                if (i->result) {
                    dump_value(i->result, out);
                    fputs(" = ", out);
                }
                fputs(gc_ir_op_name(i->op), out);
                switch (i->op) {
                    case IR_CONST:
                        if (i->type.kind == GC_TYPE_FLOAT32 ||
                            i->type.kind == GC_TYPE_FLOAT64)
                            fprintf(out, " %g", i->imm_f);
                        else
                            fprintf(out, " %lld", (long long)i->imm_i);
                        break;
                    case IR_PROGRAM_ID:
                        fprintf(out, ".%c", "xyz"[i->axis % 3]);
                        break;
                    case IR_BINOP:
                    case IR_UNOP:
                        fprintf(out, " '%s'", gc_token_kind_name(i->bin_op));
                        break;
                    default:
                        break;
                }
                for (int o = 0; o < i->n_operands; o++) {
                    fputs(o ? ", " : " ", out);
                    dump_value(i->operands[o], out);
                }
                if (i->result)
                    fprintf(out, " : %s", gc_type_str(i->type, tb, sizeof tb));
                fputc('\n', out);
            }
        }
        fputs("}\n\n", out);
    }
}

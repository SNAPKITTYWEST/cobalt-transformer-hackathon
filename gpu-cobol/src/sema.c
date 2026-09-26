#include "sema.h"

void gc_sema_init(GcSema *s, GcArena *a, GcDiagList *d) {
    memset(s, 0, sizeof(*s));
    s->arena = a;
    s->diags = d;
}

static GcSymbol *sym_lookup(GcSymTab *tab, const char *name) {
    for (int i = 0; i < tab->count; i++) {
        if (tab->table[i].name && strcmp(tab->table[i].name, name) == 0)
            return &tab->table[i];
    }
    return NULL;
}

static GcSymbol *sym_add(GcSymTab *tab, const char *name, GcType type) {
    if (tab->count >= GC_SYMTAB_SIZE) return NULL;
    GcSymbol *s = &tab->table[tab->count++];
    s->name = strdup(name);
    s->type = type;
    s->ir_value = NULL;
    s->is_param = 0;
    s->is_gpu = 0;
    return s;
}

static GcType type_from_name(const char *n) {
    if (!n) return gc_type_int32();
    if (strcmp(n, "FLOAT32") == 0 || strcmp(n, "GPU-F32") == 0 ||
        strcmp(n, "F32") == 0) return gc_type_float32();
    if (strcmp(n, "INT32") == 0 || strcmp(n, "GPU-I32") == 0)
        return gc_type_int32();
    if (strcmp(n, "POINTER") == 0 || strcmp(n, "GPU-POINTER") == 0)
        return gc_type_pointer(gc_type_float32());
    if (strcmp(n, "MASK") == 0 || strcmp(n, "GPU-MASK") == 0)
        return gc_type_mask();
    return gc_type_float32();
}

static GcIrValue *emit_expr(GcSema *s, GcAstNode *expr);

static GcIrValue *emit_literal(GcSema *s, GcAstNode *n) {
    GcType ty = gc_type_int32();
    if (n->u.literal.lit_kind == TOK_FLOAT)
        ty = gc_type_float32();
    GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                       IR_CONST, ty);
    if (n->u.literal.lit_kind == TOK_FLOAT)
        inst->imm_f = n->u.literal.float_val;
    else
        inst->imm_i = n->u.literal.int_val;
    return inst->result;
}

static GcIrValue *emit_ident(GcSema *s, GcAstNode *n) {
    const char *name = n->u.ident.name;
    if (!name) return NULL;
    GcSymbol *sym = sym_lookup(&s->locals, name);
    if (!sym) sym = sym_lookup(&s->globals, name);
    if (!sym) {
        gc_diag_emit(s->diags, GC_DIAG_ERROR, "sema", n->line, n->column,
                     "undefined identifier '%s'", name);
        return NULL;
    }
    return sym->ir_value;
}

static GcIrValue *emit_program_id(GcSema *s, GcAstNode *n) {
    GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                       IR_PROGRAM_ID, gc_type_int32());
    inst->axis = n->u.program_id.axis;
    return inst->result;
}

static GcIrValue *emit_lane_id(GcSema *s, GcAstNode *n) {
    (void)n;
    GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                       IR_LANE_ID, gc_type_int32());
    return inst->result;
}

static GcIrValue *emit_binary(GcSema *s, GcAstNode *n) {
    GcIrValue *l = emit_expr(s, n->u.binary.left);
    GcIrValue *r = emit_expr(s, n->u.binary.right);
    if (!l || !r) return NULL;
    GcType ty = l->type;
    if (n->u.binary.op == TOK_LT || n->u.binary.op == TOK_LE ||
        n->u.binary.op == TOK_GT || n->u.binary.op == TOK_GE ||
        n->u.binary.op == TOK_EQ || n->u.binary.op == TOK_NE ||
        n->u.binary.op == TOK_AND || n->u.binary.op == TOK_OR)
        ty = gc_type_mask();
    GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                       IR_BINOP, ty);
    inst->bin_op = n->u.binary.op;
    gc_ir_inst_add_operand(inst, l);
    gc_ir_inst_add_operand(inst, r);
    return inst->result;
}

static GcIrValue *emit_expr(GcSema *s, GcAstNode *expr) {
    if (!expr) return NULL;
    switch (expr->kind) {
        case AST_EXPR_LITERAL: return emit_literal(s, expr);
        case AST_EXPR_IDENT: return emit_ident(s, expr);
        case AST_EXPR_PROGRAM_ID: return emit_program_id(s, expr);
        case AST_EXPR_LANE_ID: return emit_lane_id(s, expr);
        case AST_EXPR_BINARY: return emit_binary(s, expr);
        case AST_EXPR_UNARY: {
            GcIrValue *o = emit_expr(s, expr->u.unary.operand);
            if (!o) return NULL;
            GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                               IR_UNOP, o->type);
            inst->bin_op = expr->u.unary.op;
            gc_ir_inst_add_operand(inst, o);
            return inst->result;
        }
        default:
            return NULL;
    }
}

static void emit_stmt(GcSema *s, GcAstNode *stmt);

static void emit_gpu_load(GcSema *s, GcAstNode *n) {
    /* children: ptr, [offsets], [shape], [mask], [other], dest */
    GcIrValue *ptr = NULL;
    GcIrValue *off = NULL;
    GcIrValue *mask = NULL;
    GcIrValue *dest_sym = NULL;
    for (int i = 0; i < n->n_children; i++) {
        GcAstNode *c = n->children[i];
        if (c->kind == AST_EXPR_IDENT) {
            if (!ptr) ptr = emit_expr(s, c);
            else if (!dest_sym && n->u.memop.dest == c)
                dest_sym = emit_expr(s, c);
        }
    }
    if (n->u.memop.offsets) off = emit_expr(s, n->u.memop.offsets);
    if (n->u.memop.mask) mask = emit_expr(s, n->u.memop.mask);

    GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                       IR_LOAD, gc_type_float32());
    if (ptr) gc_ir_inst_add_operand(inst, ptr);
    if (off) gc_ir_inst_add_operand(inst, off);
    if (mask) gc_ir_inst_add_operand(inst, mask);

    /* bind result to destination name if present */
    if (n->u.memop.dest && n->u.memop.dest->kind == AST_EXPR_IDENT) {
        const char *dname = n->u.memop.dest->u.ident.name;
        GcSymbol *sym = sym_lookup(&s->locals, dname);
        if (!sym)
            sym = sym_add(&s->locals, dname, gc_type_float32());
        if (sym)
            sym->ir_value = inst->result;
    }
}

static void emit_gpu_store(GcSema *s, GcAstNode *n) {
    GcIrValue *ptr = NULL, *off = NULL, *val = NULL, *mask = NULL;
    if (n->n_children > 0) ptr = emit_expr(s, n->children[0]);
    if (n->u.memop.offsets) off = emit_expr(s, n->u.memop.offsets);
    if (n->u.memop.dest) val = emit_expr(s, n->u.memop.dest);
    if (n->u.memop.mask) mask = emit_expr(s, n->u.memop.mask);

    GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                       IR_STORE, gc_type_float32());
    if (ptr) gc_ir_inst_add_operand(inst, ptr);
    if (off) gc_ir_inst_add_operand(inst, off);
    if (val) gc_ir_inst_add_operand(inst, val);
    if (mask) gc_ir_inst_add_operand(inst, mask);
}

static void emit_compute(GcSema *s, GcAstNode *n) {
    GcIrValue *rhs = emit_expr(s, n->u.assign.rhs);
    if (!rhs) return;
    if (n->u.assign.lhs && n->u.assign.lhs->kind == AST_EXPR_IDENT) {
        const char *name = n->u.assign.lhs->u.ident.name;
        GcSymbol *sym = sym_lookup(&s->locals, name);
        if (!sym)
            sym = sym_add(&s->locals, name, rhs->type);
        if (sym)
            sym->ir_value = rhs;
    }
}

static void emit_stmt_list(GcSema *s, GcAstNode *list) {
    if (!list) return;
    for (int i = 0; i < list->n_children; i++)
        emit_stmt(s, list->children[i]);
}

static void emit_stmt(GcSema *s, GcAstNode *stmt) {
    if (!stmt) return;
    switch (stmt->kind) {
        case AST_STMT_LIST: emit_stmt_list(s, stmt); break;
        case AST_STMT_GPU_LOAD: emit_gpu_load(s, stmt); break;
        case AST_STMT_GPU_STORE: emit_gpu_store(s, stmt); break;
        case AST_STMT_COMPUTE:
        case AST_STMT_SET:
        case AST_STMT_MOVE: emit_compute(s, stmt); break;
        case AST_STMT_GPU_DOT: {
            /* simplified: treat as binop */
            if (stmt->n_children >= 2) {
                GcIrValue *a = emit_expr(s, stmt->children[0]);
                GcIrValue *b = emit_expr(s, stmt->children[1]);
                GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                                   IR_DOT, gc_type_float32());
                if (a) gc_ir_inst_add_operand(inst, a);
                if (b) gc_ir_inst_add_operand(inst, b);
                if (stmt->n_children >= 3 && stmt->children[2]->kind == AST_EXPR_IDENT) {
                    const char *dn = stmt->children[2]->u.ident.name;
                    GcSymbol *sym = sym_lookup(&s->locals, dn);
                    if (!sym) sym = sym_add(&s->locals, dn, gc_type_float32());
                    if (sym) sym->ir_value = inst->result;
                }
            }
            break;
        }
        case AST_STMT_GPU_REDUCE: {
            if (stmt->n_children >= 1) {
                GcIrValue *v = emit_expr(s, stmt->children[0]);
                GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                                   IR_REDUCE, gc_type_float32());
                if (v) gc_ir_inst_add_operand(inst, v);
            }
            break;
        }
        case AST_STMT_GPU_SYNC: {
            gc_ir_inst_create(s->current_kernel, s->current_block,
                              IR_BARRIER, gc_type_int32());
            break;
        }
        case AST_STMT_IF: {
            /* emit condition; full CFG later */
            emit_expr(s, stmt->u.ifstmt.cond);
            emit_stmt_list(s, stmt->u.ifstmt.then_body);
            if (stmt->u.ifstmt.else_body)
                emit_stmt_list(s, stmt->u.ifstmt.else_body);
            break;
        }
        case AST_STMT_GOBACK: {
            gc_ir_inst_create(s->current_kernel, s->current_block,
                              IR_RETURN, gc_type_int32());
            break;
        }
        default:
            break;
    }
}

static void analyze_kernel(GcSema *s, GcAstNode *kdef) {
    if (!kdef || kdef->kind != AST_KERNEL_DEF) return;
    const char *kname = kdef->u.kernel.name ? kdef->u.kernel.name : "unnamed";
    GcIrKernel *k = gc_ir_kernel_create(s->module, kname);
    s->current_kernel = k;
    s->current_block = k->entry;
    s->locals.count = 0;

    for (int i = 0; i < kdef->u.kernel.n_params; i++) {
        GcAstNode *p = kdef->u.kernel.params[i];
        if (!p) continue;
        GcType ty = type_from_name(p->u.param.type_name);
        if (p->u.param.is_pointer)
            ty = gc_type_pointer(gc_type_float32());
        GcIrValue *pv = gc_ir_value_create(k, ty, p->u.param.name);
        if (k->n_params < GC_MAX_PARAMS)
            k->params[k->n_params++] = pv;
        GcSymbol *sym = sym_add(&s->locals, p->u.param.name, ty);
        if (sym) {
            sym->ir_value = pv;
            sym->is_param = 1;
            sym->is_gpu = 1;
        }
    }
}

static void analyze_procedure(GcSema *s, GcAstNode *proc) {
    if (!proc) return;
    /* if no kernel was defined, create a default one for the procedure body */
    if (!s->current_kernel) {
        s->current_kernel = gc_ir_kernel_create(s->module, "main_kernel");
        s->current_block = s->current_kernel->entry;
    }
    for (int i = 0; i < proc->n_children; i++)
        emit_stmt(s, proc->children[i]);
}

/* Bind WORKING-STORAGE items as globals: each named item is materialised as
 * an IR_CONST (its VALUE literal, or 0) in the current kernel's block. */
static void bind_globals(GcSema *s, GcAstNode *data_div) {
    if (!data_div) return;
    if (!s->current_kernel) {
        s->current_kernel = gc_ir_kernel_create(s->module, "main_kernel");
        s->current_block = s->current_kernel->entry;
    }
    for (int i = 0; i < data_div->n_children; i++) {
        GcAstNode *d = data_div->children[i];
        if (!d || d->kind != AST_DATA_ITEM || !d->u.data_item.name) continue;
        GcAstNode *v = d->u.data_item.value;
        GcType ty = (v && v->u.literal.lit_kind == TOK_FLOAT)
            ? gc_type_float32() : gc_type_int32();
        GcIrInst *inst = gc_ir_inst_create(s->current_kernel, s->current_block,
                                           IR_CONST, ty);
        if (v) {
            inst->imm_i = v->u.literal.int_val;
            inst->imm_f = v->u.literal.float_val;
        }
        GcSymbol *sym = sym_lookup(&s->globals, d->u.data_item.name);
        if (!sym) sym = sym_add(&s->globals, d->u.data_item.name, ty);
        if (sym) sym->ir_value = inst->result;
    }
}

GcIrModule *gc_sema_analyze(GcSema *s, GcAstNode *ast) {
    if (!ast || ast->kind != AST_PROGRAM) {
        gc_diag_emit(s->diags, GC_DIAG_ERROR, "sema", 0, 0,
                     "expected PROGRAM AST node");
        return NULL;
    }
    const char *modname = ast->u.program.program_name
        ? ast->u.program.program_name : "gpu_cobol_module";
    s->module = gc_ir_module_create(s->arena, modname);

    if (ast->u.program.kernel_sec) {
        for (int i = 0; i < ast->u.program.kernel_sec->n_children; i++)
            analyze_kernel(s, ast->u.program.kernel_sec->children[i]);
    }
    bind_globals(s, ast->u.program.data_div);
    if (ast->u.program.proc_div)
        analyze_procedure(s, ast->u.program.proc_div);

    gc_ir_validate(s->module, s->diags);
    return s->module;
}

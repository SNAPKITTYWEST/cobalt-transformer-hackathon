# =====================================================================
# COBOL 2024 BANKING SUITE
# Requires: GnuCOBOL 3.2+ (cobc), bash
# =====================================================================
COBC := cobc
STD := -std=ibm
FLAGS := -free -Wall
BUILD_DIR := build
BIN := $(BUILD_DIR)/bin
DAT := $(BUILD_DIR)/data
PROGRAMS := bank2024 bankidx bankjson bankoo
BINS := $(patsubst %,$(BIN)/%,$(PROGRAMS))
SRCS := $(patsubst %,%.cob,$(PROGRAMS)) $(BANK2024_SUITE)
.PHONY: all clean test check fmt help seed data
all: $(BINS) $(DAT)/accounts.dat
# ---- build ---------------------------------------------------------
$(BIN)/%: %.cob | $(BIN)
	$(COBC) -x $(STD) $(FLAGS) -o $@ $<
	@echo " [BUILD] $@"
$(BIN):
	mkdir -p $(BIN)
$(DAT):
	mkdir -p $(DAT)
# ---- test data -----------------------------------------------------
$(DAT)/accounts.dat: accounts.dat | $(DAT)
	cp $< $@
	@echo " [DATA ] $@"
seed: $(BIN)/bankidx
	rm -f $(DAT)/bankidx.dat*
	cd $(DAT) && ../bin/bankidx CREATE
# ---- tests ---------------------------------------------------------
test: all $(BIN)/banktest
	@bash run-tests.sh
	@echo ""
	@echo "== pure COBOL unit tests =="
	$(BIN)/banktest
check: test
# ---- formatting / lint ---------------------------------------------
fmt:
	@cobc -x $(STD) $(FLAGS) -fsyntax-only $(SRCS) && echo " [SYNTAX] OK"
# ---- coverage-ish (trace execution) --------------------------------
trace: $(BIN)/bank2024
	CO_SET_DEBUG=Y $(BIN)/bank2024
# ---- housekeeping --------------------------------------------------
clean:
	rm -rf $(BUILD_DIR)
help:
	@echo "Targets:"
	@echo " all - build all programs + test data (default)"
	@echo " test - run bash + COBOL unit test suites"
	@echo " seed - (re)create indexed test database"
	@echo " fmt - syntax-check all sources"
	@echo " clean - remove build/"

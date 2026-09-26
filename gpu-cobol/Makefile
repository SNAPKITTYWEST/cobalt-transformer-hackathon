CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -O2 -g -Iinclude
LDFLAGS =

SRCS = src/common.c src/token.c src/lexer.c src/ast.c src/parser.c \
       src/ir.c src/sema.c src/ptx.c src/main.c
OBJS = $(SRCS:src/%.c=build/%.o)

.PHONY: all clean test

all: bin/gpu-cobol

bin/gpu-cobol: $(OBJS)
	@mkdir -p bin
	$(CC) $(CFLAGS) -o $@ $(OBJS) $(LDFLAGS)

build/%.o: src/%.c
	@mkdir -p build
	$(CC) $(CFLAGS) -c -o $@ $<

test: bin/gpu-cobol
	./bin/gpu-cobol --emit-ast examples/vector-add.cbl
	./bin/gpu-cobol --emit-ir examples/vector-add.cbl
	./bin/gpu-cobol --emit-ptx --arch=sm_86 examples/vector-add.cbl

clean:
	rm -rf build bin

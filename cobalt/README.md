# Recursive COBOL Nugget Compiler — First Vertical Slice

**Corpus:** https://github.com/AHMADALIPARR/build-your-own-x  
**Selected tutorial family:** Build your own Web Server  
**Capability:** HTTP request-line parse + validate  
**Date:** 2026-09-25

This slice implements the pipeline in MASTER DIRECTIVE §14 without pretending
GnuCOBOL is installed in this sandbox. Sources are portable COBOL-85 / COBOL-2002
style intended for `cobc -x -free`.

## Selected source

Tutorial family listed under “Build your own Web Server” in the BYOX index.
Capability extracted: parse an HTTP/1.1 request line

```
METHOD SP request-target SP HTTP-version CRLF
```

and emit APPROVED / DECLINED plus extracted fields.

## Nuggets (7)

| ID | Name | Role |
|----|------|------|
| N01 | VALIDATE-BUFFER | Precondition / length / printable |
| N02 | FIND-FIRST-SPACE | Locate delimiter |
| N03 | EXTRACT-METHOD | Token 1 |
| N04 | EXTRACT-TARGET | Token 2 |
| N05 | EXTRACT-VERSION | Token 3 |
| N06 | VALIDATE-VERSION | HTTP/1.0 or HTTP/1.1 |
| N07 | PARSE-REQUEST-LINE | Composition of N01–N06 |

N07 is the composed program `REQPARSE.cbl`.

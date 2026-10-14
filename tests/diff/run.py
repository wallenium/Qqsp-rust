#!/usr/bin/env python3
"""Differenztest: Rust-Engine gegen die originale C-Engine (libqsp 5.7.0).

  QQSP_SRC=/pfad/zu/Qqsp python3 tests/diff/run.py [--fuzz N] [--seed S]

Benötigt gcc und cargo. Baut die C-Engine (aus $QQSP_SRC/qsp) und den Rust-Treiber, führt statische Testfälle
und N zufällige Ausdrücke aus und vergleicht Meldungen, Fenster-Callbacks, Fehlercodes und Beschreibungstext.
"""
import os, random, subprocess, sys, tempfile
from gen import build
from corpus_static import F, T

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
SRC = os.environ.get("QQSP_SRC")
if not SRC:
    sys.exit("QQSP_SRC setzen (Ordner des Qqsp-Repos mit dem Unterordner qsp/)")
S = os.path.join(SRC, "qsp")
OUT = tempfile.mkdtemp(prefix="qsp-diff-")
CDRV = os.path.join(OUT, "cdrv")

def sh(cmd, **kw):
    r = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if r.returncode:
        sys.exit(f"Fehler bei {cmd[:3]}:\n{r.stderr[-1500:]}")
    return r

print("baue C-Referenz …")
import glob
c_files = glob.glob(f"{S}/*.c") + glob.glob(f"{S}/onig/*.c") + glob.glob(f"{S}/onig/enc/*.c") + glob.glob(f"{S}/bindings/default/*.c")
sh(["gcc", "-O1", "-w", "-D_UNICODE", "-DNOT_RUBY", "-DEXPORT", f"-I{S}", f"-I{S}/onig", os.path.join(HERE, "drv.c"), *c_files, "-o", CDRV, "-lm"])
print("baue Rust-Treiber …")
sh(["cargo", "build", "--release", "--example", "rdrv"], cwd=ROOT)
RDRV = os.path.join(ROOT, "target", "release", "examples", "rdrv")

def run_both(code, extra=()):
    path = os.path.join(OUT, "t.qsp")
    open(path, "wb").write(build([("start", code, []), F, *extra]))
    c = subprocess.run([CDRV, path], capture_output=True, text=True, timeout=10).stdout
    r = subprocess.run([RDRV, path], capture_output=True, text=True, timeout=10).stdout
    return c, r

# Bekannter Unterschied: C gibt INT_MIN als Müll aus (Fehler in qspNumToStr), Rust korrekt.
def known(c, r):
    return "-2147483648" in r

def fuzz_expr(rng, depth=0):
    """Zufälliger Ausdruck; Typen werden absichtlich gemischt (Typfehler werden mitverglichen)."""
    def num():
        return str(rng.choice([0, 1, 2, 3, 5, 7, 10, 100, -1, -5, 2147483647, 65536, 46341]))
    def string():
        return "'" + rng.choice(["", "a", "abc", "Hello", "12", "x y", "é", "ÄÖ", "a.c", "  p  ", "-3"]) + "'"
    if depth > 3 or rng.random() < 0.25:
        return rng.choice([num, string])()
    k = rng.random()
    a, b = fuzz_expr(rng, depth + 1), fuzz_expr(rng, depth + 1)
    if k < 0.35:
        return f"({a} {rng.choice(['+', '-', '*', '/', 'mod', '<', '>', '=', '<>', '<=', '>=', 'and', 'or'])} {b})"
    if k < 0.45:
        return f"(not {a})" if rng.random() < 0.5 else f"(-{a})"
    fn = rng.choice(["len", "trim", "ucase", "lcase", "str", "val", "isnum", "abs", "sqr", "mid", "left", "right", "instr", "replace",
                     "iif", "min", "max", "strcomp", "strfind", "strpos", "rand", "arrsize", "isplay", "desc"])
    two = {"left", "right", "instr", "min", "max", "strcomp", "strfind", "strpos", "rand"}
    three = {"mid", "replace", "iif"}
    c = fuzz_expr(rng, depth + 1)
    if fn in three:
        return f"{fn}({a},{b},{c})"
    if fn in two:
        return f"{fn}({a},{b})"
    return f"{fn}({a})"

def main():
    n, seed = 500, 1
    argv = sys.argv[1:]
    if "--fuzz" in argv: n = int(argv[argv.index("--fuzz") + 1])
    if "--seed" in argv: seed = int(argv[argv.index("--seed") + 1])
    cases = list(T)
    rng = random.Random(seed)
    cases += [f"msg {fuzz_expr(rng)}" for _ in range(n)]
    # Mehrere Anweisungen mit Variablen
    for _ in range(max(n // 5, 1)):
        v = fuzz_expr(rng)
        cases.append(f"x={v}\n$s={fuzz_expr(rng)}\nmsg x\nmsg $s\nmsg arrsize('x')\nmsg arrsize('s')")
    bad = ign = 0
    for i, t in enumerate(cases):
        c, r = run_both(t)
        if c == r:
            continue
        if known(c, r):
            ign += 1
            continue
        bad += 1
        print(f"DIFF #{i}\n  Code: {t!r}\n  C : {c!r}\n  RS: {r!r}")
        if bad >= 25:
            print("… Abbruch nach 25 Unterschieden"); break
    print(f"Fälle: {len(cases)}, Unterschiede: {bad}, bekannte (INT_MIN): {ign}")
    sys.exit(1 if bad else 0)

main()

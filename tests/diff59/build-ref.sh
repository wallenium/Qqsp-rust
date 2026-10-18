#!/bin/sh
# usage: build-ref.sh <qsp-5.9.6-checkout> <oniguruma-checkout>
set -e
Q=$1; O=$2; B=$(mktemp -d)
cmake -S "$O" -B "$B/onig" -DBUILD_SHARED_LIBS=OFF -DBUILD_TEST=OFF -DCMAKE_C_FLAGS="-DUSE_CRNL_AS_LINE_TERMINATOR=1 -fPIC" -DCMAKE_BUILD_TYPE=Release >/dev/null
make -C "$B/onig" -j4 >/dev/null
printf '#define QSP_VER_STR "5.9.6"\n#define QSP_GAMEMINVER_STR "5.9.6"\n' > "$B/qsp_config.h"
printf '#define QSP_EXTERN\n' > "$B/qsp_export.h"
mkdir "$B/o"; ( cd "$B/o" && gcc -O1 -D_UNICODE -fwrapv -w -I"$B" -I"$O/src" -I"$B/onig" -I"$Q" -c "$Q"/qsp/*.c "$Q"/qsp/bindings/default/*.c )
gcc -O1 -D_UNICODE -w -I"$Q/qsp" -I"$B" -o cdrv59 cdrv59.c "$B"/o/*.o "$B/onig/libonig.a" -lm

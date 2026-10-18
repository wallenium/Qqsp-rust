# Differenztest „5.9-Funktionen“

Vergleicht die additiven 5.9-Erweiterungen (`src/ext59.rs`) mit der offiziellen libqsp 5.9.6.

    git clone --depth 1 --branch 5.9.6 https://github.com/QSPFoundation/qsp
    git clone --depth 1 https://github.com/kkos/oniguruma
    ./build-ref.sh /pfad/zu/qsp /pfad/zu/oniguruma      # erzeugt ./cdrv59
    cargo build --release --examples
    python3 run59.py          # feste Fälle (cases59.txt; "!known" = dokumentierte Typmodell-Unterschiede)
    python3 fuzz59.py 1 800   # Zufallsprogramme

Bekannte Unterschiede (Absicht: der Standardmodus bleibt 5.7): Variablen behalten das 5.7-Modell
(Zahl und Text je Eintrag, Lücken lesen als 0), 5.9 speichert genau einen typisierten Wert.
Bei einheitlich typisierten Arrays ohne Lücken sind die Ergebnisse identisch.

<!-- [W·C1·AK] QUALITY.nl.md -->
# Kwaliteitsrapport

Deze pagina laat zien hoe otel-span-bill is gecontroleerd. Ze beschrijft controles die we deden. Het is geen garantie dat de software foutloos is of dat de schattingen met een factuur overeenkomen. De Engelse versie is [QUALITY.md](QUALITY.md).

## 0.1.0

| | |
|---|---|
| Geteste commit | `cf8c5d7d3713b2cabfad1270c984c909c1128946` (eerste commit van de reparatieronde van 2026-11-24; de documenten daarbovenop zijn de enige latere wijziging) |
| Tests | 152 geautomatiseerde tests (0 mislukt) |

### Wat getest wordt

- 90 unittests in `src/` (groottebepaling, tellen, invoer, kosten, weergave, prijzen).
- 47 tests in `tests/cli.rs` die het programma draaien op de bestanden in `tests/fixtures` en op gegenereerde bestanden: afsluitcodes, opties, foutmeldingen, mappen, JSON-regels, een byte order mark, binaire invoer, een bestand van 20.000 spans, een mislukte schrijfactie op stdout, een volle of gesloten stderr, een gesloten lezer.
- 5 tests in `tests/size_golden.rs`: de groottebepaling is gelijk aan het aantal bytes van een echte protobuf-encoder.
- 10 tests in `tests/readme.rs`: elke opdracht in de Quick start van de README wordt uitgevoerd en de uitvoer vergeleken; de koppen, prijzen, URL's en datum in de README komen overeen met het programma; het aantal tests in README en op deze pagina komt overeen met de suite; licenties en changelog zijn aanwezig.

### Meting van 2026-11-24

Opdracht: `cargo test --locked --no-fail-fast -j 2`, vanuit de hoofdmap van de repository, Linux, op de boom van de documentencommit direct boven `9e53b17`. Die commit wijzigt alleen documenten; alle code en tests zijn die van `9e53b17`.

Resultaat: 147 geslaagd, 0 mislukt (90 unittests, 42 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 doctests). De broncode is in deze ronde niet gewijzigd.

STAND: groen

### Niet uitgevoerd

- `cargo fmt --check`, `cargo clippy`, `cargo test --release`, `cargo +1.85 test`
- `cargo deny`, `cargo audit`, een GitHub Actions-run, een release-run
- `cargo publish --dry-run`
- fuzzing, property tests en mutation testing
- er is niets naar GitHub gestuurd

### Meting van 2026-11-24 (reparatieronde)

Opdracht: `cargo test --locked --no-fail-fast -j 2`, plus `cargo fmt --check` en `cargo clippy --all-targets --locked -- -D warnings`, vanuit de hoofdmap, Linux, op de boom van de documentencommit direct boven `cf8c5d7d3713b2cabfad1270c984c909c1128946`. Die commit wijzigt alleen documenten.

Wijziging: een mislukte schrijfactie op stdout (behalve een gesloten lezer) geeft nu exit 2, en een volle of gesloten stderr geeft exit 2 zonder paniek (eerder: exit 0 en exit 101). 5 tests erbij in `tests/cli.rs`; op de oude `src/main.rs` zijn er 3 rood, de andere 2 bewaken gedrag dat al goed was.

Resultaat: 152 geslaagd, 0 mislukt (90 unittests, 47 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 doctests).

STAND: groen

### Eerdere metingen

De meting van 0.1.0 op 2026-11-09 en die van 2026-11-18 staan in [QUALITY.md](QUALITY.md), met de commando's en uitslagen.

<!-- [W·C1·AK] QUALITY.nl.md -->
# Kwaliteitsrapport

Deze pagina laat zien hoe otel-span-bill is gecontroleerd. Ze beschrijft controles die we deden. Het is geen garantie dat de software foutloos is of dat de schattingen met een factuur overeenkomen. De Engelse versie is [QUALITY.md](QUALITY.md).

## 0.1.0

| | |
|---|---|
| Geteste commit | `5e746f2c556c2874d2ca1f1e74246a2390c0c0bc` (de fix voor een gesloten stdout, bovenop `origin/main` `603ad2b`; de commit met deze pagina staat direct erboven en wijzigt alleen documenten) |
| Datum | 2026-11-25 |
| Tests | 157 geautomatiseerde tests (0 mislukt) |

### Wat getest wordt

- 90 unittests in `src/` (groottebepaling, tellen, invoer, kosten, weergave, prijzen).
- 52 tests in `tests/cli.rs` die het programma draaien op de bestanden in `tests/fixtures` en op gegenereerde bestanden: afsluitcodes, opties, foutmeldingen, mappen, JSON-regels, een byte order mark, binaire invoer, een bestand van 20.000 spans, een mislukte schrijfactie op stdout, een volle of gesloten stderr, een gesloten lezer, een gesloten stdout bij de start (exit 2) en stdout op /dev/null (exit 0), paden met stuurtekens in foutmeldingen.
- 5 tests in `tests/size_golden.rs`: de groottebepaling is gelijk aan het aantal bytes van een echte protobuf-encoder, voor een span met elk soort veld en voor de resource- en scope-koppen.
- 10 tests in `tests/readme.rs`: elke opdracht in de Quick start van de README wordt uitgevoerd en de uitvoer vergeleken; elke bewering in "How it counts" en "Limitations" noemt tests die bestaan; de prijzen, URL's en datum in de README komen overeen met het programma; het aantal tests in README en op deze pagina komt overeen met de suite; licenties en changelog zijn aanwezig.

### Hoe we controleerden

Gedraaid op 2026-11-25 vanuit de hoofdmap van een schone worktree van commit `5e746f2`, Linux, Rust 1.99.0 stable (`rustc 1.99.0 (b940084d7 2026-09-28)`). Aan de boom is niets gewijzigd om deze uitslag te krijgen.

| Opdracht | Uitslag |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --locked` | exit 0; 157 geslaagd, 0 mislukt (90 unit, 52 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 in `src/main.rs`, 0 doctests) |
| `cargo test --release --locked` | exit 0; 157 geslaagd, 0 mislukt (zelfde verdeling) |

Ook gedraaid op 2026-11-25 op dezelfde code (`git diff 5e746f2 9e3660d` toont geen wijziging in `src/`, `tests/`, `Cargo.toml` of `Cargo.lock`; uitgecheckt op `9e3660d`), Linux, Rust 1.85.1 (`rustc 1.85.1 (4eb161250 2025-03-15)`), de minimaal ondersteunde versie:

| Opdracht | Uitslag |
|---|---|
| `cargo +1.85 fmt --check` | exit 0 |
| `cargo +1.85 clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo +1.85 test --locked` | exit 0; 157 geslaagd, 0 mislukt (90 unit, 52 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 in `src/main.rs`, 0 doctests) |

STAND: groen

### Niet uitgevoerd

Deze meting deed dit niet (not run). Eerdere versies van dit bestand meldden een deel ervan voor oudere commits; voor `5e746f2` is het niet herhaald.

- gedraaid op 2026-11-25, Rust 1.85.1: `cargo +1.85 test --release --locked` exit 0; 157 geslaagd, 0 mislukt, 0 genegeerd (90 unit, 52 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 in `src/main.rs`, 0 doctests)
- niet uitgevoerd (not run): `cargo publish --dry-run`
- niet uitgevoerd (not run): de vorm `cargo clippy --all-targets -- -D warnings` zonder `--locked` (de pagina van 2026-11-09 gebruikte die; deze meting gebruikte de `--locked`-vorm hierboven)
- niet uitgevoerd (not run): `cargo deny`, `cargo audit`
- niet uitgevoerd (not run): Windows en macOS, en elke GitHub Actions-run (er is niets gepusht)
- niet uitgevoerd (not run): fuzzing, property tests, mutation testing

### Wat we niet controleerden

- De schattingen zijn niet vergeleken met een echte Datadog- of Grafana-factuur. De pagina's van de aanbieders zeggen niet welk aantal bytes ze rekenen.
- Alleen Linux is gebruikt. De herkenning van een gesloten stdout bij de start bestaat alleen op Linux (andere platformen houden exit 0 zonder melding). De tests met `/dev/full` of `sh` worden alleen op Linux gecompileerd, dus de bewering "exit 2 zonder paniek" bij een mislukte stdout en een volle of gesloten stderr is op Windows en macOS niet getest.
- De ingebouwde tarieven zijn in deze meting niet opnieuw naast de pagina's van de aanbieders gelegd.

`task.md` en `blueprint.md` zijn lokale werkbestanden: ze blijven buiten git en staan in `.gitignore`.

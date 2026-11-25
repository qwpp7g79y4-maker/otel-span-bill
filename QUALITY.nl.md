<!-- [W·C1·AK] QUALITY.nl.md -->
# Kwaliteitsrapport

Deze pagina laat zien hoe otel-span-bill is gecontroleerd. Ze beschrijft controles die we deden. Het is geen garantie dat de software foutloos is of dat de schattingen met een factuur overeenkomen. De Engelse versie is [QUALITY.md](QUALITY.md).

## 0.1.0

| | |
|---|---|
| Geteste commit | `6cb28ffb6bad77080daee60c3005e25e505eac28` (`origin/main` op 2026-11-25; de commit met deze pagina staat direct erboven en wijzigt alleen documenten) |
| Datum | 2026-11-25 |
| Tests | 155 geautomatiseerde tests (0 mislukt) |

### Wat getest wordt

- 90 unittests in `src/` (groottebepaling, tellen, invoer, kosten, weergave, prijzen).
- 50 tests in `tests/cli.rs` die het programma draaien op de bestanden in `tests/fixtures` en op gegenereerde bestanden: afsluitcodes, opties, foutmeldingen, mappen, JSON-regels, een byte order mark, binaire invoer, een bestand van 20.000 spans, een mislukte schrijfactie op stdout, een volle of gesloten stderr, een gesloten lezer, paden met stuurtekens in foutmeldingen.
- 5 tests in `tests/size_golden.rs`: de groottebepaling is gelijk aan het aantal bytes van een echte protobuf-encoder, voor een span met elk soort veld en voor de resource- en scope-koppen.
- 10 tests in `tests/readme.rs`: elke opdracht in de Quick start van de README wordt uitgevoerd en de uitvoer vergeleken; elke bewering in "How it counts" en "Limitations" noemt tests die bestaan; de prijzen, URL's en datum in de README komen overeen met het programma; het aantal tests in README en op deze pagina komt overeen met de suite; licenties en changelog zijn aanwezig.

### Hoe we controleerden

Gedraaid op 2026-11-25 vanuit de hoofdmap van een schone worktree van commit `6cb28ff`, Linux, Rust 1.99.0 stable (`rustc 1.99.0 (b940084d7 2026-09-28)`). Aan de boom is niets gewijzigd om deze uitslag te krijgen.

| Opdracht | Uitslag |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --locked` | exit 0; 155 geslaagd, 0 mislukt (90 unit, 50 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 in `src/main.rs`, 0 doctests) |
| `cargo test --release --locked` | exit 0; 155 geslaagd, 0 mislukt (zelfde verdeling) |

STAND: groen

### Niet uitgevoerd

Deze meting deed dit niet (not run). Eerdere versies van dit bestand meldden een deel ervan voor oudere commits; voor `6cb28ff` is het niet herhaald.

- niet uitgevoerd (not run): `cargo +1.85 test` (de minimaal ondersteunde Rust-versie)
- niet uitgevoerd (not run): `cargo publish --dry-run`
- niet uitgevoerd (not run): de vorm `cargo clippy --all-targets -- -D warnings` zonder `--locked` (de pagina van 2026-11-09 gebruikte die; deze meting gebruikte de `--locked`-vorm hierboven)
- niet uitgevoerd (not run): `cargo deny`, `cargo audit`
- niet uitgevoerd (not run): Windows en macOS, en elke GitHub Actions-run (er is niets gepusht)
- niet uitgevoerd (not run): fuzzing, property tests, mutation testing

### Wat we niet controleerden

- De schattingen zijn niet vergeleken met een echte Datadog- of Grafana-factuur. De pagina's van de aanbieders zeggen niet welk aantal bytes ze rekenen.
- Alleen Linux is gebruikt. De tests met `/dev/full` of `sh` worden alleen op Linux gecompileerd, dus de bewering "exit 2 zonder paniek" bij een mislukte stdout en een volle of gesloten stderr is op Windows en macOS niet getest.
- De ingebouwde tarieven zijn in deze meting niet opnieuw naast de pagina's van de aanbieders gelegd.

`task.md` en `blueprint.md` zijn lokale werkbestanden: ze blijven buiten git en staan in `.gitignore`.

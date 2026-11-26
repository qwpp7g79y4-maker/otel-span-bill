<!-- [W·C1·AK] README.nl.md -->
# otel-span-bill

[![crates.io](https://img.shields.io/crates/v/otel-span-bill.svg)](https://crates.io/crates/otel-span-bill) [![downloads](https://img.shields.io/crates/d/otel-span-bill.svg)](https://crates.io/crates/otel-span-bill) [![CI](https://github.com/qwpp7g79y4-maker/otel-span-bill/actions/workflows/ci.yml/badge.svg)](https://github.com/qwpp7g79y4-maker/otel-span-bill/actions/workflows/ci.yml) [![PEPAI checked](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fqwpp7g79y4-maker%2Fotel-span-bill%2Fmain%2F.github%2Fbadge.json)](QUALITY.md) [![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://github.com/qwpp7g79y4-maker/otel-span-bill/blob/main/Cargo.toml) [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/qwpp7g79y4-maker/otel-span-bill#license)

Zie hoeveel spans elke service verstuurt en wat dat kost bij Datadog APM en Grafana Cloud Traces, uit OTLP-JSON-bestanden op je eigen machine. Offline.

De tests en de gemeten aantallen staan in `QUALITY.nl.md`. De Engelse versie is [README.md](README.md).

otel-span-bill is een opdrachtregelprogramma in Rust. Het leest OpenTelemetry-tracedata die als OTLP-JSON is opgeslagen, telt spans, traces en een schatting van de datagrootte per service, en schat de maandrekening bij Datadog APM en Grafana Cloud Traces. Het leest alleen bestanden en opent geen netwerkverbinding. 157 geautomatiseerde tests controleren het, zie [QUALITY.nl.md](QUALITY.nl.md).

## Installatie

Versie 0.1.0. Rust 1.85 of nieuwer is nodig. Vanuit een checkout van deze repository:

```sh
cargo install --locked --path .
otel-span-bill --help
```

## Aan de slag in 5 minuten

1. Installeer zoals hierboven.
2. Sla traces op als OTLP-JSON, bijvoorbeeld met de `file` exporter van de OpenTelemetry Collector.
3. Draai `otel-span-bill pad/naar/bestand.json`, of geef een map (een niveau diep: `.json`, `.jsonl`, `.ndjson`).
4. Geef met `--scale N` aan dat de bestanden 1/N van een maand bevatten; dan toont het rapport de maandrekening.
5. Lees de kosten als schatting op lijstprijzen, niet als factuur.

## Opties

`otel-span-bill [OPTIES] <PAD>...`

| Optie | Betekenis |
|---|---|
| `<PAD>...` | OTLP-JSON-bestanden of mappen (een niveau diep). |
| `--json` | Schrijf het resultaat als JSON, met geld in micro-dollars. |
| `--top <N>` | Toon alleen de N grootste services; de totalen blijven alles dekken. |
| `--scale <N>` | De bestanden bevatten 1/N van een maand: vermenigvuldig groottes en aantallen met N. Standaard 1. |
| `--apm-hosts <N>` | Aantal Datadog APM-hosts. Standaard: aantal verschillende `host.name` / `host.id` in de bestanden, anders 1. |
| `--indexed-percent <P>` | Percentage van de spans dat als Datadog indexed span wordt bewaard, 0 tot 100. Standaard 0. |
| `-h`, `--help` / `-V`, `--version` | Toon de hulp of de versie. |

## Voorbeeld

```console
$ otel-span-bill tests/fixtures/basic.json --top 1
```

Dit geeft de grootste service, `checkout` met 3 spans en 2 traces, de regel `... and 1 more services`, de totalen en beide schattingen. De uitvoer staat volledig in het Quick start-voorbeeld van [README.md](README.md); een test voert die voorbeelden uit en vergelijkt de uitvoer. Voor een maandbedrag voeg je `--scale` toe, en met `--json` kan een ander programma het resultaat lezen.

## Afsluitcodes

- `0`: het rapport is geschreven. Een lezer die de pipe eerder sloot (bijvoorbeeld `| head`) is geen fout en blijft stil.
- `2`: een fout: een mislukte schrijfactie op stdout (behalve een gesloten lezer), een pad dat niet te lezen is, ongeldige JSON, JSON die geen OTLP-tracedata is, een map zonder tracebestanden of een foute optie. De melding noemt het bestand en gaat naar stderr; op stdout komt niets.

## Onderhoud

De prijzen staan vast in `src/prices.rs` en hebben een datum. Verandert een leverancier een prijs, pas dan de constanten, de tabel in de README en de datum samen aan; een test vergelijkt ze. Meld problemen en stuur patches via de Issues van de repository. Zie [CONTRIBUTING.md](CONTRIBUTING.md) en [SECURITY.md](SECURITY.md). Wijzigingen staan in [CHANGELOG.md](CHANGELOG.md).

Na een release staat de tool in sluimermodus (dormant): een nieuwe versie komt er alleen bij een gemelde bug, een verzoek of een beveiligingsprobleem. Beveiliging gaat altijd voor.

## Prijzen en verantwoordelijkheid

De prijzen staan in het bestand `src/prices.rs` en in de tabel van [README.md](README.md#prices-and-responsibility), gelezen op 2026-11-09 van de pagina's van de leveranciers. Deze pagina bepaalt geen tarief.

De bedragen zijn een schatting op lijstprijzen. Jij bent verantwoordelijk voor het controleren van de pagina's van de leveranciers en je eigen contract voordat je iets beslist. De makers geven geen garantie dat een schatting gelijk is aan een factuur.

## Licentie

Naar keuze onder [Apache License, Version 2.0](LICENSE-APACHE) of de [MIT-licentie](LICENSE-MIT).

De licenties van de externe crates in het programma staan in [THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html).

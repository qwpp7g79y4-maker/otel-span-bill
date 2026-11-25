<!-- [W·C1·AK] KEUR-NVT.md -->
build.rs: there is no build script, the crate is plain Rust and generates no code at build time.
man: no man page is shipped, the clap help of --help is the manual.
docs: there is no docs directory, README.md and QUALITY.md hold all documentation.
examples: there is no examples directory, the Quick start runs on tests/fixtures.
samples: there is no samples directory, the test fixtures in tests/fixtures are the only sample data.
scripts: there are no helper scripts, cargo is the only build and test tool.
.github/workflows/release.yml: there is no release workflow, nothing is released from this tree and only ci.yml exists.

<!-- [W·C1·AK] CONTRIBUTING.md -->
# Contributing

Thank you for helping. Changes come in through Issues: open an Issue on the repository that describes the problem or the change, and attach the patch there.

- A patch should have a test. A bug fix comes with a test that fails without the fix; a new option or output comes with a test that covers it.
- Run `cargo test --locked` before you send the patch. All tests must pass.
- The README examples and the price table are compared with the program by tests in `tests/readme.rs`. If you change output or prices, update the README and `QUALITY.md` as well.
- Prices are set only in `src/prices.rs`, with the date they were read and the vendor page.
- Report a security problem as described in [SECURITY.md](SECURITY.md), not in a public Issue.

Contributions are licensed under the same terms as the project: MIT OR Apache-2.0.

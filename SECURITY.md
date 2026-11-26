<!-- [W·C1·AK] SECURITY.md -->
# Security

Report a security problem by e-mail to pep_ai@icloud.com. Please do not open a public Issue for it.

otel-span-bill reads files only and has no network client. Even so, a crafted input file could make it fail or use a lot of memory, because files are read whole into memory. Please report such cases.

Only the latest release (0.1.1) is supported. Expect a first answer within a few days; this is a small project and there is no formal response time.

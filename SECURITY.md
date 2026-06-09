# Security Policy

## Reporting Issues

Please report security issues privately to the repository maintainers through GitHub's private vulnerability reporting flow when available.

Do not open a public issue for:

- Private key exposure.
- Payment verification bypasses.
- Replay vulnerabilities.
- Session-token leakage.
- Incorrect settlement or refund behavior.

## Public Repository Safety

This repository is intended to contain only public protocol sketches, SDK surfaces, examples, and documentation.

Do not commit:

- Private keys or seed phrases.
- API keys.
- RPC credentials.
- `.env` files.
- Private deployment addresses.
- Internal planning documents.
- Recorded private prompts or model outputs.

## Current Status

This is a hackathon-era public SDK and protocol sketch. It should not be used as a production payment system without a full security review, contract audit, and provider-specific threat model.

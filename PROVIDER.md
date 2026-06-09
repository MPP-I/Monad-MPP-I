# Provider Integration Guide

This guide describes how an inference provider can expose an MPP-I-compatible endpoint. It is intentionally implementation-neutral and does not include private infrastructure, deployment addresses, or keys.

## Provider Configuration

An MPP-I provider needs:

- Monad chain ID.
- Escrow contract address.
- Provider settlement wallet.
- Input token price in wei.
- Output token price in wei.
- Quote expiry policy.
- Session token lifetime.
- Token metering policy.

The provider should load these values from its own deployment environment or secret manager. The public SDK does not hardcode them.

## Required Endpoints

### `POST /v1/mpp-i/quotes`

Return payment terms for a specific inference request.

Provider responsibilities:

- Validate request shape.
- Compute or store a prompt commitment.
- Bind quote to `agent_id`, `max_spend_wei`, token prices, and expiry.
- Return escrow payment instructions.
- Avoid returning private configuration values.

### `POST /v1/mpp-i/payment-proofs`

Accept proof that the agent submitted the Monad escrow transaction.

Provider responsibilities:

- Verify transaction hash and chain ID.
- Verify payer, provider, escrow amount, and quote/session binding.
- Observe Monad payment state.
- Accept the session when the configured state threshold is reached.
- Return a short-lived session token.

### `POST /v1/mpp-i/chat/completions`

Serve paid streaming inference.

Provider responsibilities:

- Authenticate the session token.
- Reject unpaid, expired, replayed, or mismatched sessions.
- Start compute when the configured Monad state threshold is met.
- Stream token deltas.
- Meter input and output tokens.
- Settle actual usage.
- Refund unused escrow.

## Compute Policy

Providers can choose how much work each Monad state unlocks:

```text
Proposed  -> reserve capacity and prefill
Safe      -> first token and streaming
Finalized -> settlement accounting
Verified  -> audit and reconciliation
```

For latency-sensitive inference, the important difference from generic payment flows is that compute can advance with state confidence instead of blocking until final settlement.

## Token Pricing

MPP-I quotes should expose both input and output token prices in wei:

```text
price_per_input_token_wei
price_per_output_token_wei
```

Providers should keep pricing explicit per quote so agents can cap spend before opening escrow.

## Metering

The first public MPP-I design uses provider-side metering:

```text
amount_due = input_tokens * price_per_input_token + output_tokens * price_per_output_token
refund = max_spend - amount_due
```

Production providers should make tokenization rules explicit and stable across quote, stream, and settlement.

## What Not To Publish

Do not commit:

- Private keys.
- RPC credentials.
- Raw production prompts or outputs.
- Provider infrastructure URLs that are not intended to be public.
- Internal operational runbooks.
- Deployment-specific wallet addresses unless they are intentionally public.

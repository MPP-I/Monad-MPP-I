# Monad MPP-I

Monad MPP-I is a public protocol sketch for inference-native machine payments on Monad.

The goal is to let agents buy streaming inference from providers without API keys, subscriptions, or open-ended postpaid billing. The provider can start inference when the payment reaches Monad `Proposed` state, meter tokens while streaming, settle actual usage from escrow, and refund unused spend.

This public repository is intentionally small. It does not include private demo code, private keys, deployment addresses, recorded model responses, internal planning notes, or provider infrastructure.

## Rust Agent SDK Sketch

The first public artifact is a minimal Rust agent-side SDK sketch in [`src/lib.rs`](src/lib.rs). It shows the intended integration surface for an agent calling an MPP-I inference provider:

1. Request a quote for an inference call.
2. Submit a Monad payment proof after opening escrow.
3. Stream `/v1/mpp-i/chat/completions`.
4. Read final metered settlement and refund data.

The SDK does not sign transactions or manage private keys. Wallet signing and payment submission stay in the agent application's own custody layer.

Additional public docs:

- [`SPEC.md`](SPEC.md): protocol sketch and state mapping.
- [`PROVIDER.md`](PROVIDER.md): provider integration guide.
- [`examples/agent_request.rs`](examples/agent_request.rs): agent-side request flow.
- [`examples/provider_config.rs`](examples/provider_config.rs): provider configuration surface.

```rust
use monad_mpp_i::{
    ChatCompletionRequest, ChatMessage, ChatRole, MppiAgentClient, QuoteRequest,
};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let provider_url = std::env::var("MPPI_PROVIDER_URL")?;
    let agent_id = std::env::var("MPPI_AGENT_ID")?;
    let max_spend_wei = std::env::var("MPPI_MAX_SPEND_WEI")?;
    let payment_tx_hash = std::env::var("MPPI_PAYMENT_TX_HASH")?;
    let prompt = std::env::var("MPPI_PROMPT")?;

    let client = MppiAgentClient::new(provider_url)?;
    let request = QuoteRequest {
        agent_id,
        max_spend_wei,
        inference_request: ChatCompletionRequest {
            plan: Some("Starter".to_owned()),
            model_optimization: true,
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: prompt,
            }],
        },
    };

    let response = client
        .open_paid_chat_completions(request, payment_tx_hash)
        .await?;

    // The response body is expected to stream MPP-I newline-delimited JSON events.
    drop(response);
    Ok(())
}
```

## Public Flow

```text
agent request
  -> quote max spend and token prices
  -> open Monad escrow
  -> provider observes Proposed state
  -> stream inference tokens
  -> meter input and output tokens
  -> settle actual usage
  -> refund unused escrow
```

## Status

Hackathon-era public SDK sketch. The interface is intended to communicate the protocol shape, not to be treated as a finalized production SDK.

## Deck

The overview deck explains the MPP-I thesis, Monad-native payment-state mapping, and the inference payment flow:

[View the MPP-I overview deck](docs/mpp-i-deck.pdf)

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

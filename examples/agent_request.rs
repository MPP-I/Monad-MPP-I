//! Agent-side MPP-I request flow.
//!
//! This example assumes the agent application has already opened a Monad escrow
//! transaction and has the resulting transaction hash. Wallet custody and
//! transaction signing are intentionally outside this SDK sketch.

use monad_mpp_i::{
    parse_stream_event, ChatCompletionRequest, ChatMessage, ChatRole, MppiAgentClient,
    MppiStreamEvent, QuoteRequest,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MppiAgentClient::new(env::var("MPPI_PROVIDER_URL")?)?;
    let quote = QuoteRequest {
        agent_id: env::var("MPPI_AGENT_ID")?,
        max_spend_wei: env::var("MPPI_MAX_SPEND_WEI")?,
        inference_request: ChatCompletionRequest {
            plan: Some(env::var("MPPI_PLAN").unwrap_or_else(|_| "Starter".to_owned())),
            model_optimization: true,
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: env::var("MPPI_PROMPT")?,
            }],
        },
    };

    let payment_tx_hash = env::var("MPPI_PAYMENT_TX_HASH")?;
    let response = client
        .open_paid_chat_completions(quote, payment_tx_hash)
        .await?;

    let body = response.text().await?;
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        match parse_stream_event(line)? {
            MppiStreamEvent::TokenDelta { delta, .. } => {
                print!("{delta}");
            }
            MppiStreamEvent::StreamDone { settlement } => {
                println!();
                println!("settlement_tx={}", settlement.tx_hash);
                println!("amount_due_wei={}", settlement.amount_due_wei);
                println!("refund_wei={}", settlement.refund_wei);
            }
            MppiStreamEvent::Error { code, message } => {
                eprintln!("stream_error code={code} message={message}");
            }
            MppiStreamEvent::StreamStart { .. } | MppiStreamEvent::Usage { .. } => {}
        }
    }

    Ok(())
}

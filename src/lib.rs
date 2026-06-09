//! Agent-side SDK sketch for Monad MPP-I.
//!
//! MPP-I is a payment flow for inference endpoints. An agent requests a quote,
//! opens escrow on Monad, sends a payment proof, then streams tokens once the
//! provider observes the payment in `Proposed` state. Final token usage is
//! metered by the provider and settled from escrow, with unused spend refunded.
//!
//! This crate is intentionally small and public-safe. It does not include
//! private keys, deployment addresses, private provider code, recorded model
//! responses, or any hackathon-only operational details.

use reqwest::{Client, Response, Url};
use serde::{Deserialize, Serialize};

/// Agent client for an inference provider that exposes MPP-I endpoints.
#[derive(Clone, Debug)]
pub struct MppiAgentClient {
    http: Client,
    provider_url: Url,
}

impl MppiAgentClient {
    /// Create a client from a caller-supplied provider URL.
    ///
    /// The SDK never hardcodes provider infrastructure. Applications should pass
    /// the provider URL from their own config, service registry, or discovery
    /// layer.
    pub fn new(provider_url: impl AsRef<str>) -> Result<Self, MppiError> {
        Ok(Self {
            http: Client::new(),
            provider_url: Url::parse(provider_url.as_ref())?,
        })
    }

    /// Request payment terms for an inference call.
    pub async fn quote(&self, request: QuoteRequest) -> Result<QuoteResponse, MppiError> {
        let response = self
            .http
            .post(self.endpoint("/v1/mpp-i/quotes")?)
            .json(&request)
            .send()
            .await?;

        parse_json_response(response).await
    }

    /// Submit proof that the agent opened the Monad payment session.
    ///
    /// The proof can be a Monad transaction hash or another proof type defined
    /// by a future MPP-I method. Signing and transaction submission are kept
    /// outside this public SDK sketch.
    pub async fn submit_payment_proof(
        &self,
        proof: PaymentProofRequest,
    ) -> Result<AcceptedPayment, MppiError> {
        let response = self
            .http
            .post(self.endpoint("/v1/mpp-i/payment-proofs")?)
            .json(&proof)
            .send()
            .await?;

        parse_json_response(response).await
    }

    /// Open the paid streaming inference response.
    ///
    /// Providers should begin streaming after the payment proof is accepted in
    /// Monad `Proposed` state. The returned response body is expected to be
    /// newline-delimited JSON events.
    pub async fn open_chat_completions(
        &self,
        session_token: &str,
        request: ChatCompletionRequest,
    ) -> Result<Response, MppiError> {
        let response = self
            .http
            .post(self.endpoint("/v1/mpp-i/chat/completions")?)
            .bearer_auth(session_token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(MppiError::Provider {
                status: response.status().as_u16(),
                body: response.text().await.unwrap_or_default(),
            })
        }
    }

    /// Convenience path for agents that already have a Monad payment tx hash.
    pub async fn open_paid_chat_completions(
        &self,
        quote: QuoteRequest,
        payment_tx_hash: String,
    ) -> Result<Response, MppiError> {
        let inference_request = quote.inference_request.clone();
        let quote_response = self.quote(quote).await?;
        let accepted = self
            .submit_payment_proof(PaymentProofRequest::monad_tx(
                quote_response.session_id,
                quote_response.agent_id,
                payment_tx_hash,
            ))
            .await?;

        self.open_chat_completions(&accepted.session_token, inference_request)
            .await
    }

    fn endpoint(&self, path: &str) -> Result<Url, MppiError> {
        Ok(self.provider_url.join(path.trim_start_matches('/'))?)
    }
}

/// Quote request for a paid inference session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub agent_id: String,
    pub max_spend_wei: String,
    pub inference_request: ChatCompletionRequest,
}

/// OpenAI-compatible chat completion body with MPP-I demo fields.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(default)]
    pub model_optimization: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// Provider quote returned before the agent opens payment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub session_id: String,
    pub agent_id: String,
    pub provider_wallet: String,
    pub escrow_contract: String,
    pub chain_id: u64,
    pub max_spend_wei: String,
    pub price_per_input_token_wei: String,
    pub price_per_output_token_wei: String,
    pub payment_state: PaymentState,
}

/// Proof that the Monad payment session has been submitted.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentProofRequest {
    pub session_id: String,
    pub agent_id: String,
    pub proof: PaymentProof,
}

impl PaymentProofRequest {
    pub fn monad_tx(session_id: String, agent_id: String, tx_hash: String) -> Self {
        Self {
            session_id,
            agent_id,
            proof: PaymentProof::MonadTx { tx_hash },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaymentProof {
    MonadTx { tx_hash: String },
}

/// Provider response after observing the payment proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceptedPayment {
    pub session_id: String,
    pub session_token: String,
    pub payment_state: PaymentState,
    pub tx_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub onchain_session_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaymentState {
    Proposed,
    Settled,
    Refunded,
}

/// Streaming event shape expected from `/v1/mpp-i/chat/completions`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MppiStreamEvent {
    #[serde(rename = "token.delta")]
    TokenDelta {
        delta: String,
        cumulative_output_tokens: u64,
    },
    #[serde(rename = "stream.done")]
    StreamDone { settlement: Settlement },
    #[serde(rename = "error")]
    Error { code: String, message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settlement {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub amount_due_wei: String,
    pub refund_wei: String,
    pub tx_hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MppiError {
    #[error("invalid provider URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("provider returned HTTP {status}: {body}")]
    Provider { status: u16, body: String },
}

async fn parse_json_response<T: for<'de> Deserialize<'de>>(
    response: Response,
) -> Result<T, MppiError> {
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(MppiError::Provider {
            status: response.status().as_u16(),
            body: response.text().await.unwrap_or_default(),
        })
    }
}

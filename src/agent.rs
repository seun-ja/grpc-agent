use std::{net::SocketAddr, sync::Arc};

use futures::{StreamExt as _, future};
use schemars::JsonSchema;
use tarpc::{
    server::{self, Channel as _, incoming::Incoming as _},
    tokio_serde::formats::Json,
};

use crate::{
    Error,
    providers::{CompletionProvider, Providers},
};

#[derive(Clone)]
pub struct Agent {
    socket_addr: SocketAddr,
    pub providers: Arc<Box<dyn CompletionProvider>>,
}

impl Agent {
    pub fn new(
        port: u16,
        provider: Providers,
        system_message: &str,
        model: &str,
        api_key: Option<&str>,
        temperature: Option<f64>,
        max_tokens: Option<u64>,
    ) -> Result<Self, Error> {
        let providers = Providers::init(
            provider,
            model,
            api_key,
            system_message.to_string(),
            temperature,
            max_tokens,
        )?;

        Ok(Self {
            socket_addr: SocketAddr::from(([0, 0, 0, 0], port)),
            providers: Arc::new(providers),
        })
    }

    pub fn new_with_schema<T: JsonSchema>(
        port: u16,
        provider: Providers,
        system_message: &str,
        model: &str,
        api_key: Option<&str>,
        temperature: Option<f64>,
        max_tokens: Option<u64>,
    ) -> Result<Self, Error> {
        let providers = Providers::init_with_schema::<T>(
            provider,
            model,
            api_key,
            system_message.to_string(),
            temperature,
            max_tokens,
        )?;

        Ok(Self {
            socket_addr: SocketAddr::from(([0, 0, 0, 0], port)),
            providers: Arc::new(providers),
        })
    }

    /// Runs the agent server.
    pub async fn run(&self) -> Result<(), Error> {
        let mut listener =
            tarpc::serde_transport::tcp::listen(self.socket_addr, Json::default).await?;
        listener.config_mut().max_frame_length(usize::MAX);

        println!("Listening on: {}", listener.local_addr());

        listener
            // Ignore accept errors.
            .filter_map(|r| future::ready(r.ok()))
            .map(server::BaseChannel::with_defaults)
            // Limit channels to 1 per IP.
            .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
            .map(|channel| {
                let server = Agent::new(
                    channel.transport().peer_addr().unwrap().port(),
                    self.providers.provider(),
                    self.providers.system_message().unwrap_or_default(),
                    self.providers.model(),
                    self.providers.api_key(),
                    self.providers.temperature(),
                    self.providers.max_tokens(),
                )
                .unwrap();

                channel.execute(server.serve()).for_each(|f| async {
                    tokio::spawn(f);
                })
            })
            // Max 10 channels.
            .buffer_unordered(10)
            .for_each(|_| async {})
            .await;

        Ok(())
    }
}

#[tarpc::service]
pub trait AgentWorker {
    // TODO: Improve error handling. The error type must implement serde::Deserialize.
    async fn message(user_message: String) -> String;
}

impl AgentWorker for Agent {
    /// Handles a user message by passing it to the completion provider and returning the response.
    async fn message(self, _context: ::tarpc::context::Context, user_message: String) -> String {
        println!("Message received");
        self.providers.chat(&user_message).await.unwrap()
    }
}

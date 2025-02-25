pub mod error;
pub mod events;
pub mod exports;

use std::{
    io::{Cursor, Read},
    sync::Arc,
    time::Duration,
};

use chrono::Utc;
use futures_util::{stream::StreamExt, SinkExt};
use tokio::{net::TcpStream, sync::Mutex, time::Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;
use url::Url;
use zstd::dict::DecoderDictionary;

use crate::{
    error::{ConfigValidationError, ConnectionError, JetstreamEventError},
    events::JetstreamEvent,
};

/// The Jetstream endpoints officially provided by Bluesky themselves.
///
/// There are no guarantees that these endpoints will always be available, but you are free
/// to run your own Jetstream instance in any case.
pub enum DefaultJetstreamEndpoints {
    /// `jetstream1.us-east.bsky.network`
    USEastOne,
    /// `jetstream2.us-east.bsky.network`
    USEastTwo,
    /// `jetstream1.us-west.bsky.network`
    USWestOne,
    /// `jetstream2.us-west.bsky.network`
    USWestTwo,
}

impl From<DefaultJetstreamEndpoints> for String {
    fn from(endpoint: DefaultJetstreamEndpoints) -> Self {
        match endpoint {
            DefaultJetstreamEndpoints::USEastOne => {
                "wss://jetstream1.us-east.bsky.network/subscribe".to_owned()
            }
            DefaultJetstreamEndpoints::USEastTwo => {
                "wss://jetstream2.us-east.bsky.network/subscribe".to_owned()
            }
            DefaultJetstreamEndpoints::USWestOne => {
                "wss://jetstream1.us-west.bsky.network/subscribe".to_owned()
            }
            DefaultJetstreamEndpoints::USWestTwo => {
                "wss://jetstream2.us-west.bsky.network/subscribe".to_owned()
            }
        }
    }
}

/// The maximum number of wanted collections that can be requested on a single Jetstream connection.
const MAX_WANTED_COLLECTIONS: usize = 100;
/// The maximum number of wanted DIDs that can be requested on a single Jetstream connection.
const MAX_WANTED_DIDS: usize = 10_000;

/// The custom `zstd` dictionary used for decoding compressed Jetstream messages.
///
/// Sourced from the [official Bluesky Jetstream repo.](https://github.com/bluesky-social/jetstream/tree/main/pkg/models)
const JETSTREAM_ZSTD_DICTIONARY: &[u8] = include_bytes!("../zstd/dictionary");

/// A receiver channel for consuming Jetstream events.
pub type JetstreamReceiver = flume::Receiver<JetstreamEvent>;

/// An internal sender channel for sending Jetstream events to [JetstreamReceiver]'s.
type JetstreamSender = flume::Sender<JetstreamEvent>;

/// A wrapper connector type for working with a WebSocket connection to a Jetstream instance to
/// receive and consume events. See [JetstreamConnector::connect] for more info.
pub struct JetstreamConnector {
    /// The configuration for the Jetstream connection.
    config: JetstreamConfig,
}

pub enum JetstreamCompression {
    /// No compression, just raw plaintext JSON.
    None,
    /// Use the `zstd` compression algorithm, which can result in a ~56% smaller messages on
    /// average. See [here](https://github.com/bluesky-social/jetstream?tab=readme-ov-file#compression) for more info.
    Zstd,
}

impl From<JetstreamCompression> for bool {
    fn from(compression: JetstreamCompression) -> Self {
        match compression {
            JetstreamCompression::None => false,
            JetstreamCompression::Zstd => true,
        }
    }
}

pub struct JetstreamConfig {
    /// A Jetstream endpoint to connect to with a WebSocket Scheme i.e.
    /// `wss://jetstream1.us-east.bsky.network/subscribe`.
    pub endpoint: String,
    /// A list of collection [NSIDs](https://atproto.com/specs/nsid) to filter events for.
    ///
    /// An empty list will receive events for *all* collections.
    ///
    /// Regardless of desired collections, all subscribers receive
    /// [AccountEvent](events::account::AccountEvent) and
    /// [IdentityEvent](events::identity::Identity) events.
    pub wanted_collections: Vec<exports::Nsid>,
    /// A list of repo [DIDs](https://atproto.com/specs/did) to filter events for.
    ///
    /// An empty list will receive events for *all* repos, which is a lot of events!
    pub wanted_dids: Vec<exports::Did>,
    /// The compression algorithm to request and use for the WebSocket connection (if any).
    pub compression: JetstreamCompression,
    /// An optional timestamp to begin playback from.
    ///
    /// An absent cursor or a cursor from the future will result in live-tail operation.
    ///
    /// When reconnecting, use the time_us from your most recently processed event and maybe
    /// provide a negative buffer (i.e. subtract a few seconds) to ensure gapless playback.
    pub cursor: Option<chrono::DateTime<Utc>>,

    // Maximum number of connection retries before giving up
    pub max_retries: u32,

    // Maximum number of milliseconds to wait between connection retries
    pub max_delay_ms: u64,

    // The base time unit between connection attempts, in milliseconds.
    pub base_delay_ms: u64,

    // Minimum time the connection must remain alive for the retries count to be reset.
    pub reset_retries_min_ms: u64,
}

impl Default for JetstreamConfig {
    fn default() -> Self {
        JetstreamConfig {
            endpoint: DefaultJetstreamEndpoints::USEastOne.into(),
            wanted_collections: Vec::new(),
            wanted_dids: Vec::new(),
            compression: JetstreamCompression::None,
            cursor: None,
            max_retries: 10,
            max_delay_ms: 30_000,
            base_delay_ms: 1_000,
            reset_retries_min_ms: 30_000
        }
    }
}

impl JetstreamConfig {
    /// Constructs a new endpoint URL with the given [JetstreamConfig] applied.
    pub fn construct_endpoint(&self, endpoint: &str) -> Result<Url, url::ParseError> {
        let did_search_query = self
            .wanted_dids
            .iter()
            .map(|s| ("wantedDids", s.to_string()));

        let collection_search_query = self
            .wanted_collections
            .iter()
            .map(|s| ("wantedCollections", s.to_string()));

        let compression = (
            "compress",
            match self.compression {
                JetstreamCompression::None => "false".to_owned(),
                JetstreamCompression::Zstd => "true".to_owned(),
            },
        );

        let cursor = self
            .cursor
            .map(|c| ("cursor", c.timestamp_micros().to_string()));

        let params = did_search_query
            .chain(collection_search_query)
            .chain(std::iter::once(compression))
            .chain(cursor)
            .collect::<Vec<(&str, String)>>();

        Url::parse_with_params(endpoint, params)
    }

    /// Validates the configuration to make sure it is within the limits of the Jetstream API.
    ///
    /// # Constants
    /// The following constants are used to validate the configuration and should only be changed
    /// if the Jetstream API has itself changed.
    /// - [MAX_WANTED_COLLECTIONS]
    /// - [MAX_WANTED_DIDS]
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        let collections = self.wanted_collections.len();
        let dids = self.wanted_dids.len();

        if collections > MAX_WANTED_COLLECTIONS {
            return Err(ConfigValidationError::TooManyWantedCollections(collections));
        }

        if dids > MAX_WANTED_DIDS {
            return Err(ConfigValidationError::TooManyDids(dids));
        }

        Ok(())
    }
}

impl JetstreamConnector {
    /// Create a Jetstream connector with a valid [JetstreamConfig].
    ///
    /// After creation, you can call [connect] to connect to the provided Jetstream instance.
    pub fn new(config: JetstreamConfig) -> Result<Self, ConfigValidationError> {
        // We validate the configuration here so any issues are caught early.
        config.validate()?;
        Ok(JetstreamConnector { config })
    }

    /// Connects to a Jetstream instance as defined in the [JetstreamConfig].
    ///
    /// A [JetstreamReceiver] is returned which can be used to respond to events. When all instances
    /// of this receiver are dropped, the connection and task are automatically closed.
    pub async fn connect(&self) -> Result<JetstreamReceiver, ConnectionError> {
        // We validate the config again for good measure. Probably not necessary but it can't hurt.
        self.config
            .validate()
            .map_err(ConnectionError::InvalidConfig)?;

        // TODO: Run some benchmarks and look into using a bounded channel instead.
        let (send_channel, receive_channel) = flume::unbounded();

        let configured_endpoint = self
            .config
            .construct_endpoint(&self.config.endpoint)
            .map_err(ConnectionError::InvalidEndpoint)?;

        let max_delay_ms = self.config.max_delay_ms;
        let base_delay_ms = self.config.base_delay_ms;
        let max_retries = self.config.max_retries;
        let min_duration_before_retry_reset = Duration::from_millis(self.config.reset_retries_min_ms);

        tokio::task::spawn(async move {
            
            let mut retry_attempt = 0;

            loop {
                let dict = DecoderDictionary::copy(JETSTREAM_ZSTD_DICTIONARY);

                if let Ok((ws_stream, _)) = connect_async(&configured_endpoint).await {
                    let now = Instant::now();
                    let _ = websocket_task(dict, ws_stream, send_channel.clone()).await;
                    let after_connection_closed = Instant::now();
                    if let Some(connection_alive_duration) = after_connection_closed.checked_duration_since(now) {
                        if connection_alive_duration > min_duration_before_retry_reset  {
                            retry_attempt = 0
                        }
                    }
                }

                retry_attempt += 1;
                
                if retry_attempt >= max_retries {
                    break;
                } 

                // Exponential backoff
                let delay_ms = base_delay_ms * (2_u64.pow(retry_attempt));
                log::error!("Connection failed, retrying in {delay_ms}ms...");
                tokio::time::sleep(Duration::from_millis(delay_ms.min(max_delay_ms))).await;                
                log::info!("Attempting to reconnect...");
            }
            log::error!("Connection retries exhausted. Jetstream is disconnected.");
        });

        Ok(receive_channel)
    }
}

/// The main task that handles the WebSocket connection and sends [JetstreamEvent]'s to any
/// receivers that are listening for them.
async fn websocket_task(
    dictionary: DecoderDictionary<'_>,
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    send_channel: JetstreamSender,
) -> Result<(), JetstreamEventError> {
    // TODO: Use the write half to allow the user to change configuration settings on the fly.
    let (socket_write, mut socket_read) = ws.split();
    let shared_socket_write = Arc::new(Mutex::new(socket_write));

    let ping_cancellation_token = CancellationToken::new();
    let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
    let ping_cancelled = ping_cancellation_token.clone();
    let ping_shared_socket_write = shared_socket_write.clone();
    tokio::spawn(async move {
        loop {
            ping_interval.tick().await;
            let false = ping_cancelled.is_cancelled() else {
                break;
            };
            log::trace!("Sending ping");
            match ping_shared_socket_write
                .lock()
                .await
                .send(Message::Ping("ping".as_bytes().to_vec()))
                .await
            {
                Ok(_) => (),
                Err(error) => {
                    log::error!("Ping failed: {error}");
                    break;
                }
            }
        }
    });

    let mut closing_connection = false;
    loop {
        match socket_read.next().await {
            Some(Ok(message)) => {
                match message {
                    Message::Text(json) => {
                        let event = serde_json::from_str::<JetstreamEvent>(&json)
                            .map_err(JetstreamEventError::ReceivedMalformedJSON)?;

                        if send_channel.send(event).is_err() {
                            // We can assume that all receivers have been dropped, so we can close the
                            // connection and exit the task.
                            log::info!(
                            "All receivers for the Jetstream connection have been dropped, closing connection."
                        );
                            closing_connection = true;
                        }
                    }
                    Message::Binary(zstd_json) => {
                        let mut cursor = Cursor::new(zstd_json);
                        let mut decoder = zstd::stream::Decoder::with_prepared_dictionary(
                            &mut cursor,
                            &dictionary,
                        )
                        .map_err(JetstreamEventError::CompressionDictionaryError)?;

                        let mut json = String::new();
                        decoder
                            .read_to_string(&mut json)
                            .map_err(JetstreamEventError::CompressionDecoderError)?;

                        let event = serde_json::from_str::<JetstreamEvent>(&json)
                            .map_err(JetstreamEventError::ReceivedMalformedJSON)?;

                        if send_channel.send(event).is_err() {
                            // We can assume that all receivers have been dropped, so we can close the
                            // connection and exit the task.
                            log::info!(
                            "All receivers for the Jetstream connection have been dropped, closing connection..."
                        );
                            closing_connection = true;
                        }
                    }
                    Message::Ping(vec) => {
                        log::trace!("Ping recieved, responding");
                        _ = shared_socket_write
                            .lock()
                            .await
                            .send(Message::Pong(vec))
                            .await;
                    }
                    Message::Close(close_frame) => {
                        if let Some(close_frame) = close_frame {
                            let reason = close_frame.reason;
                            let code = close_frame.code;
                            log::trace!("Connection closed. Reason: {reason}, Code: {code}");
                        }
                    }
                    Message::Pong(pong) => {
                        let pong_payload =
                            String::from_utf8(pong).unwrap_or("Invalid payload".to_string());
                        log::trace!("Pong recieved. Payload: {pong_payload}");
                    }
                    Message::Frame(_) => (),
                }
            }
            Some(Err(error)) => {
                log::error!("Web socket error: {error}");
                ping_cancellation_token.cancel();
                closing_connection = true;
            }
            None => {
                log::error!("No web socket result");
                ping_cancellation_token.cancel();
                closing_connection = true;
            }
        }
        if closing_connection {
            _ = shared_socket_write.lock().await.close().await;
            return Ok(());
        }
    }
}

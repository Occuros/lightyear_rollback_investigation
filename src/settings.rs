use crate::common::settings::{
    ClientSettings, ClientTransports, Conditioner, ServerSettings, ServerTransports, Settings, SharedSettings,
    WebTransportCertificateSettings,
};
use lightyear::prelude::CompressionConfig;
use std::env;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::string::ToString;

#[derive(Clone, Debug)]
pub struct MySettings {
    pub common: Settings,

    /// By how many ticks an input press will be delayed?
    /// This can be useful as a tradeoff between input delay and prediction accuracy.
    /// If the input delay is greater than the RTT, then there won't ever be any mispredictions/rollbacks.
    /// See [this article](https://www.snapnet.dev/docs/core-concepts/input-delay-vs-rollback/) for more information.
    pub(crate) input_delay_ticks: u16,

    /// If visual correction is enabled, we don't instantly snapback to the corrected position
    /// when we need to rollback. Instead we interpolated between the current position and the
    /// corrected position.
    /// This controls the duration of the interpolation; the higher it is, the longer the interpolation
    /// will take
    pub(crate) correction_ticks_factor: f32,
}

pub(crate) fn get_settings() -> MySettings {
    let cert_path = format!(
        "{}/certificates/cert.pem",
        env!("CARGO_MANIFEST_DIR")
    );
    let key_path = format!(
        "{}/certificates/key.pem",
        env!("CARGO_MANIFEST_DIR")
    );

    MySettings {
        common: Settings {
            server: ServerSettings {
                headless: false,
                inspector: true,
                conditioner: Some(
                    Conditioner {
                        latency_ms: 100,
                        jitter_ms: 20,
                        packet_loss: 0.01,
                    },

                ),
                // conditioner: None,
                transport: vec![
                    ServerTransports::WebTransport {
                        local_port: 5000,
                        certificate: WebTransportCertificateSettings::AutoSelfSigned(vec![
                            "example.com".to_string(),
                            "*.gameserver.example.org".to_string(),
                            "10.1.2.3".to_string(),
                            "::1".to_string(),
                        ]),
                        // certificate: WebTransportCertificateSettings::FromFile {
                        //     cert: cert_path,
                        //     key: key_path,
                        // },
                    },
                    ServerTransports::Udp {
                        local_port: 5001,
                    },
                    #[cfg(feature = "websocket")]
                    ServerTransports::WebSocket {
                        local_port: 5002,
                    },
                    #[cfg(feature = "steam")]
                    ServerTransports::Steam {
                        app_id: 480,
                        server_ip: Ipv4Addr::UNSPECIFIED,
                        game_port: 5003,
                        query_port: 27016,
                    },
                ],
            },
            client: ClientSettings {
                inspector: true,
                client_id: 0,
                client_port: 0, // 0 means that the OS will assign a random port
                server_addr: Ipv4Addr::LOCALHOST,
                server_port: 5000, // change the port depending on the transport used
                transport: ClientTransports::WebTransport {
                    #[cfg(target_family = "wasm")]
                    certificate_digest: include_str!("../../certificates/digest.txt").to_string(),
                },
                conditioner: None,
                // conditioner: Some(
                //     Conditioner {
                //         latency_ms: 100,
                //         jitter_ms: 20,
                //         packet_loss: 0.01,
                //     },
                //
                // ),
            },
            shared: SharedSettings {
                protocol_id: 0,
                private_key: [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                compression: CompressionConfig::None,
            },
        },
        input_delay_ticks: 2,
        correction_ticks_factor: 1.0,
    }
}

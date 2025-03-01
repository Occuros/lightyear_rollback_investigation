#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use std::time::Duration;

use bevy::prelude::*;
use common::shared::FIXED_TIMESTEP_HZ;
use lightyear::prelude::client::PredictionConfig;
use serde::{Deserialize, Serialize};
use shared::SharedPlugin;

#[cfg(feature = "client")]
mod client;
mod common;
mod protocol;
#[cfg(feature = "gui")]
mod renderer;
#[cfg(feature = "server")]
mod server;
mod settings;
mod shared;
mod quantization;

use crate::common::app::{Apps, Cli, Mode};
use crate::common::settings::Settings;

fn main() {
    let cli = Cli::default();
    let settings = settings::get_settings();
    let mut apps = Apps::new(
        settings.common,
        cli,
        env!("CARGO_PKG_NAME").to_string(),
    );

    apps.update_lightyear_client_config(
        |config| {
            config
                .prediction
                .set_fixed_input_delay_ticks(settings.input_delay_ticks);
            config.prediction.correction_ticks_factor = settings.correction_ticks_factor;
            // config.interpolation.send_interval_ratio = 0.1;
            // config.interpolation.min_delay = Duration::from_secs_f32(25.0);
        },
    );

    apps.add_lightyear_plugins();
    apps.add_user_shared_plugin(SharedPlugin);
    #[cfg(feature = "server")]
    apps.add_user_server_plugin(server::ExampleServerPlugin);
    #[cfg(feature = "client")]
    apps.add_user_client_plugin(client::ExampleClientPlugin);
    #[cfg(feature = "gui")]
    apps.add_user_renderer_plugin(renderer::ExampleRendererPlugin);

    apps.run();
}

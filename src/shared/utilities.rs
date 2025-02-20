use bevy::prelude::*;
use lightyear::prelude::ClientId;

/// Generate pseudo-random color based on `client_id`.
pub(crate) fn color_from_id(client_id: ClientId) -> Color {
    let h = (((client_id.to_bits().wrapping_mul(30)) % 360) as f32) / 360.0;
    let s = 1.0;
    let l = 0.5;
    Color::hsl(
        h, s, l,
    )
}

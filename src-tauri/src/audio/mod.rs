pub mod devices;
pub mod playback;

pub use devices::get_out_devices;
pub use playback::{get_sound_duration, init_audio_thread, play_sound};

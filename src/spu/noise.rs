use super::sampler::Sampler;
use sdl2::audio::AudioQueue;

//         Noise
// FF1F ---- ---- Not used
// NR41 FF20 --LL LLLL Length load (64-L)
// NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
// NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
// NR44 FF23 TL-- ---- Trigger, Length enable
#[allow(dead_code)]
#[derive(Default)]
pub struct Noise {
    pub left_enable: bool,
    pub right_enable: bool,

    playing: bool,
    repeat: bool,

    envelope_add_mode: bool,
    envelope_start_volume: u8,
    envelope_sweep_number: u8,

    clock_shift: u8,
    clock_width_mode: u8,
    clock_divisor_code: u8,
}

impl Sampler for Noise {
    fn enqueue_audio_samples(&mut self, _queue: &mut AudioQueue<i8>) {

    }
}
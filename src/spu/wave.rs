use super::sampler::Sampler;
use sdl2::audio::AudioQueue;

//         Wave
// NR30 FF1A E--- ---- DAC power
// NR31 FF1B LLLL LLLL Length load (256-L)
// NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
// NR33 FF1D FFFF FFFF Frequency LSB
// NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
#[allow(dead_code)]
#[derive(Default)]
pub struct Wave {
    pub left_enable: bool,
    pub right_enable: bool,

    playing: bool,
    restart: bool,
    repeat: bool,
    frequency: u16,

    wave_length_load: u8,
    wave_volume: u8,
    pub wave_samples: [i8; 32],
}

impl Sampler for Wave {
    fn enqueue_audio_samples(&mut self, _queue: &mut AudioQueue<i8>) {

    }
}
pub mod flags;
pub mod noise;
pub mod sampler;
pub mod square;
pub mod util;
pub mod wave;

use flags::*;
use sampler::Sampler;
use square::Square;
use noise::Noise;
use wave::Wave;
use sdl2::audio::AudioQueue;

use crate::MemoryBus;

#[allow(dead_code)]
#[derive(Default)]
pub struct Spu {
    enabled: bool,

    // SO2
    left_volume: u8,

    // SO1
    right_volume: u8,

    // TONE & SWEEP
    channel1: Square,

    // TONE
    channel2: Square,

    // WAVE
    channel3: Wave,

    // NOISE
    channel4: Noise,
}

impl Spu {
    pub fn enqueue_audio_samples(&mut self,
            channel1: &mut AudioQueue<i8>,
            channel2: &mut AudioQueue<i8>,
            channel3: &mut AudioQueue<i8>,
            channel4: &mut AudioQueue<i8>) {
        self.channel1.enqueue_audio_samples(channel1);
        self.channel2.enqueue_audio_samples(channel2);
        self.channel3.enqueue_audio_samples(channel3);
        self.channel4.enqueue_audio_samples(channel4);
    }
}

const NR10: u16 = 0xFF10;
const NR11: u16 = 0xFF11;
const NR12: u16 = 0xFF12;
const NR13: u16 = 0xFF13;
const NR14: u16 = 0xFF14;

const NR21: u16 = 0xFF16;
const NR22: u16 = 0xFF17;
const NR23: u16 = 0xFF18;
const NR24: u16 = 0xFF19;

/*
const NR30: u16 = 0xFF1A;
const NR31: u16 = 0xFF1B;
const NR32: u16 = 0xFF1C;
const NR33: u16 = 0xFF1D;
const NR34: u16 = 0xFF1E;

const NR41: u16 = 0xFF20;
const NR42: u16 = 0xFF21;
const NR43: u16 = 0xFF22;
const NR44: u16 = 0xFF23;
*/

const NR50: u16 = 0xFF24;
const NR51: u16 = 0xFF25;
const NR52: u16 = 0xFF26;

impl MemoryBus for Spu {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            _ => 0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            NR10 => self.channel1.set_r0_for_channel1(Channel1SweepControl::from_bits_truncate(data)),
            NR11 => self.channel1.set_r1_for_channel1(Channel1SequenceControl::from_bits_truncate(data)),
            NR12 => self.channel1.set_r2_for_channel1(Channel1EnvelopeControl::from_bits_truncate(data)),
            NR13 => self.channel1.set_r3_for_channel1(data),
            NR14 => self.channel1.set_r4_for_channel1(Channel1FrequencyHigherData::from_bits_truncate(data)),

            NR21 => self.channel2.set_r1_for_channel2(Channel2SequenceControl::from_bits_truncate(data)),
            NR22 => self.channel2.set_r2_for_channel2(Channel2EnvelopeControl::from_bits_truncate(data)),
            NR23 => self.channel2.set_r3_for_channel2(data),
            NR24 => self.channel2.set_r4_for_channel2(Channel2FrequencyHigherData::from_bits_truncate(data)),

            NR50 => {
                let r = MasterVolumeControl::from_bits(data).unwrap();
                self.left_volume = r.left_volume();
                self.right_volume = r.right_volume();
            }
            NR51 => {
                let r = MasterOutputControl::from_bits(data).unwrap();

                self.channel4.left_enable = r.left_channel4_enable();
                self.channel3.left_enable = r.left_channel3_enable();
                self.channel2.left_enable = r.left_channel2_enable();
                self.channel1.left_enable = r.left_channel1_enable();

                self.channel4.right_enable = r.right_channel4_enable();
                self.channel3.right_enable = r.right_channel3_enable();
                self.channel2.right_enable = r.right_channel2_enable();
                self.channel1.right_enable = r.right_channel1_enable();
            }
            NR52 => {
                let r = MasterOnOffControl::from_bits(data).unwrap();
                self.enabled = r.all_channels_enable();
            }
            _ => { }
        }
    }
}
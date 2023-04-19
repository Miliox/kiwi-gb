use super::flags::*;
use super::util::*;
use super::sampler::*;
use sdl2::audio::AudioQueue;

use packed_struct::prelude::*;

/*
    Square 1
    NR10 FF10 -PPP NSSS Sweep period, negate, shift
    NR11 FF11 DDLL LLLL Duty, Length load (64-L)
    NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
    NR13 FF13 FFFF FFFF Frequency LSB
    NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB

        Square 2
        FF15 ---- ---- Not used
    NR21 FF16 DDLL LLLL Duty, Length load (64-L)
    NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
    NR23 FF18 FFFF FFFF Frequency LSB
    NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB
*/
#[allow(dead_code)]
pub struct Square {
    pub left_enable: bool,
    pub right_enable: bool,

    playing: bool,
    restart: bool,
    repeat: bool,

    frequency: u32,
    fparam: u32,

    envelope_direction: bool,
    envelope_start_volume: u8,
    envelope_sweep_number: u8,

    sweep_inverse: bool,
    sweep_period: u8,
    sweep_shift: u8,

    wave_duty: u8,
    wave_length: u8,

    buffer: Box<[i8; 8192]>,
    phase_duty: f32,
    phase_pos: f32,
    step_counter: f32,
    volume_step: u8,
    volume: i8,
}

impl Default for Square {
    fn default() -> Self {
        Self {
            left_enable: false,
            right_enable: false,

            playing: false,
            restart: false,
            repeat: false,
            frequency: calculate_frequency(0),
            fparam: 0,

            envelope_start_volume: 0,
            envelope_sweep_number: 0,
            envelope_direction: true,

            sweep_inverse: false,
            sweep_period: 0,
            sweep_shift: 0,
            wave_duty: 0,
            wave_length: 0,

            buffer: Box::new([0; 8192]),
            phase_duty: 0.5,
            phase_pos: 0.0,
            step_counter: 0.0,
            volume_step: 0,
            volume: 0,
        }
    }
}

impl Sampler for Square {
    fn enqueue_audio_samples(&mut self, queue: &mut AudioQueue<i8>) {
        if self.restart {
            self.restart = false;
            self.playing = true;
            self.phase_pos = 0.0;
            queue.clear();
        }

        if !self.playing {
            return;
        }

        if !self.left_enable && !self.right_enable && self.wave_length != 0 {
            return;
        }

        let phase_inc = self.frequency as f32 / queue.spec().freq as f32;
        let step_size = self.envelope_sweep_number as f32 * (queue.spec().freq as f32 / 64.0);

        let length = self.buffer.len();
        if (queue.size() as usize) < length {
            let length = length / 2;
            for i in 0..length {
                // envelope
                if step_size > 0.0 {
                    if self.step_counter >= step_size {
                        self.step_counter -= step_size;
                        if self.envelope_direction {
                            if self.volume_step < 0xF {
                                self.volume_step += 1;
                            }
                        } else {
                            if self.volume_step >= 0x1 {
                                self.volume_step -= 1;
                            }
                        }
                        self.volume = calculate_volume(self.volume_step);
                        if self.volume_step == 0 {
                            self.playing = false;
                        }
                    }
                    self.step_counter += 1.0;
                }

                // Duty   Waveform    Ratio
                // -------------------------
                // 0      00000001    12.5%
                // 1      10000001    25%
                // 2      10000111    50%
                // 3      01111110    75%
                let sample = self.volume * match self.wave_duty {
                    1 => if self.phase_pos >= 0.875 { 1 } else { -1 },
                    2 => if self.phase_pos <= 0.125 || self.phase_pos >= 0.875 { 1 } else { -1 },
                    3 => if self.phase_pos <= 0.125 || self.phase_pos >= 0.625 { 1 } else { -1 },
                    4 => if self.phase_pos >= 0.125 && self.phase_pos <= 0.875 { 1 } else { -1 },
                    _ => -1,
                };

                // left
                self.buffer[i * 2] = if self.left_enable { sample } else { 0 };

                // right
                self.buffer[i * 2 + 1] = if self.right_enable { sample } else { 0 };

                self.phase_pos += phase_inc;
                self.phase_pos %= 1.0;
            }
            queue.queue(&*self.buffer);
        }
    }
}

#[allow(dead_code)]
impl Square {
    pub fn r0(&self) -> u8 {
        let mut sc = SweepControl::default();
        sc.sweep_inverse = self.sweep_inverse;
        sc.sweep_period  = self.sweep_period;
        sc.sweep_shift   = self.sweep_shift;
        sc.pack().unwrap()[0]
    }

    pub fn set_r0(&mut self, r: u8) {
        let r: [u8; 1] = [r];
        let r = SweepControl::unpack(&r).unwrap();
        //println!("{:?}", r);

        self.sweep_inverse = r.sweep_inverse;
        self.sweep_period = r.sweep_period;
        self.sweep_shift = r.sweep_shift;
    }

    pub fn r1(&self) -> u8 {
        let mut sc = SequenceControl::default();
        sc.duty         = self.wave_duty;
        sc.data_length  = self.wave_length;
        sc.pack().unwrap()[0]
    }

    pub fn set_r1(&mut self, r: u8) {
        let r: [u8; 1] = [r];
        let r = SequenceControl::unpack(&r).unwrap();
        //println!("{:?}", r);

        self.wave_duty = r.duty;
        self.wave_length = r.data_length;
        self.phase_duty = r.phase_duty();
    }

    pub fn r2(&self) -> u8 {
        let mut ec = EnvelopeControl::default();
        ec.initial_volume = self.envelope_start_volume;
        ec.envelope_direction = self.envelope_direction;
        ec.envelope_step = self.envelope_sweep_number;
        ec.pack().unwrap()[0]
    }

    pub fn set_r2(&mut self, r: u8) {
        let r: [u8; 1] = [r];
        let r = EnvelopeControl::unpack(&r).unwrap();
        //println!("{:?}", r);

        self.envelope_start_volume = r.initial_volume;
        self.envelope_direction = r.envelope_direction;
        self.envelope_sweep_number = r.envelope_step;
        self.volume_step = self.envelope_start_volume;
        self.volume = calculate_volume(self.volume_step);
    }

    pub fn r3(&self) -> u8 {
        (self.fparam & 0xFF) as u8
    }

    pub fn set_r3(&mut self, data: u8) {
        self.fparam = set_low_frequency_param(self.fparam, data as u32);
        self.frequency = calculate_frequency(self.fparam);
    }

    pub fn r4(&self) -> u8 {
        ((self.fparam & 0x0700) >> 8) as u8
    }

    pub fn set_r4(&mut self, r: u8) {
        let r: [u8; 1] = [r];
        let r = FrequencyHigherData::unpack(&r).unwrap();
        //println!("{:?}", r);

        self.fparam = set_high_frequency_param(self.fparam, r.frequency_higher as u32);
        self.frequency = calculate_frequency(self.fparam);
        self.repeat = !r.stop_on_complete;
        self.restart = r.restart_sequence;
    }
}
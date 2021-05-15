use super::flags::*;
use super::util::*;
use super::sampler::*;
use sdl2::audio::AudioQueue;

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

        if !self.left_enable && !self.right_enable {
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
                let sample = match self.wave_duty {
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
        0
    }

    pub fn set_r0_for_channel1(&mut self, r: Channel1SweepControl) {
        self.sweep_inverse = r.sweep_inverse();
        self.sweep_period = r.sweep_period();
        self.sweep_shift = r.sweep_shift();

        trace!("NR10 sweep_inv={} sweep_period={} sweep_shift={}",
            self.sweep_inverse, self.sweep_period, self.sweep_shift);
    }

    pub fn r1(&self) -> u8 {
        0
    }

    pub fn set_r1_for_channel1(&mut self, r: Channel1SequenceControl) {
        self.wave_duty = r.sequence_duty();
        self.wave_length = r.sequence_length();
        self.phase_duty = calculate_phase_duty(self.wave_duty);

        trace!("NR11 wave_duty={} wave_len={} phase_duty={}",
            self.wave_duty, self.wave_length, self.phase_duty);
    }

    pub fn set_r1_for_channel2(&mut self, r: Channel2SequenceControl) {
        self.wave_duty = r.sequence_duty();
        self.wave_length = r.sequence_length();
        self.phase_duty = calculate_phase_duty(self.wave_duty);

        trace!("NR21 wave__duty={} wave_len={} phase_duty={}",
            self.wave_duty, self.wave_length, self.phase_duty);
    }

    pub fn r2(&self) -> u8 {
        0
    }

    pub fn set_r2_for_channel1(&mut self, r: Channel1EnvelopeControl) {
        self.envelope_start_volume = r.envelope_initial_volume();
        self.envelope_direction = r.is_envelope_increase_direction();
        self.envelope_sweep_number = r.envelope_sweep_number();
        self.volume_step = self.envelope_start_volume;
        self.volume = calculate_volume(self.volume_step);

        trace!("NR12 env_start_vol={} env_dir={} env_sweep_num={} vol={}",
            self.envelope_start_volume, self.envelope_direction, self.envelope_sweep_number,  self.volume);
    }

    pub fn set_r2_for_channel2(&mut self, r: Channel2EnvelopeControl) {
        self.envelope_start_volume = r.envelope_initial_volume();
        self.envelope_direction = r.is_envelope_increase_direction();
        self.envelope_sweep_number = r.envelope_sweep_number();
        self.volume = calculate_volume(self.envelope_start_volume);

        trace!("NR22 env_start_vol={} env_dir={} env_num={} vol={}",
            self.envelope_start_volume, self.envelope_direction, self.envelope_sweep_number, self.volume);
    }

    pub fn r3(&self) -> u8 {
        0
    }

    pub fn set_r3_for_channel1(&mut self, data: u8) {
        self.fparam = set_low_frequency_param(self.fparam, data as u32);
        self.frequency = calculate_frequency(self.fparam);

        trace!("NR13 fparam={} freq={}", self.fparam, self.frequency);
    }

    pub fn set_r3_for_channel2(&mut self, data: u8) {
        self.fparam = set_low_frequency_param(self.fparam, data as u32);
        self.frequency = calculate_frequency(self.fparam);

        trace!("NR23 fparam={} freq={}", self.fparam, self.frequency);
    }

    pub fn r4(&self) -> u8 {
        0
    }

    pub fn set_r4_for_channel1(&mut self, r: Channel1FrequencyHigherData) {
        self.fparam = set_high_frequency_param(self.fparam, r.frequency_higher_part());
        self.frequency = calculate_frequency(self.fparam);
        self.repeat = r.is_sequence_to_repeat_when_complete();
        self.restart = r.is_sequence_to_restart();

        trace!("NR14 fparam={} freq={} repeat={} restart={}",
            self.fparam, self.frequency, self.repeat, self.restart);
    }

    pub fn set_r4_for_channel2(&mut self, r: Channel2FrequencyHigherData) {
        self.fparam = set_high_frequency_param(self.fparam, r.frequency_higher_part());
        self.repeat = r.is_sequence_to_repeat_when_complete();
        self.restart = r.is_sequence_to_restart();

        trace!("NR24 ch2_fparam={} ch2_freq={} ch2_repeat={} ch2_restart={}",
            self.fparam,
            self.frequency,
            self.repeat,
            self.restart);
    }
}
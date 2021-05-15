bitflags! {
    /// FF10 - NR10 - Channel 1 Sweep register (R/W)
    /// - Bit 6-4 - Sweep Time
    /// - Bit 3   - Sweep Increase/Decrease
    ///            0: Addition    (frequency increases)
    ///            1: Subtraction (frequency decreases)
    /// - Bit 2-0 - Number of sweep shift (n: 0-7)
    ///
    /// Sweep Time:
    ///
    ///     000: sweep off - no freq change
    ///     001: 7.8 ms  (1/128Hz)
    ///     010: 15.6 ms (2/128Hz)
    ///     011: 23.4 ms (3/128Hz)
    ///     100: 31.3 ms (4/128Hz)
    ///     101: 39.1 ms (5/128Hz)
    ///     110: 46.9 ms (6/128Hz)
    ///     111: 54.7 ms (7/128Hz)
    ///
    /// The change of frequency (NR13,NR14) at each shift is calculated by the following formula where X(0) is initial freq & X(t-1) is last freq:
    ///
    /// X(t) = X(t-1) +/- X(t-1)/2^n
    pub struct Channel1SweepControl: u8 {
        const UNUSED_BIT7             = 0b1000_0000;
        const SWEEP_PERIOD_MASK       = 0b0111_0000;
        const SWEEP_SHIFT_MASK        = 0b0000_0111;

        const SWEEP_PERIOD_BIT2       = 0b0100_0000;
        const SWEEP_PERIOD_BIT1       = 0b0010_0000;
        const SWEEP_PERIOD_BIT0       = 0b0001_0000;

        const SWEEP_DIRECTION_SELECT  = 0b0000_1000;

        const SWEEP_SHIFT_BIT2        = 0b0000_0100;
        const SWEEP_SHIFT_BIT1        = 0b0000_0010;
        const SWEEP_SHIFT_BIT0        = 0b0000_0001;
    }
}

bitflags! {
    /// FF11 - NR11 - Channel 1 Sound length/Wave pattern duty (R/W)
    /// - Bit 7-6 - Wave Pattern Duty (Read/Write)
    /// - Bit 5-0 - Sound length data (Write Only) (t1: 0-63)
    ///
    /// Wave Duty:
    ///
    ///     00: 12.5% ( _-------_-------_------- )
    ///     01: 25%   ( __------__------__------ )
    ///     10: 50%   ( ____----____----____---- ) (normal)
    ///     11: 75%   ( ______--______--______-- )
    ///
    /// Sound Length = (64-t1)*(1/256) seconds.
    ///
    /// The Length value is used only if Bit 6 in NR14 is set.
    pub struct Channel1SequenceControl: u8 {
        const SOUND_SEQUENCE_DUTY_MASK   = 0b1100_0000;
        const SOUND_SEQUENCE_LENGTH_MASK = 0b0011_1111;

        const SOUND_SEQUENCE_DUTY_BIT1   = 0b1000_0000;
        const SOUND_SEQUENCE_DUTY_BIT0   = 0b0100_0000;

        const SOUND_SEQUENCE_LENGTH_BIT5 = 0b0010_0000;
        const SOUND_SEQUENCE_LENGTH_BIT4 = 0b0001_0000;
        const SOUND_SEQUENCE_LENGTH_BIT3 = 0b0000_1000;
        const SOUND_SEQUENCE_LENGTH_BIT2 = 0b0000_0100;
        const SOUND_SEQUENCE_LENGTH_BIT1 = 0b0000_0010;
        const SOUND_SEQUENCE_LENGTH_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF12 - NR12 - Channel 1 Volume Envelope (R/W)
    /// - Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
    /// - Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
    /// - Bit 2-0 - Number of envelope sweep (n: 0-7)
    ///             (If zero, stop envelope operation.)
    ///
    /// Length of 1 step = n*(1/64) seconds
    pub struct Channel1EnvelopeControl: u8 {
        const ENVELOPE_INITIAL_VOLUME_MASK = 0b1111_0000;
        const ENVELOPE_SWEEP_NUMBER_MASK   = 0b0000_0111;

        const ENVELOPE_INITIAL_VOLUME_BIT3 = 0b1000_0000;
        const ENVELOPE_INITIAL_VOLUME_BIT2 = 0b0100_0000;
        const ENVELOPE_INITIAL_VOLUME_BIT1 = 0b0010_0000;
        const ENVELOPE_INITIAL_VOLUME_BIT0 = 0b0001_0000;

        const ENVELOPE_DIRECTION_SELECT    = 0b0000_1000;

        const ENVELOPE_SWEEP_NUMBER_BIT2   = 0b0000_0100;
        const ENVELOPE_SWEEP_NUMBER_BIT1   = 0b0000_0010;
        const ENVELOPE_SWEEP_NUMBER_BIT0   = 0b0000_0001;
    }
}

bitflags! {
    /// FF13 - NR13 - Channel 1 Frequency low (Write Only)
    /// - Bit 7-0 - Lower 8 bits of 11 bit frequency (x).
    ///
    /// Next 3 bit are in NR14 ($FF14)
    pub struct Channel1FrequencyLowerData: u8 {
        const FREQUENCY_LOWER_MASK = 0b1111_1111;

        const FREQUENCY_BIT7 = 0b1000_0000;
        const FREQUENCY_BIT6 = 0b0100_0000;
        const FREQUENCY_BIT5 = 0b0010_0000;
        const FREQUENCY_BIT4 = 0b0001_0000;
        const FREQUENCY_BIT3 = 0b0000_1000;
        const FREQUENCY_BIT2 = 0b0000_0100;
        const FREQUENCY_BIT1 = 0b0000_0010;
        const FREQUENCY_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF14 - NR14 - Channel 1 Frequency high (R/W)
    /// - Bit 7   - Initial (1=Restart Sound)     (Write Only)
    /// - Bit 6   - Counter/consecutive selection (Read/Write)
    ///           (1=Stop output when length in NR11 expires)
    /// - Bit 2-0 - Frequency's higher 3 bits (x) (Write Only)
    ///
    /// Frequency = 131072/(2048-x) Hz
    pub struct Channel1FrequencyHigherData: u8 {
        const RESTART_SEQUENCE              = 0b1000_0000;
        const STOP_ON_SEQUENCE_COMPLETE     = 0b0100_0000;

        const UNUSED_BIT5 = 0b0010_0000;
        const UNUSED_BIT4 = 0b0001_0000;
        const UNUSED_BIT3 = 0b0000_1000;

        const FREQUENCY_HIGHER_MASK = 0b0000_0111;

        const FREQUENCY_BIT10 = 0b0000_0100;
        const FREQUENCY_BIT9  = 0b0000_0010;
        const FREQUENCY_BIT8  = 0b0000_0001;
    }
}

bitflags! {
    /// FF16 - NR21 - Channel 2 Sound Length/Wave Pattern Duty (R/W)
    /// - Bit 7-6 - Wave Pattern Duty (Read/Write)
    /// - Bit 5-0 - Sound length data (Write Only) (t1: 0-63)
    ///
    /// Wave Duty:
    ///
    ///     00: 12.5% ( _-------_-------_------- )
    ///     01: 25%   ( __------__------__------ )
    ///     10: 50%   ( ____----____----____---- ) (normal)
    ///     11: 75%   ( ______--______--______-- )
    ///
    /// Sound Length = (64-t1)*(1/256) seconds.
    ///
    /// The Length value is used only if Bit 6 in NR24 is set.
    pub struct Channel2SequenceControl: u8 {
        const SOUND_SEQUENCE_DUTY_MASK   = 0b1100_0000;
        const SOUND_SEQUENCE_LENGTH_MASK = 0b0011_1111;

        const SOUND_SEQUENCE_DUTY_BIT1   = 0b1000_0000;
        const SOUND_SEQUENCE_DUTY_BIT0   = 0b0100_0000;

        const SOUND_SEQUENCE_LENGTH_BIT5 = 0b0010_0000;
        const SOUND_SEQUENCE_LENGTH_BIT4 = 0b0001_0000;
        const SOUND_SEQUENCE_LENGTH_BIT3 = 0b0000_1000;
        const SOUND_SEQUENCE_LENGTH_BIT2 = 0b0000_0100;
        const SOUND_SEQUENCE_LENGTH_BIT1 = 0b0000_0010;
        const SOUND_SEQUENCE_LENGTH_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF17 - NR22 - Channel 2 Volume Envelope (R/W)
    /// - Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
    /// - Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
    /// - Bit 2-0 - Number of envelope sweep (n: 0-7)
    ///              (If zero, stop envelope operation.)
    ///
    /// Length of 1 step = n*(1/64) seconds
    pub struct Channel2EnvelopeControl: u8 {
        const ENVELOPE_INITIAL_VOLUME_MASK = 0b1111_0000;
        const ENVELOPE_SWEEP_NUMBER_MASK   = 0b0000_0111;

        const ENVELOPE_INITIAL_VOLUME_BIT3 = 0b1000_0000;
        const ENVELOPE_INITIAL_VOLUME_BIT2 = 0b0100_0000;
        const ENVELOPE_INITIAL_VOLUME_BIT1 = 0b0010_0000;
        const ENVELOPE_INITIAL_VOLUME_BIT0 = 0b0001_0000;

        const ENVELOPE_DIRECTION_SELECT    = 0b0000_1000;

        const ENVELOPE_SWEEP_NUMBER_BIT2   = 0b0000_0100;
        const ENVELOPE_SWEEP_NUMBER_BIT1   = 0b0000_0010;
        const ENVELOPE_SWEEP_NUMBER_BIT0   = 0b0000_0001;
    }
}

bitflags! {
    /// FF18 - NR23 - Channel 2 Frequency low data (W)
    /// - Bit 7-0 - Frequency's lower 8 bits of 11 bit data (x).
    ///
    /// Next 3 bits are in NR24 ($FF19).
    pub struct Channel2FrequencyLowerData: u8 {
        const FREQUENCY_LOWER_MASK = 0b1111_1111;

        const FREQUENCY_BIT7 = 0b1000_0000;
        const FREQUENCY_BIT6 = 0b0100_0000;
        const FREQUENCY_BIT5 = 0b0010_0000;
        const FREQUENCY_BIT4 = 0b0001_0000;
        const FREQUENCY_BIT3 = 0b0000_1000;
        const FREQUENCY_BIT2 = 0b0000_0100;
        const FREQUENCY_BIT1 = 0b0000_0010;
        const FREQUENCY_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF19 - NR24 - Channel 2 Frequency hi data (R/W)
    /// - Bit 7   - Initial (1=Restart Sound)     (Write Only)
    /// - Bit 6   - Counter/consecutive selection (Read/Write)
    ///            (1=Stop output when length in NR21 expires)
    /// - Bit 2-0 - Frequency's higher 3 bits (x) (Write Only)
    ///
    /// Frequency = 131072/(2048-x) Hz
    pub struct Channel2FrequencyHigherData: u8 {
        const RESTART_SEQUENCE              = 0b1000_0000;
        const STOP_ON_SEQUENCE_COMPLETE     = 0b0100_0000;

        const UNUSED_BIT5 = 0b0010_0000;
        const UNUSED_BIT4 = 0b0001_0000;
        const UNUSED_BIT3 = 0b0000_1000;

        const FREQUENCY_HIGHER_MASK = 0b0000_0111;

        const FREQUENCY_BIT10 = 0b0000_0100;
        const FREQUENCY_BIT9  = 0b0000_0010;
        const FREQUENCY_BIT8  = 0b0000_0001;
    }
}

bitflags! {
    /// FF1A - NR30 - Channel 3 Sound on/off (R/W)
    /// - Bit 7 - Sound Channel 3 Off  (0=Stop, 1=Playback)  (Read/Write)
    pub struct Channel3SoundOnOffStatus: u8 {
        const CHANNEL_3_ENABLE = 1 << 7;
        const UNUSED_MASK = 0b0111_1111;
        const UNUSED_BIT6 = 0b0100_0000;
        const UNUSED_BIT5 = 0b0010_0000;
        const UNUSED_BIT4 = 0b0001_0000;
        const UNUSED_BIT3 = 0b0000_1000;
        const UNUSED_BIT2 = 0b0000_0100;
        const UNUSED_BIT1 = 0b0000_0010;
        const UNUSED_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF1B - NR31 - Channel 3 Sound Length
    /// - Bit 7-0 - Sound length (t1: 0 - 255)
    ///
    /// Sound Length = (256-t1)*(1/256) seconds.
    ///
    /// This value is used only if Bit 6 in NR34 is set.
    pub struct Channel3SoundSequenceLength: u8 {
        const SOUND_SEQUENCE_LENGTH_MASK = 0b1111_1111;
        const SOUND_SEQUENCE_LENGTH_BIT7 = 0b1000_0000;
        const SOUND_SEQUENCE_LENGTH_BIT6 = 0b0100_0000;
        const SOUND_SEQUENCE_LENGTH_BIT5 = 0b0010_0000;
        const SOUND_SEQUENCE_LENGTH_BIT4 = 0b0001_0000;
        const SOUND_SEQUENCE_LENGTH_BIT3 = 0b0000_1000;
        const SOUND_SEQUENCE_LENGTH_BIT2 = 0b0000_0100;
        const SOUND_SEQUENCE_LENGTH_BIT1 = 0b0000_0010;
        const SOUND_SEQUENCE_LENGTH_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF1C - NR32 - Channel 3 Select output level (R/W)
    /// - Bit 6-5 - Select output level (Read/Write)
    ///
    /// Possible Output levels are:
    ///
    ///     0: Mute (No sound)
    ///     1: 100% Volume (Produce Wave Pattern RAM Data as it is)
    ///     2:  50% Volume (Produce Wave Pattern RAM data shifted once to the right)
    ///     3:  25% Volume (Produce Wave Pattern RAM data shifted twice to the right)
    pub struct Channel3VolumeSelection: u8 {
        const VOLUME_MASK = 0b0110_0000;
        const VOLUME_BIT1 = 0b0100_0000;
        const VOLUME_BIT0 = 0b0010_0000;

        const UNUSED_BIT7 = 0b1000_0000;
        const UNUSED_BIT4 = 0b0001_0000;
        const UNUSED_BIT3 = 0b0000_1000;
        const UNUSED_BIT2 = 0b0000_0100;
        const UNUSED_BIT1 = 0b0000_0010;
        const UNUSED_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF1D - NR33 - Channel 3 Frequency's lower data (W)
    /// - Bit 7-0  - Frequency's lower 8 8 bits of an 11 bit frequency (x).
    pub struct Channel3FrequencyLowerData: u8 {
        const FREQUENCY_LOWER_MASK = 0b1111_1111;

        const FREQUENCY_BIT7 = 0b1000_0000;
        const FREQUENCY_BIT6 = 0b0100_0000;
        const FREQUENCY_BIT5 = 0b0010_0000;
        const FREQUENCY_BIT4 = 0b0001_0000;
        const FREQUENCY_BIT3 = 0b0000_1000;
        const FREQUENCY_BIT2 = 0b0000_0100;
        const FREQUENCY_BIT1 = 0b0000_0010;
        const FREQUENCY_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF1E - NR34 - Channel 3 Frequency's higher data (R/W)
    /// - Bit 7   - Initial (1=Restart Sound)     (Write Only)
    /// - Bit 6   - Counter/consecutive selection (Read/Write)
    ///            (1=Stop output when length in NR31 expires)
    /// - Bit 2-0 - Frequency's higher 3 bits (x) (Write Only)
    pub struct Channel3FrequencyHigherData: u8 {
        const RESTART_SEQUENCE              = 0b1000_0000;
        const STOP_ON_SEQUENCE_COMPLETE     = 0b0100_0000;

        const UNUSED_BIT5 = 0b0010_0000;
        const UNUSED_BIT4 = 0b0001_0000;
        const UNUSED_BIT3 = 0b0000_1000;

        const FREQUENCY_HIGHER_MASK = 0b0000_0111;

        const FREQUENCY_BIT10 = 0b0000_0100;
        const FREQUENCY_BIT9  = 0b0000_0010;
        const FREQUENCY_BIT8  = 0b0000_0001;
    }
}

bitflags! {
    /// FF20 - NR41 - Channel 4 Sound Length (R/W)
    /// - Bit 5-0 - Sound length data (t1: 0-63)
    ///
    /// Sound Length = (64-t1)*(1/256) seconds.
    ///
    /// The Length value is used only if Bit 6 in NR44 is set.
    pub struct Channel4SoundSequenceLength: u8 {
        const SOUND_SEQUENCE_LENGTH_MASK = 0b0011_1111;
        const SOUND_SEQUENCE_LENGTH_BIT5 = 0b0010_0000;
        const SOUND_SEQUENCE_LENGTH_BIT4 = 0b0001_0000;
        const SOUND_SEQUENCE_LENGTH_BIT3 = 0b0000_1000;
        const SOUND_SEQUENCE_LENGTH_BIT2 = 0b0000_0100;
        const SOUND_SEQUENCE_LENGTH_BIT1 = 0b0000_0010;
        const SOUND_SEQUENCE_LENGTH_BIT0 = 0b0000_0001;

        const UNUSED_BIT7 = 0b1000_0000;
        const UNUSED_BIT6 = 0b0100_0000;
    }
}

bitflags! {
    /// FF21 - NR42 - Channel 4 Volume Envelope (R/W)
    /// - Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
    /// - Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
    /// - Bit 2-0 - Number of envelope sweep (n: 0-7)
    ///              (If zero, stop envelope operation.)
    ///
    /// Length of 1 step = n*(1/64) seconds.
    pub struct Channel4EnvelopeControl: u8 {
        const ENVELOPE_INITIAL_VOLUME_MASK = 0b1111_0000;
        const ENVELOPE_SWEEP_NUMBER_MASK   = 0b0000_0111;

        const ENVELOPE_INITIAL_VOLUME_BIT3 = 0b1000_0000;
        const ENVELOPE_INITIAL_VOLUME_BIT2 = 0b0100_0000;
        const ENVELOPE_INITIAL_VOLUME_BIT1 = 0b0010_0000;
        const ENVELOPE_INITIAL_VOLUME_BIT0 = 0b0001_0000;

        const ENVELOPE_DIRECTION_SELECT    = 0b0000_1000;

        const ENVELOPE_SWEEP_NUMBER_BIT2   = 0b0000_0100;
        const ENVELOPE_SWEEP_NUMBER_BIT1   = 0b0000_0010;
        const ENVELOPE_SWEEP_NUMBER_BIT0   = 0b0000_0001;
    }
}

bitflags! {
    /// FF22 - NR43 - Channel 4 Polynomial Counter (R/W)
    /// - Bit 7-4 - Shift Clock Frequency (s)
    /// - Bit 3   - Counter Step/Width (0=15 bits, 1=7 bits)
    /// - Bit 2-0 - Dividing Ratio of Frequencies (r)
    ///
    /// The amplitude is randomly switched between high and low at the given frequency.
    /// A higher frequency will make the noise to appear 'softer'. When Bit 3 is set,
    /// the output will become more regular, and some frequencies will sound more like
    /// Tone than Noise.
    pub struct Channel4PolynomialCounterParameterControl: u8 {
        const FREQUENCY_SHIFT_MASK   = 0b1111_0000;
        const FREQUENCY_DIVIDER_MASK = 0b0000_0111;

        const FREQUENCY_SHIFT_BIT3 = 0b1000_0000;
        const FREQUENCY_SHIFT_BIT2 = 0b0100_0000;
        const FREQUENCY_SHIFT_BIT1 = 0b0010_0000;
        const FREQUENCY_SHIFT_BIT0 = 0b0001_0000;

        const COUNTER_STEP_SELECT = 0b0000_1000;

        const FREQUENCY_DIVIDER_BIT2 = 0b0000_0100;
        const FREQUENCY_DIVIDER_BIT1 = 0b0000_0010;
        const FREQUENCY_DIVIDER_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF23 - NR44 - Channel 4 Counter/consecutive; Inital (R/W)
    /// - Bit 7   - Initial (1=Restart Sound)     (Write Only)
    /// - Bit 6   - Counter/consecutive selection (Read/Write)
    ///            (1=Stop output when length in NR41 expires)
    pub struct Channel4PolynomialCounterSequenceControl: u8 {
        const RESTART_SEQUENCE              = 0b1000_0000;
        const STOP_ON_SEQUENCE_COMPLETE     = 0b0100_0000;

        const UNUSED_BIT5 = 0b0010_0000;
        const UNUSED_BIT4 = 0b0001_0000;
        const UNUSED_BIT3 = 0b0000_1000;
        const UNUSED_BIT2 = 0b0000_0100;
        const UNUSED_BIT1 = 0b0000_0010;
        const UNUSED_BIT0 = 0b0000_0001;
    }

}

bitflags! {
    /// FF24 - NR50 - Channel control / ON-OFF / Volume (R/W)
    /// - Bit 7   - Output Vin to SO2 terminal (1=Enable)
    /// - Bit 6-4 - SO2 output level (volume)  (0-7)
    /// - Bit 3   - Output Vin to SO1 terminal (1=Enable)
    /// - Bit 2-0 - SO1 output level (volume)  (0-7)
    ///
    /// The volume bits specify the "Master Volume" for Left/Right sound output.
    /// SO2 goes to the left headphone, and SO1 goes to the right.
    ///
    /// The Vin signal is an analog signal received from the game cartridge bus,
    /// allowing external hardware in the cartridge to supply a fifth sound channel,
    /// additionally to the Game Boy's internal four channels. No licensed games used
    /// this feature, and it was omitted from the Game Boy Advance.
    ///
    ///
    /// Despite rumors, Pocket Music does not use Vin.
    /// It blocks use on the GBA for a different reason: the developer couldn't figure out
    /// how to silence buzzing associated with the wave channel's DAC.
    pub struct MasterVolumeControl: u8 {
        const LEFT_CHANNEL_5_ENABLE    = 0b1000_0000;
        const LEFT_CHANNEL_VOLUME_MASK = 0b0111_0000;
        const LEFT_CHANNEL_VOLUME_BIT2 = 0b0100_0000;
        const LEFT_CHANNEL_VOLUME_BIT1 = 0b0010_0000;
        const LEFT_CHANNEL_VOLUME_BIT0 = 0b0001_0000;

        const RIGHT_CHANNEL_5_ENABLE    = 0b0000_1000;
        const RIGHT_CHANNEL_VOLUME_MASK = 0b0000_0111;
        const RIGHT_CHANNEL_VOLUME_BIT2 = 0b0000_0100;
        const RIGHT_CHANNEL_VOLUME_BIT1 = 0b0000_0010;
        const RIGHT_CHANNEL_VOLUME_BIT0 = 0b0000_0001;
    }
}

bitflags! {
    /// FF25 - NR51 - Selection of Sound output terminal (R/W)
    /// - Bit 7 - Output sound 4 to SO2 terminal
    /// - Bit 6 - Output sound 3 to SO2 terminal
    /// - Bit 5 - Output sound 2 to SO2 terminal
    /// - Bit 4 - Output sound 1 to SO2 terminal
    /// - Bit 3 - Output sound 4 to SO1 terminal
    /// - Bit 2 - Output sound 3 to SO1 terminal
    /// - Bit 1 - Output sound 2 to SO1 terminal
    /// - Bit 0 - Output sound 1 to SO1 terminal
    ///
    /// Each channel can be panned hard left, center, or hard right.
    pub struct MasterOutputControl: u8 {
        const LEFT_CHANNEL_4_ENABLE = 1 << 7;
        const LEFT_CHANNEL_3_ENABLE = 1 << 6;
        const LEFT_CHANNEL_2_ENABLE = 1 << 5;
        const LEFT_CHANNEL_1_ENABLE = 1 << 4;
        const RIGHT_CHANNEL_4_ENABLE = 1 << 3;
        const RIGHT_CHANNEL_3_ENABLE = 1 << 2;
        const RIGHT_CHANNEL_2_ENABLE = 1 << 1;
        const RIGHT_CHANNEL_1_ENABLE = 1 << 0;
    }
}

bitflags! {
    /// FF26 - NR52 - Sound on/off
    /// - Bit 7 - All sound on/off  (0: stop all sound circuits) (Read/Write)
    /// - Bit 3 - Sound 4 ON flag (Read Only)
    /// - Bit 2 - Sound 3 ON flag (Read Only)
    /// - Bit 1 - Sound 2 ON flag (Read Only)
    /// - Bit 0 - Sound 1 ON flag (Read Only)
    ///
    /// If your GB programs don't use sound then write 00h to this register
    /// to save 16% or more on GB power consumption. Disabeling the sound
    /// controller by clearing Bit 7 destroys the contents of all sound registers.
    /// Also, it is not possible to access any sound registers (except FF26)
    /// while the sound controller is disabled.
    ///
    /// Bits 0-3 of this register are read only status bits, writing to these bits
    /// does NOT enable/disable sound. The flags get set when sound output is restarted
    /// by setting the Initial flag (Bit 7 in NR14-NR44), the flag remains set until the
    /// sound length has expired (if enabled). A volume envelopes which has decreased to
    /// zero volume will NOT cause the sound flag to go off.
    pub struct MasterOnOffControl: u8 {
        const READ_WRITE_MASK = 0b1000_0000;
        const READ_ONLY_MASK  = 0b0111_1111;

        const CHANNEL_ALL_ENABLE = 1 << 7;
        const CHANNEL_4_ENABLE = 1 << 3;
        const CHANNEL_3_ENABLE = 1 << 2;
        const CHANNEL_2_ENABLE = 1 << 1;
        const CHANNEL_1_ENABLE = 1 << 0;

        const UNUSED_BIT6 = 1 << 6;
        const UNUSED_BIT5 = 1 << 5;
        const UNUSED_BIT4 = 1 << 4;
    }
}

impl Channel1SweepControl {
    pub fn sweep_inverse(&self) -> bool {
        self.contains(Channel1SweepControl::SWEEP_DIRECTION_SELECT)
    }

    pub fn sweep_period(&self) -> u8 {
        (*self & Channel1SweepControl::SWEEP_PERIOD_MASK).bits() >> 4
    }

    pub fn sweep_shift(&self) -> u8 {
        (*self & Channel1SweepControl::SWEEP_SHIFT_MASK).bits()
    }
}

impl Channel1SequenceControl {
    pub fn sequence_duty(&self) -> u8 {
        (*self & Channel1SequenceControl::SOUND_SEQUENCE_DUTY_MASK).bits() >> 6
    }

    pub fn sequence_length(&self) -> u8 {
        (*self & Channel1SequenceControl::SOUND_SEQUENCE_LENGTH_MASK).bits()
    }
}

impl Channel1EnvelopeControl {
    pub fn envelope_initial_volume(&self) -> u8 {
        (*self & Channel1EnvelopeControl::ENVELOPE_INITIAL_VOLUME_MASK).bits() >> 4
    }

    pub fn is_envelope_increase_direction(&self) -> bool {
        self.contains(Channel1EnvelopeControl::ENVELOPE_DIRECTION_SELECT)
    }

    pub fn is_envelope_decrease_direction(&self) -> bool {
        !self.is_envelope_increase_direction()
    }

    pub fn envelope_sweep_number(&self) -> u8 {
        (*self & Channel1EnvelopeControl::ENVELOPE_SWEEP_NUMBER_MASK).bits()
    }
}

impl Channel1FrequencyLowerData {
    pub fn frequency_lower_part(&self) -> u32 {
        self.bits().into()
    }
}

impl Channel1FrequencyHigherData {
    pub fn frequency_higher_part(&self) -> u32 {
        (*self & Channel1FrequencyHigherData::FREQUENCY_HIGHER_MASK).bits().into()
    }

    pub fn is_sequence_to_stop_when_complete(&self) -> bool {
        self.contains(Channel1FrequencyHigherData::STOP_ON_SEQUENCE_COMPLETE)
    }

    pub fn is_sequence_to_repeat_when_complete(&self) -> bool {
        !self.is_sequence_to_stop_when_complete()
    }

    pub fn is_sequence_to_restart(&self) -> bool {
        self.contains(Channel1FrequencyHigherData::RESTART_SEQUENCE)
    }
}

impl Channel2SequenceControl {
    pub fn sequence_duty(&self) -> u8 {
        (*self & Channel2SequenceControl::SOUND_SEQUENCE_DUTY_MASK).bits() >> 6
    }

    pub fn sequence_length(&self) -> u8 {
        (*self & Channel2SequenceControl::SOUND_SEQUENCE_LENGTH_MASK).bits()
    }
}

impl Channel2EnvelopeControl {
    pub fn envelope_initial_volume(&self) -> u8 {
        (*self & Channel2EnvelopeControl::ENVELOPE_INITIAL_VOLUME_MASK).bits() >> 4
    }

    pub fn is_envelope_increase_direction(&self) -> bool {
        self.contains(Channel2EnvelopeControl::ENVELOPE_DIRECTION_SELECT)
    }

    pub fn is_envelope_decrease_direction(&self) -> bool {
        !self.is_envelope_increase_direction()
    }

    pub fn envelope_sweep_number(&self) -> u8 {
        (*self & Channel2EnvelopeControl::ENVELOPE_SWEEP_NUMBER_MASK).bits()
    }
}

impl Channel2FrequencyLowerData {
    pub fn frequency_lower_part(&self) -> u32 {
        self.bits().into()
    }
}

impl Channel2FrequencyHigherData {
    pub fn frequency_higher_part(&self) -> u32 {
        (*self & Channel2FrequencyHigherData::FREQUENCY_HIGHER_MASK).bits().into()
    }

    pub fn is_sequence_to_stop_when_complete(&self) -> bool {
        self.contains(Channel2FrequencyHigherData::STOP_ON_SEQUENCE_COMPLETE)
    }

    pub fn is_sequence_to_repeat_when_complete(&self) -> bool {
        !self.is_sequence_to_stop_when_complete()
    }

    pub fn is_sequence_to_restart(&self) -> bool {
        self.contains(Channel2FrequencyHigherData::RESTART_SEQUENCE)
    }
}

impl MasterVolumeControl {
    pub fn left_volume(&self) -> u8 {
        (*self & MasterVolumeControl::LEFT_CHANNEL_VOLUME_MASK).bits() >> 4
    }

    pub fn right_volume(&self) -> u8 {
        (*self & MasterVolumeControl::RIGHT_CHANNEL_VOLUME_MASK).bits()
    }
}

impl MasterOutputControl {
    pub fn left_channel1_enable(&self) -> bool {
        self.contains(MasterOutputControl::LEFT_CHANNEL_1_ENABLE)
    }

    pub fn left_channel2_enable(&self) -> bool {
        self.contains(MasterOutputControl::LEFT_CHANNEL_2_ENABLE)
    }

    pub fn left_channel3_enable(&self) -> bool {
        self.contains(MasterOutputControl::LEFT_CHANNEL_3_ENABLE)
    }

    pub fn left_channel4_enable(&self) -> bool {
        self.contains(MasterOutputControl::LEFT_CHANNEL_4_ENABLE)
    }

    pub fn right_channel1_enable(&self) -> bool {
        self.contains(MasterOutputControl::RIGHT_CHANNEL_1_ENABLE)
    }

    pub fn right_channel2_enable(&self) -> bool {
        self.contains(MasterOutputControl::RIGHT_CHANNEL_2_ENABLE)
    }

    pub fn right_channel3_enable(&self) -> bool {
        self.contains(MasterOutputControl::RIGHT_CHANNEL_3_ENABLE)
    }

    pub fn right_channel4_enable(&self) -> bool {
        self.contains(MasterOutputControl::RIGHT_CHANNEL_4_ENABLE)
    }
}

impl MasterOnOffControl {
    pub fn all_channels_enable(&self) -> bool {
        self.contains(MasterOnOffControl::CHANNEL_ALL_ENABLE)
    }
}
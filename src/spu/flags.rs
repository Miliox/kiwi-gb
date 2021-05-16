use packed_struct::prelude::*;

#[derive(PackedStruct, Default, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct SweepControl {
    #[packed_field(bits="7")]
    _unused: ReservedOne<packed_bits::Bits1>,

    #[packed_field(bits="4..=6")]
    pub sweep_period: u8,

    #[packed_field(bits="3")]
    pub sweep_inverse: bool,

    #[packed_field(bits="0..=2")]
    pub sweep_shift: u8,
}

impl std::fmt::Debug for SweepControl {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("SweepControl")
            .field("period", &self.sweep_period)
            .field("direction", &self.sweep_inverse)
            .field("shift", &self.sweep_shift)
            .finish()
    }
}

#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct SequenceControl {
    #[packed_field(bits="6..=7")]
    pub duty: u8,

    #[packed_field(bits="0..=5")]
    pub data_length: u8,
}

impl SequenceControl {
    pub fn phase_duty(&self) -> f32 {
        match self.duty {
            0 => 0.125,
            1 => 0.25,
            2 => 0.5,
            3 => 0.75,
            _ => 0.5
        }
    }
}

#[derive(PackedStruct, Default, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct EnvelopeControl {
    #[packed_field(bits="4..=7")]
    pub initial_volume: u8,

    #[packed_field(bits="3")]
    pub envelope_direction: bool,

    #[packed_field(bits="0..=2")]
    pub envelope_step: u8,
}

impl std::fmt::Debug for EnvelopeControl {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("EnvelopeControl")
            .field("initial_volume", &self.initial_volume)
            .field("direction", &self.envelope_direction)
            .field("step", &self.envelope_step)
            .finish()
    }
}

#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct FrequencyLowerData {
    #[packed_field(bits="0..=7")]
    pub frequency_lower: u8,
}

#[derive(PackedStruct, Default, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct FrequencyHigherData {
    #[packed_field(bits="7")]
    pub restart_sequence: bool,

    #[packed_field(bits="6")]
    pub stop_on_complete: bool,

    #[packed_field(bits="3..=5")]
    _unused: ReservedOnes<packed_bits::Bits3>,

    #[packed_field(bits="0..=2")]
    pub frequency_higher: u8,
}

impl std::fmt::Debug for FrequencyHigherData {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("FrequencyHigherData")
            .field("restart_sequence", &self.restart_sequence)
            .field("stop_on_complete", &self.stop_on_complete)
            .field("frequency_higher", &self.frequency_higher)
            .finish()
    }
}

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
pub type Channel1SweepControl = SweepControl;

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
pub type Channel1SequenceControl = SequenceControl;

/// FF12 - NR12 - Channel 1 Volume Envelope (R/W)
/// - Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
/// - Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
/// - Bit 2-0 - Number of envelope sweep (n: 0-7)
///             (If zero, stop envelope operation.)
///
/// Length of 1 step = n*(1/64) seconds
pub type Channel1EnvelopeControl = EnvelopeControl;

/// FF13 - NR13 - Channel 1 Frequency low (Write Only)
/// - Bit 7-0 - Lower 8 bits of 11 bit frequency (x).
///
/// Next 3 bit are in NR14 ($FF14)
pub type Channel1FrequencyLowerData = FrequencyLowerData;

/// FF14 - NR14 - Channel 1 Frequency high (R/W)
/// - Bit 7   - Initial (1=Restart Sound)     (Write Only)
/// - Bit 6   - Counter/consecutive selection (Read/Write)
///           (1=Stop output when length in NR11 expires)
/// - Bit 2-0 - Frequency's higher 3 bits (x) (Write Only)
///
/// Frequency = 131072/(2048-x) Hz
pub type Channel1FrequencyHigherData = FrequencyHigherData;

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
pub type Channel2SequenceControl = SequenceControl;

/// FF17 - NR22 - Channel 2 Volume Envelope (R/W)
/// - Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
/// - Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
/// - Bit 2-0 - Number of envelope sweep (n: 0-7)
///              (If zero, stop envelope operation.)
///
/// Length of 1 step = n*(1/64) seconds
pub type Channel2EnvelopeControl = EnvelopeControl;

/// FF18 - NR23 - Channel 2 Frequency low data (W)
/// - Bit 7-0 - Frequency's lower 8 bits of 11 bit data (x).
///
/// Next 3 bits are in NR24 ($FF19).
pub type Channel2FrequencyLowerData = FrequencyLowerData;

/// FF19 - NR24 - Channel 2 Frequency hi data (R/W)
/// - Bit 7   - Initial (1=Restart Sound)     (Write Only)
/// - Bit 6   - Counter/consecutive selection (Read/Write)
///            (1=Stop output when length in NR21 expires)
/// - Bit 2-0 - Frequency's higher 3 bits (x) (Write Only)
///
/// Frequency = 131072/(2048-x) Hz
pub type Channel2FrequencyHigherData = FrequencyHigherData;

/// FF1A - NR30 - Channel 3 Sound on/off (R/W)
/// - Bit 7 - Sound Channel 3 Off  (0=Stop, 1=Playback)  (Read/Write)
#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct Channel3SoundOnOffStatus {
    #[packed_field(bits="7")]
    pub enable: bool,

    #[packed_field(bits="0..=6")]
    _unused: ReservedOnes<packed_bits::Bits6>,
}

/// FF1B - NR31 - Channel 3 Sound Length
/// - Bit 7-0 - Sound length (t1: 0 - 255)
///
/// Sound Length = (256-t1)*(1/256) seconds.
///
/// This value is used only if Bit 6 in NR34 is set.
#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct Channel3SoundSequenceLength {
    #[packed_field(bits="0..=7")]
    pub data_length: u8,
}

/// FF1C - NR32 - Channel 3 Select output level (R/W)
/// - Bit 6-5 - Select output level (Read/Write)
///
/// Possible Output levels are:
///
///     0: Mute (No sound)
///     1: 100% Volume (Produce Wave Pattern RAM Data as it is)
///     2:  50% Volume (Produce Wave Pattern RAM data shifted once to the right)
///     3:  25% Volume (Produce Wave Pattern RAM data shifted twice to the right)
#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct Channel3VolumeSelection {
    #[packed_field(bits="7")]
    _unused1: ReservedOnes<packed_bits::Bits1>,

    #[packed_field(bits="5..=6")]
    pub volume: u8,

    #[packed_field(bits="0..=4")]
    _unused0: ReservedOnes<packed_bits::Bits5>,
}

/// FF1D - NR33 - Channel 3 Frequency's lower data (W)
/// - Bit 7-0  - Frequency's lower 8 8 bits of an 11 bit frequency (x).
pub type Channel3FrequencyLowerData = FrequencyLowerData;

/// FF1E - NR34 - Channel 3 Frequency's higher data (R/W)
/// - Bit 7   - Initial (1=Restart Sound)     (Write Only)
/// - Bit 6   - Counter/consecutive selection (Read/Write)
///            (1=Stop output when length in NR31 expires)
/// - Bit 2-0 - Frequency's higher 3 bits (x) (Write Only)
pub type Channel3FrequencyHigherData = FrequencyHigherData;

/// FF20 - NR41 - Channel 4 Sound Length (R/W)
/// - Bit 5-0 - Sound length data (t1: 0-63)
///
/// Sound Length = (64-t1)*(1/256) seconds.
///
/// The Length value is used only if Bit 6 in NR44 is set.
#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct Channel4SoundSequenceLength {
    #[packed_field(bits="6..=7")]
    _unused: ReservedOnes<packed_bits::Bits3>,

    #[packed_field(bits="0..=5")]
    pub data_length: u8,
}

/// FF21 - NR42 - Channel 4 Volume Envelope (R/W)
/// - Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
/// - Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
/// - Bit 2-0 - Number of envelope sweep (n: 0-7)
///              (If zero, stop envelope operation.)
///
/// Length of 1 step = n*(1/64) seconds.
pub type Channel4EnvelopeControl = EnvelopeControl;

/// FF22 - NR43 - Channel 4 Polynomial Counter (R/W)
/// - Bit 7-4 - Shift Clock Frequency (s)
/// - Bit 3   - Counter Step/Width (0=15 bits, 1=7 bits)
/// - Bit 2-0 - Dividing Ratio of Frequencies (r)
///
/// The amplitude is randomly switched between high and low at the given frequency.
/// A higher frequency will make the noise to appear 'softer'. When Bit 3 is set,
/// the output will become more regular, and some frequencies will sound more like
/// Tone than Noise.
#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct Channel4PolynomialCounterParameterControl {
    #[packed_field(bits="4..=7")]
    pub frequency_shift: u8,

    #[packed_field(bits="3")]
    pub step_width: u8,

    #[packed_field(bits="0..=2")]
    pub frequency_divider: u8,
}

/// FF23 - NR44 - Channel 4 Counter/consecutive; Inital (R/W)
/// - Bit 7   - Initial (1=Restart Sound)     (Write Only)
/// - Bit 6   - Counter/consecutive selection (Read/Write)
///            (1=Stop output when length in NR41 expires)
pub type Channel4PolynomialCounterSequenceControl = SequenceControl;

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
#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct MasterVolumeControl {
    #[packed_field(bits="7")]
    pub left_channel_5_enable: bool,

    #[packed_field(bits="4..=6")]
    pub left_volume: u8,

    #[packed_field(bits="3")]
    pub right_channel_5_enable: bool,

    #[packed_field(bits="0..=2")]
    pub right_volume: u8,
}

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
#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct MasterOutputControl {
    #[packed_field(bits="7")]
    pub left_channel_4_enable: bool,

    #[packed_field(bits="6")]
    pub left_channel_3_enable: bool,

    #[packed_field(bits="5")]
    pub left_channel_2_enable: bool,

    #[packed_field(bits="4")]
    pub left_channel_1_enable: bool,

    #[packed_field(bits="3")]
    pub right_channel_4_enable: bool,

    #[packed_field(bits="2")]
    pub right_channel_3_enable: bool,

    #[packed_field(bits="1")]
    pub right_channel_2_enable: bool,

    #[packed_field(bits="0")]
    pub right_channel_1_enable: bool,
}

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
#[derive(PackedStruct, Default, Debug, PartialEq)]
#[packed_struct(bit_numbering="lsb0",size_bytes="1")]
pub struct MasterOnOffControl {
    #[packed_field(bits="7")]
    pub all_channels_enable: bool,

    #[packed_field(bits="4..=6")]
    _unused: ReservedOnes<packed_bits::Bits3>,

    #[packed_field(bits="3")]
    pub channel_4_enable: bool,

    #[packed_field(bits="2")]
    pub channel_3_enable: bool,

    #[packed_field(bits="1")]
    pub channel_2_enable: bool,

    #[packed_field(bits="0")]
    pub channel_1_enable: bool,
}

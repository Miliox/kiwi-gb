use sdl2::audio::AudioQueue;

pub trait Sampler {
    fn enqueue_audio_samples(&mut self, queue: &mut AudioQueue<i8>);
}
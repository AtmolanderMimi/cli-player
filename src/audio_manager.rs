use std::{io::BufReader, fs::File};

use crate::VideoError;
use crate::video::RodioError;

use rodio::{OutputStream, OutputStreamHandle, Sink, Decoder};

// NOTE: output_stream and _output_stream_handle need to be kept in memory for the audio
// to play
pub struct AudioManager {
    _output_stream: OutputStream,
    _output_stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl AudioManager {
    pub fn build() -> Result<AudioManager, VideoError> {
        let (_output_stream, output_stream_handle) = match OutputStream::try_default() {
            Ok((output, s)) => (output, s),
            Err(e) => return Err(VideoError::RodioError(RodioError::StreamError(e)))
        };

        let sink = match Sink::try_new(&output_stream_handle) {
            Ok(s) => s,
            Err(e) => return Err(VideoError::RodioError(RodioError::PlayError(e))),
        };
        
        Ok(AudioManager {
            _output_stream,
            _output_stream_handle: output_stream_handle,
            sink,
        })
    }
}

impl AudioManager {
    fn audio_buffer_from_path(path: &str) -> Result<Decoder<BufReader<File>>, VideoError> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(VideoError::IoError(e)),
        };

        let buf_reader = BufReader::new(file);

        // "Error while playing the video: Unrecognized format" while trying to open .mp4 file
        let audio_source =  match Decoder::new(buf_reader) { 
            Ok(d) => d,
            Err(e) => return Err(VideoError::RodioError(RodioError::DecoderError(e)))
        };

        Ok(audio_source)
    }

    pub fn play(&self, audio_source: Decoder<BufReader<File>>) {
        self.sink.append(audio_source)
    }

    pub fn play_from_path(&self, path: &str) -> Result<(), VideoError> {
        let audio_source = AudioManager::audio_buffer_from_path(path)?;
        self.play(audio_source);

        Ok(())
    }

    pub fn stop(&self) {
        self.sink.stop()
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume)
    }
}
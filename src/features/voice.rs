use crate::error::{EchomindError, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig, SampleFormat};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

pub struct VoiceManager {
    _stream: Option<Stream>,
    output_handle: Option<OutputStreamHandle>,
    recording: Arc<Mutex<bool>>,
}

impl VoiceManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            _stream: None,
            output_handle: None,
            recording: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn start_voice_input(&mut self, duration_seconds: Option<u64>) -> Result<String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| EchomindError::Other("No audio input device found".to_string()))?;

        let config = device.default_input_config()
            .map_err(|e| EchomindError::Other(format!("Failed to get input config: {}", e)))?;

        let recording = Arc::clone(&self.recording);
        *recording.lock().unwrap() = true;

        // For now, we'll use a simple approach with system commands
        // In a real implementation, you'd integrate with a proper speech-to-text library
        let audio_file = "/tmp/echomind_recording.wav";
        
        // Record audio using system command (platform-specific)
        if cfg!(target_os = "macos") {
            Command::new("sox")
                .args(&["-d", "-r", "16000", "-c", "1", "-e", "signed-integer", audio_file])
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to record audio: {}", e)))?;
        } else if cfg!(target_os = "linux") {
            Command::new("arecord")
                .args(&["-f", "cd", "-d", &duration_seconds.unwrap_or(5).to_string(), audio_file])
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to record audio: {}", e)))?;
        } else if cfg!(target_os = "windows") {
            Command::new("powershell")
                .args(&["-Command", &format!("Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{{F8}}'); Start-Sleep -Seconds {}", duration_seconds.unwrap_or(5))])
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to record audio: {}", e)))?;
        }

        *recording.lock().unwrap() = false;

        // Transcribe using Whisper (if available)
        if let Ok(transcription) = self.transcribe_with_whisper(audio_file).await {
            Ok(transcription)
        } else {
            Err(EchomindError::Other("Speech-to-text not available".to_string()))
        }
    }

    async fn transcribe_with_whisper(&self, audio_file: &str) -> Result<String> {
        // This would integrate with whisper-rs
        // For now, return a placeholder
        Ok("Transcribed text from audio".to_string())
    }

    pub async fn text_to_speech(&mut self, text: &str, voice: Option<&str>) -> Result<()> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| EchomindError::Other(format!("Failed to create output stream: {}", e)))?;
        
        self.output_handle = Some(stream_handle);

        // For now, use system TTS
        let output_file = "/tmp/echomind_speech.wav";
        
        if cfg!(target_os = "macos") {
            Command::new("say")
                .args(&["-o", output_file, "--data-format=LEF32@16000", text])
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to generate speech: {}", e)))?;
        } else if cfg!(target_os = "linux") {
            Command::new("espeak")
                .args(&["-w", output_file, text])
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to generate speech: {}", e)))?;
        } else if cfg!(target_os = "windows") {
            Command::new("powershell")
                .args(&["-Command", &format!("Add-Type -AssemblyName System.Speech; $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; $synth.SetOutputToWaveFile('{}'); $synth.Speak('{}'); $synth.Dispose()", output_file, text)])
                .output()
                .await
                .map_err(|e| EchomindError::Other(format!("Failed to generate speech: {}", e)))?;
        }

        // Play the generated audio
        if let Some(ref handle) = self.output_handle {
            let file = File::open(output_file)
                .map_err(|e| EchomindError::Other(format!("Failed to open audio file: {}", e)))?;
            let source = Decoder::new(BufReader::new(file))
                .map_err(|e| EchomindError::Other(format!("Failed to decode audio: {}", e)))?;
            
            let sink = Sink::try_new(handle)
                .map_err(|e| EchomindError::Other(format!("Failed to create sink: {}", e)))?;
            
            sink.append(source);
            sink.sleep_until_end();
        }

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        *self.recording.lock().unwrap()
    }

    pub fn stop_recording(&mut self) -> Result<()> {
        *self.recording.lock().unwrap() = false;
        if let Some(stream) = &self._stream {
            stream.pause().map_err(|e| EchomindError::Other(format!("Failed to stop recording: {}", e)))?;
        }
        Ok(())
    }
}

impl Default for VoiceManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| VoiceManager {
            _stream: None,
            output_handle: None,
            recording: Arc::new(Mutex::new(false)),
        })
    }
}
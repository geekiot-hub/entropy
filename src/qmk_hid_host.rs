//! Small built-in qmk-hid-host bridge for display presets that expect host data.
//! Sends the same Raw HID packet family as https://github.com/ergohaven/qmk-hid-host.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const RAW_HID_PACKET_LEN: usize = 32;
const REPORT_PACKET_LEN: usize = RAW_HID_PACKET_LEN + 1;
const DATA_TIME: u8 = 0xAA;
const DATA_VOLUME: u8 = 0xAB;
const DATA_MEDIA_ARTIST: u8 = 0xAD;
const DATA_MEDIA_TITLE: u8 = 0xAE;

pub struct QmkHidHostBridge {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl QmkHidHostBridge {
    pub fn start(path: String) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let thread = thread::spawn(move || run_bridge(path, worker_stop));
        Self {
            stop,
            thread: Some(thread),
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for QmkHidHostBridge {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_bridge(path: String, stop: Arc<AtomicBool>) {
    let mut device: Option<hidapi::HidDevice> = None;
    let mut last_open_attempt = Instant::now() - Duration::from_secs(5);
    let mut last_time = None;
    let mut last_volume = None;
    let mut last_artist = String::new();
    let mut last_title = String::new();
    let mut last_time_poll = Instant::now() - Duration::from_secs(60);
    let mut last_volume_poll = Instant::now() - Duration::from_secs(60);
    let mut last_media_poll = Instant::now() - Duration::from_secs(60);

    while !stop.load(Ordering::Relaxed) {
        if device.is_none() && last_open_attempt.elapsed() >= Duration::from_secs(2) {
            last_open_attempt = Instant::now();
            device = open_raw_hid(&path)
                .map_err(|e| log::warn!("qmk-hid-host open failed: {e}"))
                .ok();
            if device.is_some() {
                log::info!("qmk-hid-host bridge started");
            }
        }

        let Some(dev) = device.as_ref() else {
            thread::sleep(Duration::from_millis(250));
            continue;
        };

        let mut write_failed = false;

        if last_time_poll.elapsed() >= Duration::from_secs(1) {
            last_time_poll = Instant::now();
            let now = current_time_payload();
            if last_time != Some(now) {
                last_time = Some(now);
                write_failed |= write_payload(dev, &[DATA_TIME, now.0, now.1]).is_err();
            }
        }

        if last_volume_poll.elapsed() >= Duration::from_secs(2) {
            last_volume_poll = Instant::now();
            if let Some(volume) = current_volume_percent() {
                if last_volume != Some(volume) {
                    last_volume = Some(volume);
                    write_failed |= write_payload(dev, &[DATA_VOLUME, volume]).is_err();
                }
            }
        }

        if last_media_poll.elapsed() >= Duration::from_secs(3) {
            last_media_poll = Instant::now();
            if let Some((artist, title)) = current_media_info() {
                if artist != last_artist {
                    last_artist = artist.clone();
                    write_failed |= write_text_payload(dev, DATA_MEDIA_ARTIST, &artist).is_err();
                }
                if title != last_title {
                    last_title = title.clone();
                    write_failed |= write_text_payload(dev, DATA_MEDIA_TITLE, &title).is_err();
                }
            }
        }

        if write_failed {
            log::warn!("qmk-hid-host bridge write failed; reconnecting");
            device = None;
        }

        thread::sleep(Duration::from_millis(200));
    }

    log::info!("qmk-hid-host bridge stopped");
}

fn open_raw_hid(path: &str) -> anyhow::Result<hidapi::HidDevice> {
    let api = hidapi::HidApi::new()?;
    Ok(api.open_path(&std::ffi::CString::new(path)?)?)
}

fn write_payload(device: &hidapi::HidDevice, payload: &[u8]) -> hidapi::HidResult<usize> {
    let mut packet = [0u8; REPORT_PACKET_LEN];
    let len = payload.len().min(RAW_HID_PACKET_LEN);
    packet[1..1 + len].copy_from_slice(&payload[..len]);
    device.write(&packet)
}

fn write_text_payload(
    device: &hidapi::HidDevice,
    data_type: u8,
    value: &str,
) -> hidapi::HidResult<usize> {
    let mut payload = Vec::with_capacity(RAW_HID_PACKET_LEN);
    let mut bytes = value.as_bytes().to_vec();
    bytes.truncate(30);
    payload.push(data_type);
    payload.push(bytes.len() as u8);
    payload.extend(bytes);
    write_payload(device, &payload)
}

fn current_time_payload() -> (u8, u8) {
    use chrono::Timelike;
    let now = chrono::Local::now();
    (now.hour() as u8, now.minute() as u8)
}

#[cfg(target_os = "windows")]
fn current_volume_percent() -> Option<u8> {
    windows_platform::volume_percent()
}

#[cfg(target_os = "linux")]
fn current_volume_percent() -> Option<u8> {
    command_stdout("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .and_then(|out| {
            out.split_whitespace()
                .find_map(|part| part.parse::<f32>().ok())
                .map(|v| (v * 100.0).round().clamp(0.0, 100.0) as u8)
        })
        .or_else(|| {
            command_stdout("pactl", &["get-sink-volume", "@DEFAULT_SINK@"]).and_then(|out| {
                out.split_whitespace()
                    .find(|part| part.ends_with('%'))
                    .and_then(|part| part.trim_end_matches('%').parse::<u8>().ok())
            })
        })
}

#[cfg(target_os = "macos")]
fn current_volume_percent() -> Option<u8> {
    command_stdout(
        "osascript",
        &["-e", "output volume of (get volume settings)"],
    )
    .and_then(|out| out.trim().parse::<u8>().ok())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn current_volume_percent() -> Option<u8> {
    None
}

#[cfg(target_os = "windows")]
fn current_media_info() -> Option<(String, String)> {
    windows_platform::media_info()
}

#[cfg(target_os = "linux")]
fn current_media_info() -> Option<(String, String)> {
    command_stdout(
        "playerctl",
        &["metadata", "--format", "{{artist}}\t{{title}}"],
    )
    .and_then(|out| split_media_line(&out))
}

#[cfg(target_os = "macos")]
fn current_media_info() -> Option<(String, String)> {
    let script = r#"
set mediaArtist to ""
set mediaTitle to ""
tell application "System Events"
    if exists process "Spotify" then
        tell application "Spotify"
            if player state is not stopped then
                set mediaArtist to artist of current track
                set mediaTitle to name of current track
            end if
        end tell
    else if exists process "Music" then
        tell application "Music"
            if player state is not stopped then
                set mediaArtist to artist of current track
                set mediaTitle to name of current track
            end if
        end tell
    end if
end tell
return mediaArtist & tab & mediaTitle
"#;
    command_stdout("osascript", &["-e", script]).and_then(|out| split_media_line(&out))
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn current_media_info() -> Option<(String, String)> {
    None
}

#[cfg(not(target_os = "windows"))]
fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

#[cfg(not(target_os = "windows"))]
fn split_media_line(line: &str) -> Option<(String, String)> {
    let mut parts = line.trim().splitn(2, '\t');
    let artist = parts.next().unwrap_or_default().trim().to_string();
    let title = parts.next().unwrap_or_default().trim().to_string();
    (!artist.is_empty() || !title.is_empty()).then_some((artist, title))
}

#[cfg(target_os = "windows")]
mod windows_platform {
    use windows::{
        Media::Control::GlobalSystemMediaTransportControlsSessionManager,
        Win32::{
            Media::Audio::{
                eMultimedia, eRender, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator,
                MMDeviceEnumerator,
            },
            System::Com::{
                CoCreateInstance, CoInitializeEx, CLSCTX_ALL, CLSCTX_INPROC_SERVER,
                COINIT_MULTITHREADED,
            },
        },
    };

    pub fn volume_percent() -> Option<u8> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).ok()?;
            let endpoint = enumerator
                .GetDefaultAudioEndpoint(eRender, eMultimedia)
                .ok()?;
            let volume = endpoint
                .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
                .ok()?;
            let scalar = volume.GetMasterVolumeLevelScalar().ok()?;
            Some((scalar * 100.0).round().clamp(0.0, 100.0) as u8)
        }
    }

    pub fn media_info() -> Option<(String, String)> {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .and_then(|request| request.get())
            .ok()?;
        let session = manager.GetCurrentSession().ok()?;
        let props = session
            .TryGetMediaPropertiesAsync()
            .and_then(|request| request.get())
            .ok()?;
        let artist = props.Artist().unwrap_or_default().to_string();
        let title = props.Title().unwrap_or_default().to_string();
        (!artist.is_empty() || !title.is_empty()).then_some((artist, title))
    }
}

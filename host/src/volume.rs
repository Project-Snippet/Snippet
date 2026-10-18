//! The default speaker's volume and mute state.
//!
//! Windows only: other platforms report `None`, because there is no single
//! API to ask. The unsafe COM calls are confined to `read_windows`.

/// Master volume (0-100) and whether the default output device is muted.
#[cfg(windows)]
pub fn read() -> Option<(u8, bool)> {
    read_windows().ok()
}

#[cfg(not(windows))]
pub fn read() -> Option<(u8, bool)> {
    None
}

#[cfg(windows)]
fn read_windows() -> windows::core::Result<(u8, bool)> {
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    /// Pairs CoInitializeEx with CoUninitialize, even on early return.
    struct ComGuard;
    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }

    // SAFETY: plain Win32 COM calls with valid arguments. The guard is declared
    // first so it is dropped last, after every COM object below is released.
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let _guard = ComGuard;

        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

        let level = endpoint.GetMasterVolumeLevelScalar()?; // 0.0 to 1.0
        let muted = endpoint.GetMute()?.as_bool();
        Ok((crate::system::to_percent(level * 100.0), muted))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reading_never_panics_and_is_a_valid_percentage() {
        if let Some((percent, _muted)) = super::read() {
            assert!(percent <= 100);
        }
    }
}

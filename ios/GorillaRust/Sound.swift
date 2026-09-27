import AVFoundation

/// iOS mutes "ambient" sound in Silent mode, so with sound on the game plays
/// as media does. Muted, it goes back to ambient, so a silent game never
/// stops music another app is playing.
enum AudioSessionPolicy {
    static func category(muted: Bool) -> AVAudioSession.Category {
        muted ? .ambient : .playback
    }
}

/// The mute choice, remembered between launches.
struct MuteStore {
    let defaults: UserDefaults
    var muted: Bool {
        get { defaults.bool(forKey: "muted") }
        set { defaults.set(newValue, forKey: "muted") }
    }
}

enum Sound {
    /// Set the audio session for `muted`, and mute or unmute the core.
    static func apply(muted: Bool) {
        let session = AVAudioSession.sharedInstance()
        try? session.setCategory(AudioSessionPolicy.category(muted: muted))
        try? session.setActive(true)
        Core.setMuted(muted)
    }

    /// When a call or Siri ends, iOS may have closed the audio output;
    /// reactivate the session and have the core reopen it.
    static func observeInterruptions() {
        NotificationCenter.default.addObserver(
            forName: AVAudioSession.interruptionNotification, object: nil, queue: .main
        ) { note in
            guard let raw = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt,
                  AVAudioSession.InterruptionType(rawValue: raw) == .ended else { return }
            try? AVAudioSession.sharedInstance().setActive(true)
            Core.reopenAudio()
        }
    }
}

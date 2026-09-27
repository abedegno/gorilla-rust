import AVFoundation
import SwiftUI

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

    /// Reactivate the session and have the core reopen its audio output,
    /// which iOS may have closed during a call, Siri or a media reset.
    static func resume() {
        try? AVAudioSession.sharedInstance().setActive(true)
        Core.reopenAudio()
    }

    /// Resume when an interruption ends, and when iOS resets its media
    /// services. Coming back from the background resumes too (see
    /// `AudioRecovery`), since iOS does not always say an interruption ended.
    static func observeInterruptions() {
        NotificationCenter.default.addObserver(
            forName: AVAudioSession.interruptionNotification, object: nil, queue: .main
        ) { note in
            guard let raw = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt,
                  AVAudioSession.InterruptionType(rawValue: raw) == .ended else { return }
            resume()
        }
        NotificationCenter.default.addObserver(
            forName: AVAudioSession.mediaServicesWereResetNotification, object: nil, queue: .main
        ) { _ in resume() }
    }
}

/// When to reopen the audio output as the app's scene changes phase: on the
/// way back from the background, where a call may have left it closed, but
/// not for a glance at Control Centre, which would cut a tune short.
enum AudioRecovery {
    static func shouldReopen(from old: ScenePhase, to new: ScenePhase) -> Bool {
        old == .background && new == .active
    }
}

import SwiftUI

@main
struct GorillaRustApp: App {
    init() {
        // The audio session is set before the core opens the audio output.
        let muted = MuteStore(defaults: .standard).muted
        Sound.apply(muted: muted)
        Sound.observeInterruptions()
        Core.start(seed: UInt64.random(in: 0..<(1 << 32)), muted: muted)
    }

    var body: some Scene {
        WindowGroup { ContentView() }
    }
}

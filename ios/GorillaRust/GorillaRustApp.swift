import SwiftUI

@main
struct GorillaRustApp: App {
    init() {
        Core.start(seed: UInt64.random(in: 0..<(1 << 32)), muted: false)
    }

    var body: some Scene {
        WindowGroup { ContentView() }
    }
}

import SwiftUI

struct ContentView: View {
    @Environment(\.scenePhase) private var phase
    @State private var error: String?

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()
            GameView(running: phase == .active && error == nil) { error = $0 }
            if let error {
                Text(error).foregroundStyle(.white).padding()
            }
        }
        .statusBarHidden()
        .persistentSystemOverlays(.hidden)
    }
}

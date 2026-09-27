import SwiftUI

struct ContentView: View {
    @Environment(\.scenePhase) private var phase
    @State private var error: String?
    @FocusState private var focused: Bool
    /// A plain Bool, so SwiftUI sees it change; MuteStore is only where it
    /// is kept between launches.
    @State private var muted = MuteStore(defaults: .standard).muted

    var body: some View {
        GeometryReader { geo in
            let landscape = geo.size.width > geo.size.height
            let game = GameView(running: phase == .active && error == nil) { error = $0 }
            let pad = VStack(spacing: 8) {
                Button(muted ? "Sound off" : "Sound on") {
                    muted.toggle()
                    var store = MuteStore(defaults: .standard)
                    store.muted = muted
                    Sound.apply(muted: muted)
                }
                .buttonStyle(.bordered)
                .tint(.white)
                .accessibilityValue(muted ? "off" : "on")
                Keypad { Core.push($0) }
            }
            Group {
                if landscape {
                    HStack(spacing: 12) {
                        game
                        pad.frame(width: min(geo.size.width * 0.34, 380))
                    }
                } else {
                    VStack(spacing: 12) {
                        game.frame(height: geo.size.width * 400 / 640)
                        pad
                        Spacer(minLength: 0)
                    }
                }
            }
            .padding(8)
            .overlay {
                if let error { Text(error).foregroundStyle(.white).padding() }
            }
        }
        .background(Color.black.ignoresSafeArea())
        .focusable()
        .focused($focused)
        .focusEffectDisabled()
        .onKeyPress(phases: .down) { press in
            guard let c = HardwareKeys.character(for: press.key, characters: press.characters,
                                                 modifiers: press.modifiers) else { return .ignored }
            Core.push(c)
            return .handled
        }
        .onAppear { focused = true }
        .statusBarHidden()
        .persistentSystemOverlays(.hidden)
    }
}

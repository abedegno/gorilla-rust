import SwiftUI

struct ContentView: View {
    @Environment(\.scenePhase) private var phase
    @State private var error: String?
    @FocusState private var focused: Bool

    var body: some View {
        GeometryReader { geo in
            let landscape = geo.size.width > geo.size.height
            let game = GameView(running: phase == .active && error == nil) { error = $0 }
            let pad = Keypad { Core.push($0) }
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

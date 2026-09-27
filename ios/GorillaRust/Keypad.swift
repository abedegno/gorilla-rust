import SwiftUI
import UIKit

/// The on-screen keypad, sending keys to the game through `send`.
struct Keypad: View {
    let send: (Character) -> Void
    @State private var state = PadState()
    private let haptic = UIImpactFeedbackGenerator(style: .light)

    var body: some View {
        let rows = state.layout == .digits ? Keys.digits : Keys.letters(shifted: state.shifted)
        VStack(spacing: 6) {
            ForEach(rows.indices, id: \.self) { r in
                HStack(spacing: 6) {
                    ForEach(rows[r]) { key in
                        Button { press(key) } label: {
                            Text(key.label)
                                .font(.system(size: 20, weight: .semibold, design: .rounded))
                                // "ABC" wrapped onto two lines in a narrow
                                // landscape column; shrink it to fit instead.
                                .lineLimit(1)
                                .minimumScaleFactor(0.6)
                                .frame(maxWidth: .infinity, minHeight: 44)
                        }
                        .buttonStyle(.bordered)
                        .tint(key.action == .shift && state.shifted ? .yellow : .white)
                        .layoutPriority(key.wide ? 1 : 0)
                        .accessibilityLabel(key.accessibility ?? key.label)
                    }
                }
            }
        }
    }

    private func press(_ key: PadKey) {
        haptic.impactOccurred()
        if let c = state.press(key) { send(c) }
    }
}

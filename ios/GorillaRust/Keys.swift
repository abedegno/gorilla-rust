import SwiftUI

enum PadLayout: Equatable { case digits, letters }

enum PadAction: Equatable {
    case send(Character)
    case switchTo(PadLayout)
    case shift
}

struct PadKey: Identifiable, Equatable {
    let label: String
    let action: PadAction
    var accessibility: String? = nil
    var wide = false
    var id: String { label }
}

/// The keypad's two layouts, key for key as the web page's.
enum Keys {
    static let digits: [[PadKey]] = [
        [.init(label: "7", action: .send("7")), .init(label: "8", action: .send("8")),
         .init(label: "9", action: .send("9")), .init(label: "⌫", action: .send("\u{8}"), accessibility: "Backspace")],
        [.init(label: "4", action: .send("4")), .init(label: "5", action: .send("5")),
         .init(label: "6", action: .send("6")), .init(label: "V", action: .send("V"))],
        [.init(label: "1", action: .send("1")), .init(label: "2", action: .send("2")),
         .init(label: "3", action: .send("3")), .init(label: "P", action: .send("P"))],
        [.init(label: "0", action: .send("0")), .init(label: ".", action: .send(".")),
         .init(label: "ABC", action: .switchTo(.letters), accessibility: "Letters"),
         .init(label: "⏎", action: .send("\r"), accessibility: "Enter")],
    ]

    static func letters(shifted: Bool) -> [[PadKey]] {
        func row(_ s: String) -> [PadKey] {
            s.map { c in PadKey(label: shifted ? c.uppercased() : String(c), action: .send(c)) }
        }
        return [
            row("qwertyuiop"),
            row("asdfghjkl"),
            [PadKey(label: "⇧", action: .shift, accessibility: "Shift")] + row("zxcvbnm")
                + [PadKey(label: "⌫", action: .send("\u{8}"), accessibility: "Backspace")],
            [PadKey(label: "123", action: .switchTo(.digits), accessibility: "Numbers"),
             PadKey(label: "space", action: .send(" "), wide: true),
             PadKey(label: "⏎", action: .send("\r"), accessibility: "Enter")],
        ]
    }
}

/// Which layout shows and whether shift is on. Shift is one-shot, as on a
/// phone and on the web page: it capitalises the next letter and lets go.
struct PadState {
    var layout: PadLayout = .digits
    var shifted = false

    /// The character a press sends to the game, if any.
    mutating func press(_ key: PadKey) -> Character? {
        switch key.action {
        case .switchTo(let layout):
            self.layout = layout
            return nil
        case .shift:
            shifted.toggle()
            return nil
        case .send(let c):
            guard shifted, c.isLetter, c.isLowercase else { return c }
            shifted = false
            return Character(c.uppercased())
        }
    }
}

/// What a hardware keyboard key sends, as the web page maps browser keys:
/// Return and Delete, and printable ASCII. Chords with Command, Control or
/// Option are left to the system.
enum HardwareKeys {
    static func character(for key: KeyEquivalent, characters: String,
                          modifiers: EventModifiers) -> Character? {
        if !modifiers.intersection([.command, .control, .option]).isEmpty { return nil }
        if key == .return { return "\r" }
        if key == .delete { return "\u{8}" }
        guard characters.count == 1, let c = characters.first,
              let ascii = c.asciiValue, (0x20...0x7e).contains(ascii) else { return nil }
        return c
    }
}

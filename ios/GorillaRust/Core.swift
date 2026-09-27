import Foundation

/// The Rust game core, through the C functions in gorilla.h. Call it only on
/// the main thread, where the core lives.
enum Core {
    struct Frame {
        let width: Int
        let height: Int
        let pixels: Data
    }

    @discardableResult
    static func start(seed: UInt64, muted: Bool) -> Bool { gr_start(seed, muted) }

    /// Run the game until it waits again. True if it presented a new frame.
    static func step() -> Bool { gr_step() }

    /// The last frame the game presented, as RGBA bytes.
    static func frame() -> Frame? {
        var width: UInt32 = 0
        var height: UInt32 = 0
        guard let pixels = gr_frame(&width, &height) else { return nil }
        let count = Int(width) * Int(height) * 4
        return Frame(width: Int(width), height: Int(height), pixels: Data(bytes: pixels, count: count))
    }

    static func push(_ key: Character) {
        for scalar in key.unicodeScalars { gr_push_key(scalar.value) }
    }

    static func setMuted(_ muted: Bool) { gr_set_muted(muted) }
    static func reopenAudio() { gr_reopen_audio() }
    static var version: String { String(cString: gr_version()) }
    static var lastError: String? { gr_last_error().map { String(cString: $0) } }
}

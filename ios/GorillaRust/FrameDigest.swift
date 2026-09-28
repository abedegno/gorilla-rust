import Foundation

/// A 64-bit FNV-1a hash of a frame's bytes, exposed to UI tests as the game
/// view's accessibility value so they can tell when the screen changes.
enum FrameDigest {
    static func of(_ data: Data) -> UInt64 {
        var hash: UInt64 = 0xcbf2_9ce4_8422_2325
        for byte in data {
            hash ^= UInt64(byte)
            hash = hash &* 0x0000_0100_0000_01b3
        }
        return hash
    }
}

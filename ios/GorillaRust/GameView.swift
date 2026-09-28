import SwiftUI
import UIKit

/// Shows the game's frame with square, sharp pixels, and steps the game
/// once per display refresh.
final class FrameView: UIView {
    private var link: CADisplayLink?
    var onError: ((String) -> Void)?

    /// Whether to hash each frame into the accessibility value, which only
    /// the UI tests read. Hashing a megabyte every frame costs battery, so
    /// it is off unless a test launches the app with -UITestFrameDigest.
    static let exposesDigest = ProcessInfo.processInfo.arguments.contains("-UITestFrameDigest")

    override init(frame: CGRect) {
        super.init(frame: frame)
        backgroundColor = .black
        layer.magnificationFilter = .nearest
        layer.contentsGravity = .resizeAspect
        isAccessibilityElement = true
        accessibilityIdentifier = "game"
        accessibilityLabel = "Game screen"
    }

    required init?(coder: NSCoder) { fatalError("FrameView is made in code") }

    func start() {
        guard link == nil else { return }
        let link = CADisplayLink(target: self, selector: #selector(tick))
        // The game's waits keep their pace at any frame rate, and a 1990
        // game gains nothing from 120 frames a second but a flatter battery.
        link.preferredFrameRateRange = CAFrameRateRange(minimum: 30, maximum: 60, preferred: 60)
        link.add(to: .main, forMode: .common)
        self.link = link
    }

    func stop() {
        link?.invalidate()
        link = nil
    }

    @objc private func tick() {
        if Core.step(), let frame = Core.frame() {
            show(frame)
        } else if let error = Core.lastError {
            stop()
            onError?(error)
        }
    }

    private func show(_ frame: Core.Frame) {
        if Self.exposesDigest {
            accessibilityValue = String(FrameDigest.of(frame.pixels), radix: 16)
        }
        guard let provider = CGDataProvider(data: frame.pixels as CFData),
              let image = CGImage(
                  width: frame.width, height: frame.height,
                  bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: frame.width * 4,
                  space: CGColorSpaceCreateDeviceRGB(),
                  bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.noneSkipLast.rawValue),
                  provider: provider, decode: nil, shouldInterpolate: false, intent: .defaultIntent)
        else { return }
        layer.contents = image
    }
}

/// The game screen, stepping while `running`.
struct GameView: UIViewRepresentable {
    let running: Bool
    let onError: (String) -> Void

    func makeUIView(context: Context) -> FrameView {
        let view = FrameView(frame: .zero)
        view.onError = onError
        return view
    }

    func updateUIView(_ view: FrameView, context: Context) {
        if running { view.start() } else { view.stop() }
    }

    static func dismantleUIView(_ view: FrameView, coordinator: ()) {
        view.stop()
    }
}

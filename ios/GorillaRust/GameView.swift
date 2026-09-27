import SwiftUI
import UIKit

/// Shows the game's frame with square, sharp pixels, and steps the game
/// once per display refresh.
final class FrameView: UIView {
    private var link: CADisplayLink?
    var onError: ((String) -> Void)?

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
        accessibilityValue = String(FrameDigest.of(frame.pixels), radix: 16)
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

import XCTest

/// App Store screenshots: the intro, the players' choice, and a game, in
/// portrait and landscape. Runs only when ios/screenshots.sh sets
/// SCREENSHOTS_DIR.
final class ScreenshotUITests: XCTestCase {
    func testCaptureTheScreens() throws {
        guard let dir = ProcessInfo.processInfo.environment["SCREENSHOTS_DIR"] else {
            throw XCTSkip("screenshots are taken by ios/screenshots.sh")
        }
        let app = XCUIApplication()
        app.launch()
        let device = UIDevice.current.userInterfaceIdiom == .pad ? "ipad" : "iphone"
        func shot(_ name: String) {
            Thread.sleep(forTimeInterval: 1.5)
            let url = URL(fileURLWithPath: dir).appendingPathComponent("\(device)-\(name).png")
            // A screenshot comes in the screen's native portrait shape with a
            // rotation flag; App Store Connect goes by the pixels, so draw it
            // upright first and a landscape shot is landscape.
            let image = XCUIScreen.main.screenshot().image
            let format = UIGraphicsImageRendererFormat()
            format.scale = image.scale
            let upright = UIGraphicsImageRenderer(size: image.size, format: format).pngData { _ in
                image.draw(in: CGRect(origin: .zero, size: image.size))
            }
            try? upright.write(to: url)
        }
        for orientation in [UIDeviceOrientation.landscapeLeft, .portrait] {
            XCUIDevice.shared.orientation = orientation
            let tag = orientation == .portrait ? "portrait" : "landscape"
            shot("1-intro-\(tag)")
        }
        XCUIDevice.shared.orientation = .landscapeLeft
        app.buttons["1"].tap()                        // end the intro
        for _ in 0..<4 { app.buttons["Enter"].tap() } // default names, points and gravity
        shot("2-choice-landscape")
        app.buttons["P"].tap()                        // play
        shot("3-game-landscape")
    }
}

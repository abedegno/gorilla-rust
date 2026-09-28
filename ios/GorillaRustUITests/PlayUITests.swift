import XCTest

final class PlayUITests: XCTestCase {
    /// Whether the game view's frame digest holds still for `stable` seconds
    /// at some point before `limit`.
    private func settles(_ game: XCUIElement, for stable: TimeInterval, within limit: TimeInterval) -> Bool {
        let end = Date().addingTimeInterval(limit)
        var last = game.value as? String
        var since = Date()
        while Date() < end {
            Thread.sleep(forTimeInterval: 0.1)
            let now = game.value as? String
            if now != last {
                last = now
                since = Date()
            } else if Date().timeIntervalSince(since) >= stable {
                return true
            }
        }
        return false
    }

    func testAKeyOnTheKeypadEndsTheIntroEvenAfterRotating() {
        let app = XCUIApplication()
        // The game view only publishes its frame digest when asked.
        app.launchArguments = ["-UITestFrameDigest"]
        app.launch()
        let game = app.otherElements["game"]
        XCTAssertTrue(game.waitForExistence(timeout: 10))

        // The intro's border sparkles, so its frame never holds still.
        XCTAssertFalse(settles(game, for: 1.0, within: 3.0), "the intro should be animating")

        XCUIDevice.shared.orientation = .landscapeLeft
        app.buttons["1"].tap()

        // The first name prompt waits for typing: nothing on it moves.
        XCTAssertTrue(settles(game, for: 1.5, within: 8.0), "the key should reach the game")
        XCUIDevice.shared.orientation = .portrait
    }
}

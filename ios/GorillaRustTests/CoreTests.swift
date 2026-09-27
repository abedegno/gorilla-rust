import XCTest
@testable import GorillaRust

final class CoreTests: XCTestCase {
    func testReportsTheCrateVersion() {
        XCTAssertNotNil(Core.version.range(of: #"^\d+\.\d+\.\d+$"#, options: .regularExpression))
    }

    func testTheAppStartedTheCoreOnce() {
        // The host app started it at launch; a second start is refused.
        XCTAssertFalse(Core.start(seed: 1, muted: true))
    }

    func testSteppingPresentsTheIntroOnTheTextScreen() {
        // The host app's display link steps the core too, so this reads
        // the frame itself rather than relying on its own step being the
        // one that presented it.
        var frame: Core.Frame?
        for _ in 0..<600 where frame == nil {
            _ = Core.step()
            if let f = Core.frame(), f.height == 400 { frame = f }
            RunLoop.current.run(until: Date().addingTimeInterval(0.01))
        }
        XCTAssertEqual(frame?.width, 640)
        XCTAssertEqual(frame?.height, 400)
    }
}

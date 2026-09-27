import XCTest

final class LaunchUITests: XCTestCase {
    func testTheAppLaunchesToTheGame() {
        let app = XCUIApplication()
        app.launch()
        XCTAssertTrue(app.otherElements["game"].waitForExistence(timeout: 10))
    }
}

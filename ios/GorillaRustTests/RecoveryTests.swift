import SwiftUI
import XCTest
@testable import GorillaRust

final class AudioRecoveryTests: XCTestCase {
    func testReopensWhenComingBackFromTheBackground() {
        // iOS does not always say an interruption ended when a call sends
        // the app to the background, so coming back is the moment to reopen.
        XCTAssertTrue(AudioRecovery.shouldReopen(from: .background, to: .active))
    }

    func testLeavesSoundAloneForAGlanceAtControlCentre() {
        XCTAssertFalse(AudioRecovery.shouldReopen(from: .inactive, to: .active))
        XCTAssertFalse(AudioRecovery.shouldReopen(from: .active, to: .inactive))
    }
}

final class FrameDigestSwitchTests: XCTestCase {
    func testTheDigestIsOnlyWorkedOutForUITests() {
        // Hashing every frame costs battery; only the UI tests read it, and
        // they ask for it with a launch argument.
        XCTAssertFalse(FrameView.exposesDigest)
    }
}

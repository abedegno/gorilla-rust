import AVFoundation
import XCTest
@testable import GorillaRust

final class AudioSessionPolicyTests: XCTestCase {
    func testSoundOnPlaysThroughSilentMode() {
        XCTAssertEqual(AudioSessionPolicy.category(muted: false), .playback)
    }

    func testMutedLeavesOtherAppsAudioAlone() {
        XCTAssertEqual(AudioSessionPolicy.category(muted: true), .ambient)
    }
}

final class MuteStoreTests: XCTestCase {
    func testRemembersTheChoice() {
        let defaults = UserDefaults(suiteName: "MuteStoreTests-\(UUID().uuidString)")!
        var store = MuteStore(defaults: defaults)
        XCTAssertFalse(store.muted, "sound is on at first")
        store.muted = true
        XCTAssertTrue(MuteStore(defaults: defaults).muted)
    }
}

import XCTest
@testable import GorillaRust

final class FrameDigestTests: XCTestCase {
    func testMatchesTheFNV1aReferenceValues() {
        XCTAssertEqual(FrameDigest.of(Data()), 0xcbf2_9ce4_8422_2325)
        XCTAssertEqual(FrameDigest.of(Data("a".utf8)), 0xaf63_dc4c_8601_ec8c)
    }
}

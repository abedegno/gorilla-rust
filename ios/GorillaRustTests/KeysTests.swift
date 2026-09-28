import XCTest
@testable import GorillaRust

final class KeysTests: XCTestCase {
    private func labels(_ rows: [[PadKey]]) -> [String] { rows.flatMap { $0.map(\.label) } }

    func testTheDigitsLayoutMatchesTheWebPage() {
        XCTAssertEqual(labels(Keys.digits),
                       ["7", "8", "9", "⌫", "4", "5", "6", "V", "1", "2", "3", "P", "0", ".", "ABC", "⏎"])
    }

    func testEnterAndBackspaceSendWhatTheWebPageSends() {
        var state = PadState()
        XCTAssertEqual(state.press(PadKey(label: "⏎", action: .send("\r"))), "\r")
        XCTAssertEqual(state.press(PadKey(label: "⌫", action: .send("\u{8}"))), "\u{8}")
    }

    func testShiftCapitalisesOneLetterThenLetsGo() {
        var state = PadState()
        XCTAssertNil(state.press(PadKey(label: "⇧", action: .shift)))
        XCTAssertEqual(state.press(PadKey(label: "a", action: .send("a"))), "A")
        XCTAssertEqual(state.press(PadKey(label: "b", action: .send("b"))), "b")
    }

    func testShiftLeavesDigitsAlone() {
        var state = PadState()
        _ = state.press(PadKey(label: "⇧", action: .shift))
        XCTAssertEqual(state.press(PadKey(label: "1", action: .send("1"))), "1")
    }

    func testSwitchingLayoutsSendsNothing() {
        var state = PadState()
        XCTAssertNil(state.press(PadKey(label: "ABC", action: .switchTo(.letters))))
        XCTAssertEqual(state.layout, .letters)
    }

    func testTheLettersShowCapitalsWhileShifted() {
        XCTAssertTrue(labels(Keys.letters(shifted: true)).contains("Q"))
        XCTAssertTrue(labels(Keys.letters(shifted: false)).contains("q"))
    }
}

import SwiftUI
import XCTest
@testable import GorillaRust

final class HardwareKeysTests: XCTestCase {
    func testReturnDeleteAndCharacters() {
        XCTAssertEqual(HardwareKeys.character(for: .return, characters: "\r", modifiers: []), "\r")
        XCTAssertEqual(HardwareKeys.character(for: .delete, characters: "", modifiers: []), "\u{8}")
        XCTAssertEqual(HardwareKeys.character(for: KeyEquivalent("x"), characters: "x", modifiers: []), "x")
        XCTAssertEqual(HardwareKeys.character(for: KeyEquivalent("5"), characters: "5", modifiers: []), "5")
    }

    func testChordsAndUnprintableKeysAreIgnored() {
        XCTAssertNil(HardwareKeys.character(for: KeyEquivalent("c"), characters: "c", modifiers: .command))
        XCTAssertNil(HardwareKeys.character(for: .upArrow, characters: "", modifiers: []))
        XCTAssertNil(HardwareKeys.character(for: KeyEquivalent("é"), characters: "é", modifiers: []))
    }
}

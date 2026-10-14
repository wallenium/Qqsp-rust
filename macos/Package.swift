// swift-tools-version:6.0
// Liquid Glass benötigt macOS 26 SDK (Xcode 26). Vorher `./build-core.sh` ausführen.
import PackageDescription

let package = Package(
    name: "Qqsp",
    platforms: [.macOS("26.0")],
    targets: [
        .target(name: "CQsp", path: "Sources/CQsp"),
        .executableTarget(
            name: "Qqsp",
            dependencies: ["CQsp"],
            path: "Sources/Qqsp",
            swiftSettings: [.swiftLanguageMode(.v5)],
            linkerSettings: [.unsafeFlags(["-L", "lib", "-lqsp_core"])]
        ),
        .testTarget(
            name: "QqspTests",
            dependencies: ["Qqsp"],
            path: "Tests/QqspTests",
            swiftSettings: [.swiftLanguageMode(.v5)],
            linkerSettings: [.unsafeFlags(["-L", "lib", "-lqsp_core"])]
        ),
    ]
)

//
//  ContentView.swift
//  HelloWorld
//
//  Created by Antonius Naumann on 06.04.23.
//

import SwiftUI
import Greeter

struct ContentView: View {
    var body: some View {
        VStack {
            Text(greet(name: "World"))
            Text(platformDescription)
        }
    }

    var platformName: String {
        switch currentPlatform() {
        case .ios: return "iOS"
        case .macOs: return "macOS"
        }
    }

    var platformDescription: String {
        // Types and functions that only exist on one platform must be guarded with #if,
        // just like the #[cfg(target_os = ...)] attributes on the Rust side.
        #if os(iOS)
        let pattern = defaultHapticPattern()
        return "\(platformName): haptic intensity \(pattern.intensity), \(pattern.durationMs) ms"
        #elseif os(macOS)
        let item = MenuBarItem(title: "Greeter")
        switch item.badge(unread: 3) {
        case .none:
            return "\(platformName): \(item.title())"
        case .count(let value):
            return "\(platformName): \(item.title()) (\(value) unread)"
        }
        #endif
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}

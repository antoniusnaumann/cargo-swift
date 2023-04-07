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
        Text(greet(name: "World"))
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}

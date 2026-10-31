// necro_notch.swift: a dark-glass pill that docks under the MacBook notch.
//
// Drop a .cob or .ch8 file on it (or press "Resurrect sample") and it sends the file to the
// NecroCore daemon on 127.0.0.1:4300. The daemon saves the file under samples/dropped/ and runs the
// command from .kiro/hooks/on-legacy-drop.json. The pill then shows the Ghost, Vampire and Zombie
// counts before and after healing and opens the healed result in the Observatory.
//
// Build:  swiftc -O apps/macos-notch/necro_notch.swift -o apps/macos-notch/necro_notch
// Run:    apps/macos-notch/necro_notch
// Check without the window:
//         apps/macos-notch/necro_notch --sample PAYROLL-BATCH-1982 [--snapshot out.png]
//         apps/macos-notch/necro_notch --drop path/to/FILE.cob     [--snapshot out.png]
import AppKit
import Foundation

let base = "http://127.0.0.1:4300"
let necroRoot = ProcessInfo.processInfo.environment["NECROCORE_ROOT"] ?? NSHomeDirectory() + "/NecroCore"
let sampleNames = ["PAYROLL-BATCH-1982", "BANK-INTEREST-1978", "WIRE-CLEARING-1985"]

// MARK: daemon client

func request(_ path: String, body: [String: Any]?, timeout: TimeInterval, _ done: @escaping ([String: Any]?, String?) -> Void) {
    guard let url = URL(string: base + path) else { return done(nil, "bad url") }
    var req = URLRequest(url: url, timeoutInterval: timeout)
    if let body = body {
        req.httpMethod = "POST"
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try? JSONSerialization.data(withJSONObject: body)
    }
    URLSession.shared.dataTask(with: req) { data, _, err in
        if let err = err { return done(nil, (err as NSError).code == NSURLErrorCannotConnectToHost ? "offline" : err.localizedDescription) }
        guard let data = data, let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return done(nil, "unreadable reply") }
        if let e = obj["error"] as? String { return done(nil, e) }
        done(obj, nil)
    }.resume()
}

/// Starts the daemon from the checkout when nothing answers on port 4300.
func ensureDaemon(_ done: @escaping (Bool) -> Void) {
    request("/api/bootstrap", body: nil, timeout: 2) { reply, _ in
        if reply != nil { return done(true) }
        let p = Process()
        p.executableURL = URL(fileURLWithPath: necroRoot + "/target/release/necro-cli")
        p.arguments = ["observatory", "--serve", "--root", necroRoot]
        p.standardOutput = FileHandle.nullDevice
        p.standardError = FileHandle.nullDevice
        do { try p.run() } catch { return done(false) }
        func poll(_ left: Int) {
            request("/api/bootstrap", body: nil, timeout: 2) { r, _ in
                if r != nil { done(true) } else if left > 0 { DispatchQueue.global().asyncAfter(deadline: .now() + 0.4) { poll(left - 1) } } else { done(false) }
            }
        }
        DispatchQueue.global().asyncAfter(deadline: .now() + 0.6) { poll(20) }
    }
}

struct Outcome {
    var line: String
    var detail: String
    var url: URL?
    var ok: Bool
}

func summarize(_ r: [String: Any]) -> Outcome {
    let b = r["before"] as? [String: Any] ?? [:]
    let a = r["after"] as? [String: Any] ?? [:]
    func n(_ d: [String: Any], _ k: String) -> Int { (d[k] as? NSNumber)?.intValue ?? 0 }
    let name = r["name"] as? String ?? "program"
    let line = "\u{1F47B} \(n(b, "ghost"))   \u{1F9DB} \(n(b, "vampire"))   \u{1F9DF} \(n(b, "zombie"))   \u{2192}   \(n(a, "total")) left"
    var hookText = "hook did not run"
    if let h = r["hook"] as? [String: Any], (h["ran"] as? Bool) == true {
        let code = (h["exit_code"] as? NSNumber)?.intValue
        let ms = (h["ms"] as? NSNumber)?.intValue ?? 0
        hookText = code == 0 ? "on-legacy-drop hook ran in \(ms) ms" : "on-legacy-drop hook exit \(code.map(String.init) ?? "none")"
    }
    return Outcome(line: line, detail: "\(name): \(hookText)", url: (r["url"] as? String).flatMap(URL.init(string:)), ok: true)
}

func resurrect(file: URL?, sample: String?, _ done: @escaping (Outcome) -> Void) {
    var body: [String: Any] = [:]
    if let s = sample {
        body["sample"] = s
    } else if let f = file {
        guard let data = try? Data(contentsOf: f) else { return done(Outcome(line: "Cannot read \(f.lastPathComponent)", detail: "", url: nil, ok: false)) }
        body["filename"] = f.lastPathComponent
        body["b64"] = data.base64EncodedString()
    }
    ensureDaemon { up in
        guard up else { return done(Outcome(line: "Daemon not running", detail: "Build it with cargo build --release in \(necroRoot)", url: nil, ok: false)) }
        request("/api/drop", body: body, timeout: 200) { reply, err in
            if let reply = reply { done(summarize(reply)) } else { done(Outcome(line: "Daemon said no", detail: err ?? "", url: nil, ok: false)) }
        }
    }
}

// MARK: pill

final class PillView: NSVisualEffectView {
    var onDrop: (([URL]) -> Void)?
    private func files(_ info: NSDraggingInfo) -> [URL] {
        let urls = info.draggingPasteboard.readObjects(forClasses: [NSURL.self], options: [.urlReadingFileURLsOnly: true]) as? [URL] ?? []
        return urls.filter { ["cob", "ch8"].contains($0.pathExtension.lowercased()) }
    }
    private func glow(_ on: Bool) {
        layer?.borderColor = (on ? NSColor(calibratedRed: 0.2, green: 0.83, blue: 0.6, alpha: 1) : NSColor(white: 1, alpha: 0.14)).cgColor
        layer?.borderWidth = on ? 2 : 1
    }
    override func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation {
        if files(sender).isEmpty { return [] }
        glow(true)
        return .copy
    }
    override func draggingExited(_ sender: NSDraggingInfo?) { glow(false) }
    override func performDragOperation(_ sender: NSDraggingInfo) -> Bool {
        glow(false)
        let f = files(sender)
        if f.isEmpty { return false }
        onDrop?(f)
        return true
    }
}

final class NotchController: NSObject {
    let panel: NSPanel
    let pill = PillView()
    let title = NSTextField(labelWithString: "\u{26A1} NecroCore \u{00B7} Drop Dead Code to Resurrect")
    let result = NSTextField(labelWithString: "")
    let detail = NSTextField(labelWithString: "")
    let quick = NSButton(title: "Resurrect sample", target: nil, action: nil)
    let open = NSButton(title: "Observatory", target: nil, action: nil)
    let close = NSButton(title: "\u{2715}", target: nil, action: nil)
    var sampleIndex = 0
    var lastURL: URL?
    var collapse: DispatchWorkItem?
    let width: CGFloat = 520

    override init() {
        panel = NSPanel(contentRect: NSRect(x: 0, y: 0, width: 520, height: 46), styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
        super.init()
        panel.level = .statusBar
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = true
        panel.isMovableByWindowBackground = false
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary]
        panel.appearance = NSAppearance(named: .darkAqua)

        pill.material = .hudWindow
        pill.blendingMode = .behindWindow
        pill.state = .active
        pill.wantsLayer = true
        pill.layer?.cornerRadius = 20
        pill.layer?.masksToBounds = true
        pill.layer?.borderColor = NSColor(white: 1, alpha: 0.14).cgColor
        pill.layer?.borderWidth = 1
        pill.registerForDraggedTypes([.fileURL])
        pill.setAccessibilityLabel("NecroCore drop target. Drop a COBOL or CHIP-8 file to resurrect it.")
        pill.onDrop = { [weak self] urls in self?.run(file: urls[0], sample: nil) }

        title.font = .systemFont(ofSize: 13, weight: .semibold)
        title.textColor = .white
        title.lineBreakMode = .byTruncatingTail
        result.font = .monospacedSystemFont(ofSize: 13, weight: .medium)
        result.textColor = .white
        result.isHidden = true
        detail.font = .systemFont(ofSize: 11)
        detail.textColor = NSColor(white: 1, alpha: 0.7)
        detail.isHidden = true
        detail.lineBreakMode = .byTruncatingTail

        for b in [quick, open, close] {
            b.bezelStyle = .inline
            b.isBordered = true
            b.controlSize = .small
            b.contentTintColor = .white
            b.font = .systemFont(ofSize: 12, weight: .medium)
            b.target = self
            b.attributedTitle = NSAttributedString(string: b.title, attributes: [.foregroundColor: NSColor.white, .font: NSFont.systemFont(ofSize: 12, weight: .medium)])
        }
        quick.action = #selector(quickTapped)
        quick.setAccessibilityLabel("Resurrect a sample COBOL program")
        open.action = #selector(openTapped)
        open.setAccessibilityLabel("Open the Observatory in the browser")
        close.action = #selector(closeTapped)
        close.setAccessibilityLabel("Quit the NecroCore notch")

        let top = NSStackView(views: [title, NSView(), quick, open, close])
        top.orientation = .horizontal
        top.spacing = 8
        top.alignment = .centerY
        let col = NSStackView(views: [top, result, detail])
        col.orientation = .vertical
        col.alignment = .leading
        col.spacing = 3
        col.translatesAutoresizingMaskIntoConstraints = false
        pill.addSubview(col)
        NSLayoutConstraint.activate([
            col.leadingAnchor.constraint(equalTo: pill.leadingAnchor, constant: 16),
            col.trailingAnchor.constraint(equalTo: pill.trailingAnchor, constant: -12),
            col.centerYAnchor.constraint(equalTo: pill.centerYAnchor),
            top.trailingAnchor.constraint(equalTo: col.trailingAnchor),
        ])
        panel.contentView = pill
        place(height: 46)
    }

    func place(height: CGFloat) {
        let screen = NSScreen.main ?? NSScreen.screens[0]
        let inset = screen.safeAreaInsets.top > 0 ? screen.safeAreaInsets.top : 24
        let x = screen.frame.midX - width / 2
        let y = screen.frame.maxY - inset - height - 4
        panel.setFrame(NSRect(x: x, y: y, width: width, height: height), display: true, animate: false)
    }

    func show() { panel.orderFrontRegardless() }

    func busy(_ text: String) {
        collapse?.cancel()
        result.isHidden = false
        result.stringValue = text
        detail.isHidden = true
        place(height: 62)
    }

    func present(_ o: Outcome) {
        result.isHidden = false
        result.stringValue = o.line
        result.textColor = o.ok ? .white : NSColor(calibratedRed: 1, green: 0.55, blue: 0.55, alpha: 1)
        detail.isHidden = false
        detail.stringValue = o.detail
        lastURL = o.url
        place(height: 78)
        let w = DispatchWorkItem { [weak self] in
            self?.result.isHidden = true
            self?.detail.isHidden = true
            self?.place(height: 46)
        }
        collapse = w
        DispatchQueue.main.asyncAfter(deadline: .now() + 14, execute: w)
    }

    func run(file: URL?, sample: String?, completion: ((Outcome) -> Void)? = nil) {
        busy("Resurrecting \(file?.lastPathComponent ?? sample ?? "")\u{2026}")
        resurrect(file: file, sample: sample) { [weak self] o in
            DispatchQueue.main.async {
                self?.present(o)
                if o.ok, let u = o.url { NSWorkspace.shared.open(u) }
                completion?(o)
            }
        }
    }

    @objc func quickTapped() {
        let s = sampleNames[sampleIndex % sampleNames.count]
        sampleIndex += 1
        run(file: nil, sample: s)
    }
    @objc func openTapped() { NSWorkspace.shared.open(lastURL ?? URL(string: base + "/")!) }
    @objc func closeTapped() { NSApp.terminate(nil) }

    func snapshot(to path: String) {
        pill.layoutSubtreeIfNeeded()
        guard let rep = pill.bitmapImageRepForCachingDisplay(in: pill.bounds) else { return }
        pill.cacheDisplay(in: pill.bounds, to: rep)
        if let png = rep.representation(using: .png, properties: [:]) { try? png.write(to: URL(fileURLWithPath: path)) }
    }
}

// MARK: entry

let args = CommandLine.arguments
func value(after flag: String) -> String? { args.firstIndex(of: flag).flatMap { $0 + 1 < args.count ? args[$0 + 1] : nil } }

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let controller = NotchController()

if let s = value(after: "--sample") ?? value(after: "--drop") {
    // Headless check: same code path as a real drop, no window, no browser.
    let isSample = value(after: "--sample") != nil
    controller.busy("Resurrecting\u{2026}")
    resurrect(file: isSample ? nil : URL(fileURLWithPath: s), sample: isSample ? s : nil) { o in
        DispatchQueue.main.async {
            controller.present(o)
            print(o.line)
            print(o.detail)
            print(o.url?.absoluteString ?? "no url")
            if let p = value(after: "--snapshot") { controller.snapshot(to: p) }
            exit(o.ok ? 0 : 1)
        }
    }
} else {
    controller.show()
}
app.run()

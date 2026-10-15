# Σ Sigma — High-Performance Vector Design Engine

> A blazing fast, memory-safe, open-source alternative to Figma built in **Rust** with **WebAssembly & WebGPU/WebGL**.

🌐 **Live Demo (Multiplayer):** [https://sigma-sync.bhavuk-arora03.workers.dev](https://sigma-sync.bhavuk-arora03.workers.dev)

Sigma is engineered from the ground up for extreme performance:
* **Real-time Multiplayer:** Live multi-cursor presence, remote selection tags, and real-time document synchronization running on Cloudflare Workers & Durable Objects at the edge.
* **Web First:** Compiles directly to WebAssembly (`wasm32-unknown-unknown`) running in any modern browser at 60–120 FPS.
* **Native Desktop:** Compiles into native standalone applications for **macOS (Metal)**, **Windows (DX12/Vulkan)**, and **Linux (Vulkan)** from the exact same codebase.
* **Figma Pro Features:** Infinite canvas, Auto Layout (Flexbox flow), Scene Graph hierarchy, 8-handle transformation boxes, real-time property inspector, SVG export, and live CSS Dev Mode.

---

## 🏗 Architecture

```
sigma/
├── Cargo.toml                 # Cargo workspace configuration
├── index.html                 # Web entry point for Trunk
├── Trunk.toml                 # WebAssembly build & dev server config
└── crates/
    ├── sigma-core/            # Pure, platform-agnostic engine
    │   ├── src/
    │   │   ├── math.rs        # 2D affine transforms, Vec2, Rect
    │   │   ├── color.rs       # Hex parsing/serialization, RGBA color math
    │   │   ├── style.rs       # Fills, Strokes, Effects, Corner Radii
    │   │   ├── layout.rs      # Auto Layout engine (Direction, Gap, Padding, Sizing)
    │   │   ├── node.rs        # Scene graph nodes (Frame, Rect, Ellipse, Text)
    │   │   ├── document.rs    # Document tree, hit-testing, alignment
    │   │   ├── history.rs     # Undo / Redo snapshot history stack
    │   │   └── export.rs      # Vector SVG generator & CSS Dev Mode exporter
    │
    └── sigma-app/             # Application shell & UI Chrome
        ├── src/
        │   ├── canvas.rs      # Infinite GPU canvas, coordinate math, rendering
        │   ├── sidebar.rs     # Layers hierarchy tree (visibility, lock, z-order)
        │   ├── inspector.rs   # Design inspector (Transform, Auto Layout, Fills)
        │   ├── toolbar.rs     # Tools (V, F, R, O, T, H), Zoom controls, Export
        │   ├── app.rs         # Event loop, keyboard shortcuts, input management
        │   ├── lib.rs         # WebAssembly entry point (`wasm-bindgen`)
        │   └── main.rs        # Native desktop entry point (`eframe::run_native`)
```

---

## 🚀 Quick Start

### Prerequisites
* Rust 1.95+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
* WebAssembly target: `rustup target add wasm32-unknown-unknown`
* Trunk (Wasm bundler): `brew install trunk` or `cargo install trunk`

### 1. Run in the Browser (Web)
```bash
# Start local dev server with hot reload
trunk serve
```
Open [http://127.0.0.1:8080](http://127.0.0.1:8080) in your browser.

### 2. Run as Native Desktop App (macOS, Windows, Linux)
```bash
# Run natively on your OS
cargo run -p sigma-app --release
```

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Tool / Action |
| :--- | :--- |
| `V` | Move / Selection tool |
| `F` | Frame tool (artboard) |
| `R` | Rectangle tool |
| `O` | Ellipse / Circle tool |
| `T` | Text tool |
| `H` / `Space + Drag` | Hand / Pan tool |
| `Cmd/Ctrl + Z` | Undo |
| `Cmd/Ctrl + Shift + Z` | Redo |
| `Cmd/Ctrl + D` | Duplicate selected layer |
| `Delete` / `Backspace` | Delete selected layer |
| `Mouse Wheel` / `Trackpad` | Smooth pan & zoom to cursor |

---

## 🗺 Roadmap (Learning & Open-Source Evolution)

1. [x] Infinite Canvas with GPU-accelerated pan & zoom
2. [x] Document Model, Scene Graph, and Selection Bounding Boxes
3. [x] Auto Layout foundation (Direction, Gap, Padding)
4. [x] SVG & CSS Code generation
5. [ ] **Vector Networks:** Graph-based Bézier path editor (Figma's signature vector model)
6. [ ] **Multiplayer CRDT:** Real-time tree synchronization using [Loro](https://github.com/loro-dev/loro)
7. [ ] **Component System:** Master components, variants, and override trees
8. [ ] **Interactive Prototyping:** Smart Animate and transition state machines

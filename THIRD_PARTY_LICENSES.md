# Third-Party Licenses

This document lists all third-party dependencies used by Mandelrust and their respective licenses.

## License Summary

This project uses dependencies under the following licenses:

- **Apache-2.0**: Apache License 2.0
- **MIT**: MIT License
- **BSD-2-Clause**: BSD 2-Clause "Simplified" License
- **BSD-3-Clause**: BSD 3-Clause "New" or "Revised" License
- **Unicode-3.0**: Unicode License Agreement - Data Files and Software
- **ISC**: ISC License
- **BSL-1.0**: Boost Software License 1.0
- **MPL-2.0**: Mozilla Public License 2.0
- **Zlib**: zlib License
- **Unlicense**: The Unlicense (Public Domain)
- **OFL-1.1**: SIL Open Font License 1.1
- **LGPL-2.1-or-later**: GNU Lesser General Public License v2.1 or later

Most dependencies are dual-licensed under Apache-2.0 OR MIT, providing flexibility in license choice.

## Direct Dependencies

These are the crates explicitly listed in `Cargo.toml`:

| Package | Version | License |
|---------|---------|---------|
| rayon | 1.8 | Apache-2.0 OR MIT |
| serde | 1.0 | Apache-2.0 OR MIT |
| serde_json | 1.0 | Apache-2.0 OR MIT |
| once_cell | 1.19 | Apache-2.0 OR MIT |
| eframe | 0.25 | Apache-2.0 OR MIT |
| directories | 5.0 | Apache-2.0 OR MIT |
| image | 0.24 | Apache-2.0 OR MIT |
| rfd | 0.12 | MIT |

## Transitive Dependencies by License Type

### Apache-2.0 OR MIT (Dual Licensed)

The majority of dependencies use this dual-license combination, allowing users to choose either license:

- accesskit, accesskit_consumer, accesskit_macos, accesskit_unix, accesskit_windows
- ahash, arboard, arrayvec, as-raw-xcb-connection
- async-broadcast, async-channel, async-executor, async-fs, async-io, async-lock, async-once-cell, async-process, async-recursion, async-signal, async-task, async-trait
- atomic-waker, atspi, atspi-common, atspi-connection, atspi-proxies, autocfg
- bit_field, bitflags, block-buffer, blocking, bumpalo
- byteorder, cc, cesu8, cfg-expr, cfg-if, cgl, cocoa, cocoa-foundation
- concurrent-queue, core-foundation, core-foundation-sys, core-graphics, core-graphics-types
- cpufeatures, crc32fast, crossbeam-deque, crossbeam-epoch, crossbeam-utils, crypto-common
- derivative, digest, directories, dirs-sys, dispatch2, displaydoc, downcast-rs
- ecolor, eframe, egui, egui_glow, egui-winit, either, emath, enumflags2, enumflags2_derive
- equivalent, errno, event-listener, event-listener-strategy
- fastrand, fdeflate, find-msvc-tools, flate2, foreign-types, foreign-types-macros, foreign-types-shared, form_urlencoded
- futures-core, futures-io, futures-lite, futures-sink, futures-task, futures-util
- generic-array, getrandom, gif, half, hashbrown, heck, hermit-abi, hex, home
- idna, idna_adapter, image, indexmap, io-lifetimes, itoa
- jni, jni-sys, jobserver, jpeg-decoder, libc, lock_api, log
- memmap2, miniz_oxide, ndk, ndk-context, ndk-sys, nohash-hasher, num-traits
- objc2-app-kit, objc2-core-foundation, objc2-core-graphics, objc2-foundation, objc2-io-surface
- once_cell, ordered-stream, parking, parking_lot, parking_lot_core, paste, percent-encoding
- pin-project-lite, pin-utils, piper, pkg-config, png, polling
- proc-macro2, proc-macro-crate, qoi, quote
- rand, rand_chacha, rand_core, rayon, rayon-core
- regex, regex-automata, regex-syntax
- rustversion, scoped-tls, scopeguard
- serde, serde_core, serde_derive, serde_json, serde_repr, serde_spanned
- sha1, shlex, signal-hook-registry, smallvec, smol_str, socket2, stable_deref_trait, static_assertions
- syn, system-deps, tempfile, thiserror, thiserror-impl
- toml, toml_datetime, toml_edit, ttf-parser, typenum
- url, utf8_iter, version_check, waker-fn
- wasm-bindgen, wasm-bindgen-futures, wasm-bindgen-macro, wasm-bindgen-macro-support, wasm-bindgen-shared
- web-sys, web-time, webbrowser, weezl
- winapi, winapi-i686-pc-windows-gnu, winapi-x86_64-pc-windows-gnu
- windows, windows-implement, windows-interface, windows-link, windows-sys, windows-targets
- windows_aarch64_gnullvm, windows_aarch64_msvc, windows_i686_gnu, windows_i686_gnullvm, windows_i686_msvc
- windows_x86_64_gnu, windows_x86_64_gnullvm, windows_x86_64_msvc
- x11rb, x11rb-protocol

### Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT

- io-lifetimes, linux-raw-sys, rustix, wasi, wasip2, wit-bindgen

### Apache-2.0

- ab_glyph, ab_glyph_rasterizer, accesskit_winit
- gethostname, gl_generator, glutin, glutin_egl_sys, glutin_glx_sys, glutin_wgl_sys
- khronos_api, owned_ttf_parser, winit

### MIT

- android-properties, atk-sys, block, block2, block-sys, bytes
- cairo-sys-rs, calloop, calloop-wayland-source, cfg_aliases, combine, crunchy
- dispatch, dlib, gdk-pixbuf-sys, gdk-sys, gio-sys, glib-sys, glutin-winit, gobject-sys, gtk-sys
- libredox, malloc_buf, memoffset, nix, objc, objc_id, objc2, objc2-encode, objc2-foundation, objc-foundation, objc-sys
- orbclient, pango-sys, redox_syscall, redox_users, rfd
- sctk-adwaita, slab, smithay-client-toolkit, smithay-clipboard, strict-num, synstructure
- tiff, tracing, tracing-attributes, tracing-core, uds_windows, version-compare
- wayland-backend, wayland-client, wayland-csd-frame, wayland-cursor, wayland-protocols, wayland-protocols-experimental
- wayland-protocols-misc, wayland-protocols-plasma, wayland-protocols-wlr, wayland-scanner, wayland-sys
- winnow, x11-dl, xcursor, xdg-home, xkbcommon-dl, xml-rs
- zbus, zbus_macros, zbus_names, zmij, zvariant, zvariant_derive, zvariant_utils

### MIT OR Unlicense

- aho-corasick, memchr, same-file, walkdir, winapi-util

### Unicode-3.0

- icu_collections, icu_locale_core, icu_normalizer, icu_normalizer_data, icu_properties, icu_properties_data
- icu_provider, litemap, potential_utf, tinystr, writeable
- yoke, yoke-derive, zerofrom, zerofrom-derive, zerotrie, zerovec, zerovec-derive

### BSD-3-Clause

- arrayref, exr, instant, lebe, tiny-skia, tiny-skia-path

### BSD-2-Clause

- num_enum, num_enum_derive

### BSL-1.0 (Boost Software License)

- clipboard-win, error-code

### ISC

- libloading

### MPL-2.0 (Mozilla Public License)

- option-ext

### Zlib

- slotmap

### Apache-2.0 OR MIT OR Zlib

- bytemuck, bytemuck_derive, cursor-icon, dispatch2, glow, raw-window-handle, xkeysym, zune-inflate

### Apache-2.0 OR BSD-2-Clause OR MIT

- zerocopy, zerocopy-derive

### Apache-2.0 OR BSD-3-Clause OR MIT

- num_enum, num_enum_derive

### (Apache-2.0 OR MIT) AND Unicode-3.0

- unicode-ident

### (Apache-2.0 OR MIT) AND OFL-1.1 AND LicenseRef-UFL-1.0

- epaint (includes embedded fonts)

### Apache-2.0 OR LGPL-2.1-or-later OR MIT

- r-efi

### Apache-2.0 WITH LLVM-exception

- target-lexicon

### 0BSD OR Apache-2.0 OR MIT

- adler2

## Notable License Details

### Font Licenses in epaint

The `epaint` crate includes embedded fonts and is therefore licensed under multiple licenses:
- Apache-2.0 OR MIT for the code
- OFL-1.1 (SIL Open Font License) for fonts
- UFL-1.0 (Ubuntu Font License) for fonts

### Unicode License

Several ICU (International Components for Unicode) related crates use the Unicode-3.0 license, which is permissive and OSI-approved.

### Public Domain (Unlicense)

Some dependencies offer the Unlicense option, effectively placing the code in the public domain where applicable.

## Compliance

To comply with the licenses of all dependencies:

1. **Apache-2.0**: Include the Apache License text and any NOTICE files from dependencies
2. **MIT**: Include the MIT License text and copyright notices
3. **BSD Licenses**: Include license text and copyright notices (no trademark grant)
4. **Unicode-3.0**: Include copyright notice and license terms
5. **Font Licenses (OFL/UFL)**: These apply to font files, not code - include appropriate notices if distributing fonts

## Full License Texts

The full text of the Apache License 2.0 and MIT License are included in the `LICENSE-APACHE` and `LICENSE-MIT` files in the root of this repository.

Other license texts can be found at:
- **BSD-2-Clause**: https://opensource.org/licenses/BSD-2-Clause
- **BSD-3-Clause**: https://opensource.org/licenses/BSD-3-Clause
- **Unicode-3.0**: https://www.unicode.org/license.txt
- **ISC**: https://opensource.org/licenses/ISC
- **BSL-1.0**: https://www.boost.org/LICENSE_1_0.txt
- **MPL-2.0**: https://www.mozilla.org/en-US/MPL/2.0/
- **Zlib**: https://opensource.org/licenses/Zlib
- **Unlicense**: https://unlicense.org/
- **OFL-1.1**: https://scripts.sil.org/OFL

## Generating This File

This file was generated using `cargo-license`:

```bash
cargo install cargo-license
cargo license --json > licenses.json
cargo license --do-not-bundle
```

Last updated: January 11, 2026

## overview
steps to be caried out are found in the plan*.md files.  If you have not context you should read through the existing context files to get an idea of where the project is going.  This project is heavily created through LLMs.  If during working on this project you spot a rule that might be helpful to include in the agents.md then please indicate this.

## working through a plan
- when working through a plan you should update progress and implementation notes
- if plan results in a feature being deprecated it should start a deprecation plan that will span at least one more feature (so if its marked for deprecation in plan v0.1.3 then plan v0.1.4 should be updated to reflect that removign the feature will be completed)

## Final to-do before each release
- only executed after the plan for the release is complete and you should ask if we're at the end stage
- summarize plan and add to release section in readme.md
- move old plan to old_plans
- use scripts in build_scripts to construct a release
- git commits. 

## Benchmarks
The benchmark results can now be referenced easily between versions. When you update to v0.1.4 or later, simply run:
```powershell
cargo bench --bench fractal_bench
```
And copy the relevant results into BENCHMARKS.md for comparison

## Adding New Colormaps

### Using the Coolors Parser Utility

1. **Get palette from coolors.co:**
   - Visit https://coolors.co/ and create or find a palette
   - Click the menu (three dots) → Export → XML
   - Copy the XML content

2. **Save the XML:**
   ```powershell
   # Create a temporary file with the palette XML
   # Example: build_scripts/my_palette.xml
   ```

3. **Generate the colormap JSON:**
   ```powershell
   python build_scripts/coolors_parser.py --pretty \
       -i build_scripts/my_palette.xml \
       -o src/colormaps/my_colormap.json
   ```

4. **Edit the generated JSON:**
   - Open `src/colormaps/my_colormap.json`
   - Change the `"name"` field to a user-friendly name (e.g., "Electric Neon")
   - The parser auto-generates a name from color names, but you should customize it

5. **Register the colormap in the code:**
   - Open `src/colorschemes_io.rs`
   - Find the `define_builtin_colormaps!` macro invocation (around line 60)
   - Add your colormap to the list following the pattern:
     ```rust
     "My Colormap Name" => MY_COLORMAP_JSON => "colormaps/my_colormap.json",
     ```
   - **That's it!** The macro automatically:
     - Creates the constant with `include_str!`
     - Adds it to the `load_builtin_colormap()` function
     - Adds it to the `is_builtin_colormap()` check
     - Adds it to the GUI dropdown list via `list_available_colormaps()`
   - No need to manually update multiple functions anymore!

6. **Test the colormap:**
   ```powershell
   cargo build --release
   .\target\release\forma-fractalis.exe
   ```
   - Select your new colormap from the dropdown
   - Verify colors look correct on a fractal

7. **Clean up:**
   ```powershell
   # Remove the temporary XML file if desired
   rm build_scripts/my_palette.xml
   ```

### Manual Colormap Creation

If you prefer to create colormaps manually without the parser:

1. **Create JSON file:** `src/colormaps/my_colormap.json`

2. **Structure:**
   ```json
   {
     "name": "Display Name",
     "gradient": [
       {"position": 0.0, "r": 255, "g": 0, "b": 0},
       {"position": 0.5, "r": 0, "g": 255, "b": 0},
       {"position": 1.0, "r": 0, "g": 0, "b": 255}
     ]
   }
   ```

3. **Guidelines:**
   - `position`: Must be between 0.0 and 1.0, sorted ascending
   - Include at least 2 gradient stops
   - RGB values: 0-255
   - More stops = smoother transitions

4. **Register the colormap (same as step 5 above):**
   - Add one line to `define_builtin_colormaps!` macro in `src/colorschemes_io.rs`
   - See "Using the Coolors Parser Utility" step 5 for details

### Colormap Best Practices

- **Name conventions:** Use descriptive names (e.g., "Ocean Depths", "Fire Storm")
- **Positions:** Evenly distribute for smooth gradients, or cluster for sharp transitions
- **Colors:** Choose colors with good contrast for visibility
- **Testing:** Test with different fractals (Mandelbrot, Julia) and iteration counts
- **Inspiration:** Paul Bourke's fractal gallery, nature photography, artwork
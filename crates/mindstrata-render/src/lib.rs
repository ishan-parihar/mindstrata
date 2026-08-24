//! Deterministic 2D visual rendering for Mindstrata (AP2 Phase 4 —
//! "Visual rendering" row).
//!
//! The renderer turns a [`World`] plus a list of agent markers into a pixel
//! image: terrain fills each cell, sites overlay a colored block, and agents
//! draw as outlined sprites on top. It is a **pure function of its inputs**:
//! no RNG, no wall-clock data, no external state — the same world + markers
//! always produce byte-identical PNG output. It is also strictly read-only:
//! the renderer never mutates a world, never runs inside the tick loop, and
//! has no call site in `mindstrata-sim`, so calibrated windows (golden
//! replays, snapshots) are structurally untouched.
//!
//! PNG encoding is delegated to the `image` crate (the `png` feature only),
//! which is deterministic for identical pixel input.

use image::{ImageFormat, Rgba, RgbaImage};
use mindstrata_core::id::EntityId;
use mindstrata_sim::world::{SiteKind, Terrain, World};
use std::io::Cursor;

/// An agent marker to draw on the map.
///
/// Kept as a small value type so the renderer stays decoupled from the
/// simulation's `AgentBundle` — the CLI (or any caller) converts agents
/// into these. `hue` selects a sprite color from the fixed 8-color palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderAgent {
    /// X coordinate on the world grid.
    pub x: i32,
    /// Y coordinate on the world grid.
    pub y: i32,
    /// Palette selector (0..8); clamped internally.
    pub hue: u8,
}

impl RenderAgent {
    /// Create a marker with a wrapped hue (safe for any `u8`).
    pub fn new(x: i32, y: i32, hue: u8) -> Self {
        Self { x, y, hue }
    }
}

/// Default cell size in pixels: a 16×16 world renders 192×192 px.
pub const DEFAULT_CELL_PIXELS: u32 = 12;

/// Agent sprite outline color (dark, visible on every terrain).
const OUTLINE: [u8; 3] = [0x21, 0x21, 0x21];

/// Terrain palette: one opaque RGB color per terrain kind.
fn terrain_color(terrain: Terrain) -> [u8; 3] {
    match terrain {
        Terrain::Grassland => [0x7C, 0xB3, 0x42],
        Terrain::Forest => [0x2E, 0x7D, 0x32],
        Terrain::Hill => [0xC8, 0xA2, 0x4A],
        Terrain::Mountain => [0x9E, 0x9E, 0x9E],
        Terrain::Water => [0x29, 0xB6, 0xF6],
        Terrain::Desert => [0xE6, 0xD6, 0x9C],
        Terrain::Swamp => [0x6D, 0x4C, 0x41],
    }
}

/// Site palette: one opaque RGB color per site kind (overlay blocks).
fn site_color(kind: SiteKind) -> [u8; 3] {
    match kind {
        SiteKind::House => [0xA1, 0x88, 0x7F],
        SiteKind::Farm => [0xF9, 0xA8, 0x25],
        SiteKind::Well => [0x4D, 0xD0, 0xE1],
        SiteKind::Market => [0xAB, 0x47, 0xBC],
        SiteKind::Temple => [0xEC, 0xEF, 0xF1],
        SiteKind::Barracks => [0x37, 0x47, 0x4F],
        SiteKind::Workshop => [0x8D, 0x6E, 0x63],
        SiteKind::Square => [0xD7, 0xCC, 0xC8],
        SiteKind::Prison => [0x45, 0x5A, 0x64],
        SiteKind::School => [0x79, 0x86, 0xCB],
    }
}

/// Agent sprite palette: 8 distinct hues (Material 500 series).
const AGENT_PALETTE: [[u8; 3]; 8] = [
    [0xE5, 0x39, 0x35], // red
    [0xFF, 0x98, 0x00], // orange
    [0xF7, 0x63, 0x97], // pink
    [0x8E, 0x24, 0xAA], // purple
    [0x39, 0x49, 0xAB], // indigo
    [0x1E, 0x88, 0xE5], // blue
    [0x00, 0x89, 0x7B], // teal
    [0x43, 0xA0, 0x47], // green
];

/// The rendered image: width, height, and an opaque RGBA pixel buffer.
pub struct RenderedImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row-major RGBA bytes (`width * height * 4`).
    pub rgba: Vec<u8>,
}

impl RenderedImage {
    /// Encode the buffer as a PNG in memory.
    ///
    /// Fails only if the underlying encoder rejects the image (impossible
    /// for our internally-consistent buffers), returning the encoder error.
    pub fn to_png(&self) -> Result<Vec<u8>, image::ImageError> {
        // Rebuild the image cell-by-cell from the raw buffer — no clone, and
        // no `Option`-returning constructor to unwrap.
        let img = RgbaImage::from_fn(self.width, self.height, |col, row| {
            let i = (row as usize * self.width as usize + col as usize) * 4;
            Rgba([
                self.rgba[i],
                self.rgba[i + 1],
                self.rgba[i + 2],
                self.rgba[i + 3],
            ])
        });
        let mut out = Vec::new();
        img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)?;
        Ok(out)
    }
}

/// Fill a rectangular block with an opaque color.
fn fill_block(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    x0: u32,
    y0: u32,
    block_w: u32,
    block_h: u32,
    color: [u8; 3],
) {
    for row_delta in 0..block_h {
        let row = y0 + row_delta;
        if row >= height {
            // caller pre-clamps; defensive guard only
            continue;
        }
        let row_base = row as usize * width as usize * 4;
        for col_delta in 0..block_w {
            let col = x0 + col_delta;
            if col >= width {
                // caller pre-clamps; defensive guard only (symmetric with row)
                continue;
            }
            let i = row_base + col as usize * 4;
            rgba[i] = color[0];
            rgba[i + 1] = color[1];
            rgba[i + 2] = color[2];
            rgba[i + 3] = 255;
        }
    }
}

/// Paint a filled circle centered on a cell's block (agent sprite).
///
/// `radius` is in pixels; the circle spans the cell block minus a 2px margin.
fn fill_circle(
    rgba: &mut [u8],
    image_width: u32,
    image_height: u32,
    center_x: u32,
    center_y: u32,
    radius: u32,
    color: [u8; 3],
) {
    let r2 = radius * radius;
    let r = radius as i64;
    for row_delta in -(r as i32)..=r as i32 {
        let row = center_y as i64 + row_delta as i64;
        if row < 0 || row >= image_height as i64 {
            continue;
        }
        let row_delta2 = (row_delta as i64) * (row_delta as i64);
        for col_delta in -(r as i32)..=r as i32 {
            let col = center_x as i64 + col_delta as i64;
            if col < 0 || col >= image_width as i64 {
                continue;
            }
            if (col_delta as i64) * (col_delta as i64) + row_delta2 <= r2 as i64 {
                let i = row as usize * image_width as usize * 4 + col as usize * 4;
                rgba[i] = color[0];
                rgba[i + 1] = color[1];
                rgba[i + 2] = color[2];
                rgba[i + 3] = 255;
            }
        }
    }
}

/// Render a world + agent markers into a raw RGBA buffer.
///
/// - Every world cell is a `cell_pixels × cell_pixels` block painted with its
///   terrain color.
/// - A site overlays its cell with a centered block in the site's color
///   (60% of the cell, so terrain shows around it).
/// - Each agent draws a filled outlined circle at its cell's center, on top
///   of terrain and sites.
///
/// Out-of-bounds agent positions are ignored. The result is fully
/// deterministic: identical inputs yield identical pixels.
pub fn render_world_rgba(world: &World, agents: &[RenderAgent], cell_pixels: u32) -> RenderedImage {
    let cell = cell_pixels.max(2);
    // PNG forbids zero dimensions — clamp so ANY world (even a degenerate
    // 0x0 one) still produces a valid image. The buffer is pre-filled with a
    // neutral dark backdrop so a degenerate world renders as a sensible
    // fallback (not transparent black); every real world overwrites all
    // pixels in the terrain pass.
    let image_width = (world.width * cell).max(1);
    let image_height = (world.height * cell).max(1);
    let mut rgba = vec![0x26u8; (image_width * image_height * 4) as usize];
    for px in rgba.as_chunks_mut::<4>().0 {
        px[3] = 255;
    }

    // 1. Terrain base.
    for y in 0..world.height {
        for x in 0..world.width {
            let tile = world.tile(x as i32, y as i32);
            let color = terrain_color(tile.map_or(Terrain::Grassland, |t| t.terrain));
            fill_block(
                &mut rgba,
                image_width,
                image_height,
                x * cell,
                y * cell,
                cell,
                cell,
                color,
            );
        }
    }

    // 2. Site overlays.
    for site in &world.sites {
        let Some(tile_pos) = site_position(world, site.id) else {
            continue;
        };
        let color = site_color(site.kind);
        let inset = (cell as f32 * 0.2) as u32;
        let side = cell - 2 * inset;
        if side == 0 {
            continue;
        }
        let x0 = tile_pos.0 * cell + inset;
        let y0 = tile_pos.1 * cell + inset;
        if x0 + side > image_width || y0 + side > image_height {
            continue;
        }
        fill_block(
            &mut rgba,
            image_width,
            image_height,
            x0,
            y0,
            side,
            side,
            color,
        );
    }

    // 3. Agent sprites on top.
    for agent in agents {
        if agent.x < 0
            || agent.y < 0
            || agent.x as u32 >= world.width
            || agent.y as u32 >= world.height
        {
            continue;
        }
        let cx = agent.x as u32 * cell + cell / 2;
        let cy = agent.y as u32 * cell + cell / 2;
        let radius = (cell / 2).saturating_sub(2).max(2);
        let color = AGENT_PALETTE[agent.hue as usize % AGENT_PALETTE.len()];
        fill_circle(&mut rgba, image_width, image_height, cx, cy, radius, color);
        // Thin outline ring for visibility on any terrain.
        fill_circle(
            &mut rgba,
            image_width,
            image_height,
            cx,
            cy,
            radius,
            OUTLINE,
        );
        // Re-paint the inner fill on top of the outline ring.
        fill_circle(
            &mut rgba,
            image_width,
            image_height,
            cx,
            cy,
            radius.saturating_sub(1),
            color,
        );
    }

    RenderedImage {
        width: image_width,
        height: image_height,
        rgba,
    }
}

/// Render a world + agent markers to PNG bytes.
///
/// Convenience wrapper over [`render_world_rgba`] + [`RenderedImage::to_png`].
pub fn render_world_png(
    world: &World,
    agents: &[RenderAgent],
    cell_pixels: u32,
) -> Result<Vec<u8>, image::ImageError> {
    render_world_rgba(world, agents, cell_pixels).to_png()
}

/// One frame of a replay: the world state plus agent markers at a tick.
///
/// Borrows both inputs so the caller (typically the CLI) can sample the
/// live simulation at a cadence without cloning state.
#[derive(Debug, Clone, Copy)]
pub struct ReplayFrame<'a> {
    /// World state at this tick.
    pub world: &'a World,
    /// Agent markers at this tick.
    pub agents: &'a [RenderAgent],
}

/// Render a sequence of world states as an animated GIF (AP2 Phase 5 —
/// "add replay visualizations").
///
/// Each [`ReplayFrame`] becomes one GIF frame: terrain + sites + agent
/// sprites, identical rendering to [`render_world_png`] per frame. The
/// result is a **deterministic function of its inputs** — the GIF encoder
/// is stateless per frame and the palette/frame order is fixed, so the
/// same frame sequence always produces byte-identical output (verified by
/// the determinism test below). Strictly read-only: no RNG, no wall-clock
/// data, no tick-loop call site — calibrated windows are untouched.
///
/// `frame_delay_ms` is the display time per frame (milliseconds; the GIF
/// format stores delays in 10ms units, so values are rounded down to the
/// nearest 10ms). `repeat` controls the loop: `Repeat::Infinite` loops the
/// animation (the default for replay viewers); pass `Repeat::Finite(0)` for
/// a single play-through.
///
/// Returns the encoded GIF bytes. Fails only if the encoder rejects a
/// frame (impossible for our internally-consistent buffers).
pub fn render_replay_gif(
    frames: &[ReplayFrame<'_>],
    cell_pixels: u32,
    frame_delay_ms: u32,
    repeat: image::codecs::gif::Repeat,
) -> Result<Vec<u8>, image::ImageError> {
    use image::codecs::gif::GifEncoder;
    use image::{Delay, Frame as GifFrame};

    // GIF delays are in 10ms units; clamp to the format's legal range
    // (1..=65535 centiseconds) so extreme inputs cannot produce an
    // unencodable frame.
    let delay_cs = (frame_delay_ms / 10).clamp(1, 65535);
    let delay = Delay::from_numer_denom_ms(delay_cs * 10, 1);

    let mut out = Vec::new();
    {
        let mut encoder = GifEncoder::new(&mut out);
        encoder.set_repeat(repeat)?;
        for frame in frames {
            let img = render_world_rgba(frame.world, frame.agents, cell_pixels);
            let rgba = RgbaImage::from_fn(img.width, img.height, |col, row| {
                let i = (row as usize * img.width as usize + col as usize) * 4;
                Rgba([
                    img.rgba[i],
                    img.rgba[i + 1],
                    img.rgba[i + 2],
                    img.rgba[i + 3],
                ])
            });
            encoder.encode_frame(GifFrame::from_parts(rgba, 0, 0, delay))?;
        }
    }
    Ok(out)
}

/// Locate the grid position of a site by its id, scanning site-bearing tiles.
fn site_position(world: &World, site_id: EntityId) -> Option<(u32, u32)> {
    for y in 0..world.height {
        for x in 0..world.width {
            if let Some(tile) = world.tile(x as i32, y as i32) {
                if tile.site == Some(site_id) {
                    return Some((x, y));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;
    use mindstrata_sim::world::Site;

    /// A tiny 2×2 world with hand-placed terrain and a site.
    fn fixture_world() -> World {
        let mut world = World::new(2, 2);
        world.tiles[0].terrain = Terrain::Forest;
        world.tiles[1].terrain = Terrain::Water;
        world.tiles[2].terrain = Terrain::Hill;
        // tile 3 stays Grassland.
        world.sites.push(Site {
            id: EntityId::new(7),
            kind: SiteKind::Farm,
            name: "Farm".into(),
            owner: None,
            capacity: 100,
            storage_capacity: mindstrata_core::fixed::Fixed::from_f64(200.0),
            inventory: Vec::new(),
        });
        world.tiles[0].site = Some(EntityId::new(7));
        world
    }

    #[test]
    fn render_is_deterministic_same_input_same_bytes() {
        let world = fixture_world();
        let agents = [RenderAgent::new(0, 1, 0), RenderAgent::new(1, 0, 4)];
        let a = render_world_png(&world, &agents, DEFAULT_CELL_PIXELS).unwrap();
        let b = render_world_png(&world, &agents, DEFAULT_CELL_PIXELS).unwrap();
        assert_eq!(a, b, "identical inputs must produce byte-identical PNGs");
    }

    #[test]
    fn render_produces_valid_png_of_expected_dimensions() {
        let world = fixture_world();
        let png = render_world_png(&world, &[], DEFAULT_CELL_PIXELS).unwrap();
        let decoded = image::load_from_memory(&png).expect("output must be a decodable PNG");
        assert_eq!(decoded.dimensions(), (24, 24), "2x2 world at 12px/cell");
    }

    #[test]
    fn terrain_colors_paint_known_cells() {
        let world = fixture_world();
        let img = render_world_rgba(&world, &[], DEFAULT_CELL_PIXELS);
        let at = |x: u32, y: u32| -> [u8; 3] {
            let i = (y * img.width + x) as usize * 4;
            [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]]
        };
        // Cell (0,0) carries the Farm overlay, so its corner pixel (1,1) must
        // still show Forest (the overlay is inset). The other three cells have
        // no overlay: their centers show their terrain.
        assert_eq!(at(1, 1), terrain_color(Terrain::Forest));
        assert_eq!(at(18, 6), terrain_color(Terrain::Water));
        assert_eq!(at(6, 18), terrain_color(Terrain::Hill));
        assert_eq!(at(18, 18), terrain_color(Terrain::Grassland));
    }

    #[test]
    fn site_overlay_marks_its_cell() {
        let world = fixture_world();
        let img = render_world_rgba(&world, &[], DEFAULT_CELL_PIXELS);
        // The Farm sits on the Forest cell (0,0): its center must be the farm
        // color, while a bare-corner pixel stays Forest (inset leaves margin).
        let i = (6 * img.width + 6) as usize * 4;
        assert_eq!(
            [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]],
            site_color(SiteKind::Farm)
        );
        // A corner pixel of the same cell keeps the terrain color (inset 20%).
        let corner = 1usize;
        let c = corner * 4;
        assert_eq!(
            [img.rgba[c], img.rgba[c + 1], img.rgba[c + 2]],
            terrain_color(Terrain::Forest)
        );
    }

    #[test]
    fn agent_sprite_renders_at_position() {
        let world = fixture_world();
        let agents = [RenderAgent::new(1, 1, 3)]; // purple on the Grassland cell
        let img = render_world_rgba(&world, &agents, DEFAULT_CELL_PIXELS);
        // Center of cell (1,1) = (18, 18) in pixels.
        let i = (18 * img.width + 18) as usize * 4;
        assert_eq!(
            [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]],
            AGENT_PALETTE[3],
            "agent sprite must replace the terrain at its cell center"
        );
    }

    #[test]
    fn agents_off_grid_are_ignored() {
        let world = fixture_world();
        let agents = [
            RenderAgent::new(-1, 0, 0),
            RenderAgent::new(0, 5, 1),
            RenderAgent::new(2, 0, 2),
            RenderAgent::new(0, 1, 4), // the only on-grid agent
        ];
        let img = render_world_rgba(&world, &agents, DEFAULT_CELL_PIXELS);
        // The on-grid agent is drawn (teal at cell (0,1) center = (6,18)).
        let i = (18 * img.width + 6) as usize * 4;
        assert_eq!(
            [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]],
            AGENT_PALETTE[4]
        );
    }

    #[test]
    fn empty_world_renders_valid_png() {
        // PNG forbids zero dimensions; a degenerate world must still render
        // as a valid 1x1 image rather than failing to encode.
        let world = World::new(0, 0);
        let png = render_world_png(&world, &[], DEFAULT_CELL_PIXELS).unwrap();
        let decoded = image::load_from_memory(&png).expect("degenerate world must still encode");
        assert_eq!(decoded.dimensions(), (1, 1));
        // The clamped fallback is the neutral backdrop (opaque, not
        // transparent black).
        let rgba = decoded.to_rgba8();
        let px = rgba.get_pixel(0, 0);
        assert_eq!([px[0], px[1], px[2], px[3]], [0x26, 0x26, 0x26, 255]);
    }

    #[test]
    fn hue_wraps_into_palette() {
        let world = fixture_world();
        // hue 8 wraps to palette index 0; the sprite must be drawn either way.
        let img = render_world_rgba(&world, &[RenderAgent::new(1, 1, 8)], DEFAULT_CELL_PIXELS);
        let i = (18 * img.width + 18) as usize * 4;
        assert_eq!(
            [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]],
            AGENT_PALETTE[0]
        );
    }

    #[test]
    fn world_without_sites_or_agents_is_plain_terrain() {
        let world = World::new(1, 1);
        let img = render_world_rgba(&world, &[], DEFAULT_CELL_PIXELS);
        for px in img.rgba.chunks_exact(4) {
            assert_eq!([px[0], px[1], px[2]], terrain_color(Terrain::Grassland));
            assert_eq!(px[3], 255, "all pixels must be opaque");
        }
    }

    /// A 3-frame replay over the same 2×2 fixture world with the agent
    /// moving diagonally from cell (0,0) to cell (1,1) across frames — the
    /// canonical replay shape. Both the world and each frame's agent array
    /// are leaked so the frames borrow for 'static (test-only; the world is
    /// immutable after construction).
    fn replay_fixture() -> Vec<ReplayFrame<'static>> {
        let world: &'static World = Box::leak(Box::new(fixture_world()));
        // (0,0) → (0,1) → (1,1): every position on-grid in the 2×2 world.
        let positions = [(0i32, 0i32), (0, 1), (1, 1)];
        positions
            .into_iter()
            .map(|(x, y)| {
                let agents: &'static [RenderAgent] =
                    Box::leak(vec![RenderAgent::new(x, y, 0)].into_boxed_slice());
                ReplayFrame { world, agents }
            })
            .collect()
    }

    /// Decode GIF bytes back into frames via the GifDecoder.
    fn decode_gif(gif: &[u8]) -> Vec<image::Frame> {
        use image::AnimationDecoder;
        let decoder = image::codecs::gif::GifDecoder::new(std::io::Cursor::new(gif))
            .expect("output must be a decodable GIF");
        decoder
            .into_frames()
            .collect_frames()
            .expect("frames decode")
    }

    #[test]
    fn replay_gif_is_deterministic_same_frames_same_bytes() {
        let frames = replay_fixture();
        let a = render_replay_gif(
            &frames,
            DEFAULT_CELL_PIXELS,
            200,
            image::codecs::gif::Repeat::Infinite,
        )
        .unwrap();
        let b = render_replay_gif(
            &frames,
            DEFAULT_CELL_PIXELS,
            200,
            image::codecs::gif::Repeat::Infinite,
        )
        .unwrap();
        assert_eq!(
            a, b,
            "identical frame sequences must produce byte-identical GIFs"
        );
    }

    #[test]
    fn replay_gif_decodes_to_expected_frame_count_and_dimensions() {
        let frames = replay_fixture();
        let gif = render_replay_gif(
            &frames,
            DEFAULT_CELL_PIXELS,
            200,
            image::codecs::gif::Repeat::Infinite,
        )
        .unwrap();
        let frames_enum = decode_gif(&gif);
        assert_eq!(
            frames_enum.len(),
            3,
            "all three replay frames must be encoded"
        );
        let dims = frames_enum[0].buffer().dimensions();
        assert_eq!(dims, (24, 24), "2x2 world at 12px/cell");
    }

    #[test]
    fn replay_gif_frames_show_agent_movement() {
        let frames = replay_fixture();
        let gif = render_replay_gif(
            &frames,
            DEFAULT_CELL_PIXELS,
            200,
            image::codecs::gif::Repeat::Infinite,
        )
        .unwrap();
        let frames_enum = decode_gif(&gif);
        // Frame 0: agent at cell (0,0) → center (6,6). Frame 2: agent at
        // cell (1,1) → center (18,18). The agent palette index 0 is red.
        let px = |f: &image::Frame, x: u32, y: u32| -> [u8; 3] {
            let b = f.buffer();
            let p = b.get_pixel(x, y);
            [p[0], p[1], p[2]]
        };
        assert_eq!(px(&frames_enum[0], 6, 6), AGENT_PALETTE[0]);
        assert_eq!(px(&frames_enum[2], 18, 18), AGENT_PALETTE[0]);
        // The agent is NOT at frame 0's destination yet — movement is real.
        assert_eq!(
            px(&frames_enum[0], 18, 18),
            terrain_color(Terrain::Grassland)
        );
    }

    #[test]
    fn replay_gif_single_frame_is_valid_and_decodes() {
        // A GIF needs at least one frame to be decodable; the single-frame
        // case is the minimum valid replay.
        let frames = replay_fixture();
        let single = render_replay_gif(
            &frames[..1],
            DEFAULT_CELL_PIXELS,
            200,
            image::codecs::gif::Repeat::Infinite,
        )
        .unwrap();
        let frames_enum = decode_gif(&single);
        assert_eq!(frames_enum.len(), 1, "one frame in, one frame out");
    }

    #[test]
    fn replay_gif_empty_frame_sequence_returns_empty() {
        // The GIF encoder buffers everything until the first frame; an empty
        // frame list therefore yields empty bytes (no header is emitted).
        // The render call itself must not fail — the caller's contract is
        // to supply ≥1 frame for a decodable replay.
        let gif = render_replay_gif(
            &[],
            DEFAULT_CELL_PIXELS,
            200,
            image::codecs::gif::Repeat::Infinite,
        )
        .expect("encoding must not fail on an empty frame list");
        assert!(gif.is_empty(), "zero frames in, zero bytes out");
    }

    #[test]
    fn replay_gif_finite_repeat_differs_from_infinite() {
        let frames = replay_fixture();
        let inf = render_replay_gif(
            &frames,
            DEFAULT_CELL_PIXELS,
            200,
            image::codecs::gif::Repeat::Infinite,
        )
        .unwrap();
        let once = render_replay_gif(
            &frames,
            DEFAULT_CELL_PIXELS,
            200,
            image::codecs::gif::Repeat::Finite(1),
        )
        .unwrap();
        assert_ne!(inf, once, "loop metadata must be encoded differently");
    }
}

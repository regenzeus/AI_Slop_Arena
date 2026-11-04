//! Asset overrides (ASSET-01): the game draws and synthesises everything it
//! needs, and a file in `assets/` replaces any of it. Nothing has to be
//! there; the game runs with no `assets/` folder at all.
//!
//! ```text
//! assets/sprites/slop_<name>.png      a slop character ("slop_six_finger_hand.png")
//! assets/sprites/boss_<name>.png      a boss
//! assets/sprites/hero_<name>_<face>.png  a hero's face 0 to 3
//! assets/sprites/floor_<name>.png     a biome's floor tile
//! assets/music/chapter<1-5>.wav|ogg   a chapter's loop
//! assets/sfx/<sound>.wav|ogg          a sound ("hit", "levelup", "dash", ...)
//! ```
//!
//! Names are lower case with `_` for spaces and punctuation.

use std::path::{Path, PathBuf};

use bevy::prelude::*;

use crate::art::Canvas;

#[derive(Resource, Debug, Clone)]
pub struct Overrides {
    dir: Option<PathBuf>,
}

impl Overrides {
    /// No overrides: everything is drawn and synthesised.
    pub fn none() -> Overrides {
        Overrides { dir: None }
    }

    /// Overrides from `dir`, whether or not it exists yet.
    pub fn at(dir: impl Into<PathBuf>) -> Overrides {
        Overrides { dir: Some(dir.into()) }
    }

    /// The `assets/` folder next to the executable, or in the working
    /// directory, or at the workspace root when run from cargo.
    pub fn find() -> Overrides {
        let mut candidates = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join("assets"));
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("assets"));
        }
        if let Some(manifest) = option_env!("CARGO_MANIFEST_DIR") {
            candidates.push(Path::new(manifest).join("../../assets"));
        }
        match candidates.into_iter().find(|c| c.is_dir()) {
            Some(dir) => {
                info!("asset overrides from {}", dir.display());
                Overrides::at(dir)
            }
            None => Overrides::none(),
        }
    }

    /// "Six-Finger Hand" becomes "six_finger_hand".
    pub fn slug(name: &str) -> String {
        let mut out = String::new();
        for c in name.chars() {
            if c.is_alphanumeric() {
                out.extend(c.to_lowercase());
            } else if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
        }
        out.trim_end_matches('_').to_string()
    }

    fn file(&self, sub: &str, name: &str, exts: &[&str]) -> Option<PathBuf> {
        let dir = self.dir.as_ref()?.join(sub);
        exts.iter().map(|e| dir.join(format!("{name}.{e}"))).find(|p| p.is_file())
    }

    /// A sprite from `assets/sprites/<name>.png`, as a canvas the art can
    /// whiten and scale like its own.
    pub fn canvas(&self, name: &str) -> Option<Canvas> {
        let path = self.file("sprites", name, &["png"])?;
        let img = match image::open(&path) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                warn!("cannot read {}: {e}", path.display());
                return None;
            }
        };
        let (w, h) = (img.width() as usize, img.height() as usize);
        Some(Canvas::from_rgba8(w, h, img.as_raw()))
    }

    /// A sound's bytes from `assets/sfx/<name>.wav` or `.ogg`.
    pub fn sound(&self, name: &str) -> Option<Vec<u8>> {
        std::fs::read(self.file("sfx", name, &["wav", "ogg"])?).ok()
    }

    /// A chapter's loop from `assets/music/chapter<n>.wav` or `.ogg`.
    pub fn music(&self, chapter: usize) -> Option<Vec<u8>> {
        std::fs::read(self.file("music", &format!("chapter{}", chapter + 1), &["wav", "ogg"])?).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: ASSET-01
    #[test]
    fn files_replace_drawn_art_and_nothing_is_required() {
        assert_eq!(Overrides::slug("Six-Finger Hand"), "six_finger_hand");
        assert_eq!(Overrides::slug("Grandma's 103rd Birthday Cake"), "grandma_s_103rd_birthday_cake");
        let none = Overrides::none();
        assert!(none.canvas("slop_chadgpt").is_none() && none.sound("hit").is_none() && none.music(0).is_none());
        let dir = std::env::temp_dir().join(format!("slop_assets_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sprites")).unwrap();
        std::fs::create_dir_all(dir.join("sfx")).unwrap();
        let mut img = image::RgbaImage::new(8, 4);
        img.put_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
        img.save(dir.join("sprites/slop_test.png")).unwrap();
        std::fs::write(dir.join("sfx/hit.wav"), b"RIFF").unwrap();
        let over = Overrides::at(&dir);
        let c = over.canvas("slop_test").expect("the png is read");
        assert_eq!((c.w, c.h), (8, 4));
        assert_eq!(c.whitened().image().width(), 8);
        assert_eq!(over.sound("hit").as_deref(), Some(&b"RIFF"[..]));
        assert!(over.canvas("slop_missing").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

//! Catalogue of the Crafting Apps the Glyphra start screen starts: the ArtCraft apps and
//! SolidCraft (3D CAD, from Tangency Labs), in alphabetical order.
//!
//! MdCraft is not listed: Glyphra's Markdown module replaces it. PdfCraft is listed (it organises
//! and protects PDFs) but does not own the `.pdf` extension: Glyphra's PDF module opens those.
//! ArtCraft is the AI image and video studio, Craft Anything the tool that explains how any
//! software works; neither owns a file type.
//! The start screen lists the installed apps and launches them; files they own appear
//! under **All**.

use crate::doctype::DocGroup;

/// One Crafting App and the files it opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CraftApp {
    /// Machine name: binary, package and asset stem (`photocraft`).
    pub id: &'static str,
    pub name: &'static str,
    /// Other names its installs may use: the upstream spelling (`CADCraft`) or a
    /// former name (`PrintCraft`, binary `printcraft`).
    pub aliases: &'static [&'static str],
    /// macOS bundle id, Flatpak id and Linux desktop-file id.
    pub bundle_id: &'static str,
    pub purpose: &'static str,
    /// Who publishes it.
    pub maker: &'static str,
    /// Short badge text for its files.
    pub badge: &'static str,
    /// Filter pill its files appear under.
    pub group: DocGroup,
    /// Lower-case extensions without the dot. Each extension belongs to one app.
    pub extensions: &'static [&'static str],
}

/// Where the ArtCraft team publishes the launcher for its apps.
pub const LAUNCHER_REPO: &str = "https://github.com/storytold/craft-launcher";
/// SolidCraft's repository.
pub const SOLIDCRAFT_REPO: &str = "https://github.com/tangency-labs/solidcraft";

const ARTCRAFT: &str = "ArtCraft";

/// The Crafting Apps known today, alphabetically.
///
/// `designcraft` and `effectcraft` follow VectorCraft's `.vectorcraft`
/// project-file pattern and still need confirming against those apps.
pub const CRAFT_APPS: [CraftApp; 18] = [
    CraftApp {
        name: "ArtCraft",
        id: "artcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.artcraft",
        purpose: "AI image and video studio",
        maker: ARTCRAFT,
        badge: "ART",
        group: DocGroup::Media,
        extensions: &[],
    },
    CraftApp {
        name: "CADCraft",
        id: "cadcraft",
        aliases: &["CADCraft"],
        bundle_id: "ai.storyteller.cadcraft",
        purpose: "2D and 3D CAD drafting",
        maker: ARTCRAFT,
        badge: "DWG",
        group: DocGroup::Cad,
        extensions: &["dwg", "dxf"],
    },
    CraftApp {
        name: "Craft Anything",
        id: "craft-anything",
        aliases: &["CraftAnything"],
        bundle_id: "ai.storyteller.craft-anything",
        purpose: "See how any software works, then build your own",
        maker: ARTCRAFT,
        badge: "ANY",
        group: DocGroup::Graphics,
        extensions: &[],
    },
    CraftApp {
        name: "DeckCraft",
        id: "deckcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.deckcraft",
        purpose: "Presentations",
        maker: ARTCRAFT,
        badge: "PPT",
        group: DocGroup::Office,
        extensions: &["pptx"],
    },
    CraftApp {
        name: "DesignCraft",
        id: "designcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.designcraft",
        purpose: "Page layout and publishing",
        maker: ARTCRAFT,
        badge: "DTP",
        group: DocGroup::Graphics,
        extensions: &["designcraft"],
    },
    CraftApp {
        name: "EffectCraft",
        id: "effectcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.effectcraft",
        purpose: "Motion graphics",
        maker: ARTCRAFT,
        badge: "FX",
        group: DocGroup::Media,
        extensions: &["effectcraft"],
    },
    CraftApp {
        name: "FigCraft",
        id: "figcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.figcraft",
        purpose: "Interface design and prototyping",
        maker: ARTCRAFT,
        badge: "UI",
        group: DocGroup::Graphics,
        extensions: &["figcraft", "fig"],
    },
    CraftApp {
        name: "FilmCraft",
        id: "filmcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.filmcraft",
        purpose: "Video editing",
        maker: ARTCRAFT,
        badge: "VID",
        group: DocGroup::Media,
        extensions: &["mp4", "mov", "mkv", "webm"],
    },
    CraftApp {
        name: "GridCraft",
        id: "gridcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.gridcraft",
        purpose: "Spreadsheets and data",
        maker: ARTCRAFT,
        badge: "XLS",
        group: DocGroup::Office,
        extensions: &["xlsx", "csv"],
    },
    CraftApp {
        name: "LightCraft",
        id: "lightcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.lightcraft",
        purpose: "Photo library and raw",
        maker: ARTCRAFT,
        badge: "RAW",
        group: DocGroup::Media,
        extensions: &["dng", "cr2", "cr3", "nef", "arw", "raf", "orf", "rw2"],
    },
    CraftApp {
        name: "MaterialCraft",
        id: "materialcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.materialcraft",
        purpose: "Procedural materials with node graphs",
        maker: ARTCRAFT,
        badge: "MAT",
        group: DocGroup::Graphics,
        extensions: &["sbs", "mcraft", "mcmat"],
    },
    CraftApp {
        name: "PdfCraft",
        id: "pdfcraft",
        aliases: &["PrintCraft"],
        bundle_id: "ai.storyteller.pdfcraft",
        purpose: "Organise and protect PDFs",
        maker: ARTCRAFT,
        badge: "PDF",
        group: DocGroup::Pdf,
        extensions: &[],
    },
    CraftApp {
        name: "PhotoCraft",
        id: "photocraft",
        aliases: &[],
        bundle_id: "ai.storyteller.photocraft",
        purpose: "Image editing, PSD files",
        maker: ARTCRAFT,
        badge: "IMG",
        group: DocGroup::Graphics,
        extensions: &["psd", "png", "jpg", "jpeg", "webp", "tif", "tiff"],
    },
    CraftApp {
        name: "SolidCraft",
        id: "solidcraft",
        aliases: &["SolveCraft"],
        bundle_id: "io.tangencylabs.solidcraft",
        purpose: "3D CAD and modelling",
        maker: "Tangency Labs",
        badge: "3D",
        group: DocGroup::Cad,
        extensions: &["step", "stp", "iges", "igs", "3mf", "stl", "obj", "gltf", "glb"],
    },
    CraftApp {
        name: "SoundCraft",
        id: "soundcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.soundcraft",
        purpose: "Recording, editing and mixing audio",
        maker: ARTCRAFT,
        badge: "AUD",
        group: DocGroup::Media,
        extensions: &["wav", "aif", "aiff", "flac", "mp3", "ogg", "m4a", "caf", "mid", "midi"],
    },
    CraftApp {
        name: "TextureCraft",
        id: "texturecraft",
        aliases: &[],
        bundle_id: "ai.storyteller.texturecraft",
        purpose: "3D texture painting",
        maker: ARTCRAFT,
        badge: "TEX",
        group: DocGroup::Graphics,
        extensions: &["texturecraft"],
    },
    CraftApp {
        name: "VectorCraft",
        id: "vectorcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.vectorcraft",
        purpose: "Vector illustration",
        maker: ARTCRAFT,
        badge: "VEC",
        group: DocGroup::Graphics,
        extensions: &["eps", "emf", "wmf", "vectorcraft"],
    },
    CraftApp {
        name: "WordCraft",
        id: "wordcraft",
        aliases: &[],
        bundle_id: "ai.storyteller.wordcraft",
        purpose: "Word processing",
        maker: ARTCRAFT,
        badge: "DOC",
        group: DocGroup::Office,
        extensions: &["docx"],
    },
];

/// The Crafting App that opens files with this extension (lower-case, no dot).
pub fn craft_for_extension(ext: &str) -> Option<&'static CraftApp> {
    CRAFT_APPS.iter().find(|a| a.extensions.contains(&ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_solidcraft_and_its_cad_formats() {
        let s = craft_for_extension("step").unwrap();
        assert_eq!(s.name, "SolidCraft");
        assert_eq!(s.group, DocGroup::Cad);
        assert_eq!(craft_for_extension("stl").unwrap().name, "SolidCraft");
    }

    #[test]
    fn apps_are_alphabetical_and_include_the_new_ones() {
        let names: Vec<&str> = CRAFT_APPS.iter().map(|a| a.name).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        for n in ["ArtCraft", "CADCraft", "Craft Anything", "FigCraft", "MaterialCraft", "PdfCraft", "SoundCraft", "TextureCraft"] {
            assert!(names.contains(&n), "{n} missing");
        }
        assert_eq!(craft_for_extension("dwg").unwrap().name, "CADCraft");
        assert_eq!(craft_for_extension("fig").unwrap().name, "FigCraft");
        assert_eq!(craft_for_extension("flac").unwrap().name, "SoundCraft");
        assert_eq!(craft_for_extension("sbs").unwrap().name, "MaterialCraft");
        assert!(craft_for_extension("pdf").is_none(), "PDF files belong to Glyphra's PDF module");
        assert_eq!(CRAFT_APPS.len(), 18);
    }

    #[test]
    fn no_extension_is_claimed_twice() {
        let mut seen = std::collections::HashSet::new();
        for a in CRAFT_APPS {
            for e in a.extensions {
                assert!(seen.insert(*e), "{e} is listed twice");
            }
        }
    }
}

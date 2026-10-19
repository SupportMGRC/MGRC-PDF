use crate::font::UNITS_PER_EM;
use crate::font::outline::OutlinePath;
use kurbo::BezPath;
use skrifa::instance::{LocationRef, Size};
use skrifa::metrics::GlyphMetrics;
use skrifa::outline::{DrawSettings, Engine, HintingInstance, HintingOptions, Target};
use skrifa::raw::TableProvider;
use skrifa::raw::ps::cff::{CffFontRef, Subfont, charset::Charset, v1::Cff};
use skrifa::raw::ps::string::Sid;
use skrifa::raw::ps::type1::Type1Font;
use skrifa::raw::tables::post::DEFAULT_GLYPH_NAMES;
use skrifa::raw::{FontData as ReadFontData, FontRead};
use skrifa::{FontRef, GlyphId, MetadataProvider, OutlineGlyphCollection};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use yoke::{Yoke, Yokeable};

type FontData = Arc<dyn AsRef<[u8]> + Send + Sync>;
type OpenTypeFontYoke = Yoke<OTFYoke<'static>, FontData>;
type CffFontYoke = Yoke<CFFYoke<'static>, FontData>;

/// A font blob for type 1 fonts.
#[derive(Clone)]
pub(crate) struct Type1FontBlob(Arc<Type1Font>);

impl Debug for Type1FontBlob {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type1 Font {{ .. }}")
    }
}

impl Type1FontBlob {
    pub(crate) fn new(data: FontData) -> Option<Self> {
        let table = Type1Font::new(data.as_ref().as_ref()).ok()?;
        Some(Self(Arc::new(table)))
    }

    pub(crate) fn table(&self) -> &Type1Font {
        self.0.as_ref()
    }

    pub(crate) fn outline_glyph(&self, gid: GlyphId) -> BezPath {
        let mut path = OutlinePath::new();
        let _ = self.table().draw(gid, Some(UNITS_PER_EM), &mut path);

        path.take()
    }
}

/// A font blob for CFF-based fonts.
#[derive(Clone)]
pub(crate) struct CffFontBlob(Arc<CffFontYoke>);

impl Debug for CffFontBlob {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type1 Font {{ .. }}")
    }
}

impl CffFontBlob {
    pub(crate) fn new(data: FontData) -> Option<Self> {
        let font = CffFontRef::new(data.as_ref().as_ref(), 0, None).ok()?;
        let cff = Cff::read(ReadFontData::new(data.as_ref().as_ref())).ok()?;
        let charset = font.charset();
        let subfonts = (0..font.num_subfonts())
            .map(|index| font.subfont(index, &[]).ok())
            .collect::<Option<Vec<_>>>()?;

        let yoke = Yoke::<CFFYoke<'static>, FontData>::attach_to_cart(data.clone(), |data| {
            let bytes = data.as_ref();
            let font = CffFontRef::new(bytes, 0, None).unwrap();
            let cff = Cff::read(ReadFontData::new(bytes)).unwrap();
            let charset = font.charset();
            let subfonts = (0..font.num_subfonts())
                .map(|index| font.subfont(index, &[]).unwrap())
                .collect();
            CFFYoke {
                font,
                cff,
                charset,
                subfonts,
            }
        });

        let _ = (cff, charset, subfonts);
        Some(Self(Arc::new(yoke)))
    }

    pub(crate) fn font_data(&self) -> FontData {
        self.0.backing_cart().clone()
    }

    pub(crate) fn font(&self) -> &CffFontRef<'_> {
        &self.0.as_ref().get().font
    }

    fn charset(&self) -> Option<&Charset<'_>> {
        self.0.as_ref().get().charset.as_ref()
    }

    fn subfont(&self, glyph: GlyphId) -> Option<&Subfont> {
        let index = self.font().subfont_index(glyph)? as usize;
        self.0.as_ref().get().subfonts.get(index)
    }

    pub(crate) fn outline_glyph(&self, glyph: GlyphId) -> BezPath {
        let mut path = OutlinePath::new();
        let Some(subfont) = self.subfont(glyph) else {
            return BezPath::new();
        };

        let _ = self
            .font()
            .draw(subfont, glyph, &[], Some(UNITS_PER_EM), &mut path);

        path.take()
    }

    pub(crate) fn glyph_names(&self) -> Vec<(GlyphId, String)> {
        let Some(charset) = self.charset() else {
            return Vec::new();
        };

        // TODO: Avoid collecting here.
        charset
            .iter()
            .filter_map(|(gid, sid)| {
                let bytes = self.0.as_ref().get().cff.string(sid)?;
                let name = std::str::from_utf8(bytes).ok()?.to_string();
                Some((gid, name))
            })
            .collect()
    }

    pub(crate) fn glyph_index_by_name(&self, name: &str) -> Option<GlyphId> {
        // TODO: This is probably slow to do repeatedly?
        self.charset()?.iter().find_map(|(gid, sid)| {
            let bytes = self.0.as_ref().get().cff.string(sid)?;
            (bytes == name.as_bytes()).then_some(gid)
        })
    }

    pub(crate) fn glyph_index_by_cid(&self, cid: u16) -> Option<GlyphId> {
        self.charset()?.glyph_id(Sid::new(cid))
    }

    pub(crate) fn glyph_index(&self, code: u8) -> Option<GlyphId> {
        self.font()
            .encoding()
            .and_then(|encoding| encoding.map(code))
    }

    pub(crate) fn num_glyphs(&self) -> u32 {
        self.font().num_glyphs()
    }

    pub(crate) fn is_cid(&self) -> bool {
        self.font().is_cid()
    }
}

/// A font blob for OpenType fonts.
#[derive(Clone)]
pub(crate) struct OpenTypeFontBlob {
    yoke: Arc<OpenTypeFontYoke>,
    cff_blob: Option<CffFontBlob>,
}

impl Debug for OpenTypeFontBlob {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenType Font {{ .. }}")
    }
}

impl OpenTypeFontBlob {
    pub(crate) fn new(data: FontData, index: u32) -> Option<Self> {
        // PdfCraft patch: repair an invalid `head.indexToLocFormat` before parsing (see
        // `repair_loca_format`).
        let data = if index == 0 { repair_loca_format(data) } else { data };
        // Check first whether the font is valid so we can unwrap in the closure.
        let f = FontRef::from_index(data.as_ref().as_ref(), index).ok()?;
        // Reject fonts with invalid post table version, fixes pdf.js issue 9462. Not sure if there
        // is a better fix, for some reason skrifa accepts the font which is completely invalid.
        let invalid = f.post().is_ok_and(|p| {
            !matches!(
                p.version().to_major_minor(),
                (1, 0) | (2, 0) | (2, 5) | (3, 0)
            )
        });

        if invalid {
            return None;
        }

        // We store this separately because we want to be able to cache the subfonts
        // of a CFF OpenType font, which is not possible using the current skrifa API.
        // Hopefully there will be some way in the future.
        let cff_blob = f
            .cff()
            .ok()
            .map(|cff| Arc::new(cff.offset_data().as_ref().to_vec()) as FontData)
            .and_then(CffFontBlob::new);

        let font_ref_yoke =
            Yoke::<OTFYoke<'static>, FontData>::attach_to_cart(data.clone(), |data| {
                let font_ref = FontRef::from_index(data.as_ref(), index).unwrap();

                let hinting_instance = if font_ref.outline_glyphs().require_interpreter() {
                    HintingInstance::new(
                        &font_ref.outline_glyphs(),
                        Size::new(UNITS_PER_EM),
                        LocationRef::default(),
                        HintingOptions {
                            engine: Engine::Interpreter,
                            target: Target::Mono,
                        },
                    )
                    .ok()
                } else {
                    None
                };

                OTFYoke {
                    font_ref: font_ref.clone(),
                    outline_glyphs: font_ref.outline_glyphs(),
                    hinting_instance,
                    glyph_metrics: font_ref
                        .glyph_metrics(Size::new(UNITS_PER_EM), LocationRef::default()),
                }
            });

        Some(Self {
            yoke: Arc::new(font_ref_yoke),
            cff_blob,
        })
    }

    pub(crate) fn font_data(&self) -> FontData {
        self.yoke.backing_cart().clone()
    }

    pub(crate) fn font_ref(&self) -> &FontRef<'_> {
        &self.yoke.as_ref().get().font_ref
    }

    pub(crate) fn glyph_metrics(&self) -> &GlyphMetrics<'_> {
        &self.yoke.as_ref().get().glyph_metrics
    }

    pub(crate) fn glyph_names(&self) -> HashMap<String, GlyphId> {
        // Note: We don't call the `glyph_name` method provided by read-fonts because
        // calling it repeatedly is very slow.
        let mut glyph_names = HashMap::new();
        let Ok(post) = self.font_ref().post() else {
            return glyph_names;
        };

        match post.version().to_major_minor() {
            (1, 0) => {
                for (gid, name) in DEFAULT_GLYPH_NAMES.iter().enumerate() {
                    glyph_names.insert((*name).to_string(), GlyphId::new(gid as u32));
                }
            }
            (2, 0) => {
                let Some(name_indices) = post.glyph_name_index() else {
                    return glyph_names;
                };
                let Some(string_data) = post.string_data() else {
                    return glyph_names;
                };
                let custom_names = string_data
                    .iter()
                    .map(|entry| entry.ok().map(|name| name.as_str()))
                    .collect::<Vec<_>>();

                for (gid, idx) in name_indices.iter().enumerate() {
                    let idx = idx.get() as usize;
                    if let Some(name) = DEFAULT_GLYPH_NAMES.get(idx).copied().or_else(|| {
                        custom_names
                            .get(idx.saturating_sub(DEFAULT_GLYPH_NAMES.len()))
                            .copied()
                            .flatten()
                    }) {
                        glyph_names.insert(name.to_string(), GlyphId::new(gid as u32));
                    }
                }
            }
            _ => {}
        }

        glyph_names
    }

    fn outline_glyphs(&self) -> &OutlineGlyphCollection<'_> {
        &self.yoke.as_ref().get().outline_glyphs
    }

    pub(crate) fn outline_glyph(&self, glyph: GlyphId) -> BezPath {
        if let Some(cff_blob) = self.cff_blob.as_ref() {
            return cff_blob.outline_glyph(glyph);
        }

        let mut path = OutlinePath::new();

        let draw_settings = if let Some(instance) = self.yoke.get().hinting_instance.as_ref() {
            // Note: We always hint at the font size `UNITS_PER_EM`, which obviously isn't very useful. We don't do this
            // for better text quality (right now), but instead because there are some PDFs with obscure fonts that
            // actually render wrongly if hinting is disabled!
            DrawSettings::hinted(instance, false)
        } else {
            DrawSettings::unhinted(Size::new(UNITS_PER_EM), LocationRef::default())
        };

        let Some(outline) = self.outline_glyphs().get(glyph) else {
            return BezPath::new();
        };

        let _ = outline.draw(draw_settings, &mut path);
        path.take()
    }
}

#[derive(Yokeable, Clone)]
struct OTFYoke<'a> {
    font_ref: FontRef<'a>,
    glyph_metrics: GlyphMetrics<'a>,
    hinting_instance: Option<HintingInstance>,
    outline_glyphs: OutlineGlyphCollection<'a>,
}

#[derive(Yokeable, Clone)]
struct CFFYoke<'a> {
    font: CffFontRef<'a>,
    cff: Cff<'a>,
    charset: Option<Charset<'a>>,
    subfonts: Vec<Subfont>,
}

/// PdfCraft patch: `head.indexToLocFormat` must be 0 (short `loca` offsets) or 1 (long), but some
/// PDF producers write other values (Datalogics-made invoices embed Arial subsets with 256).
/// skrifa reads anything but 1 as short, so every glyph comes out empty or garbled and the text
/// shows as boxes; FreeType, pdf.js and Acrobat read such fonts. Work out the real format from
/// the size of `loca` (one entry more than there are glyphs, 2 or 4 bytes each) and patch a copy.
/// Fonts with a valid value, collections and anything unreadable are returned unchanged.
/// Covered by `pdfcraft-render` `raster::tests::fonts_with_an_invalid_loca_format_still_draw`.
pub(crate) fn repair_loca_format(data: FontData) -> FontData {
    match loca_format_fix(data.as_ref().as_ref()) {
        Some((offset, format)) => {
            let mut bytes = data.as_ref().as_ref().to_vec();
            match bytes.get_mut(offset..offset + 2) {
                Some(field) => {
                    field.copy_from_slice(&format.to_be_bytes());
                    Arc::new(bytes)
                }
                None => data,
            }
        }
        None => data,
    }
}

/// Where `head.indexToLocFormat` is and what it should be, when it is invalid and `loca`'s size
/// says which format the font really uses.
fn loca_format_fix(b: &[u8]) -> Option<(usize, u16)> {
    let u16_at = |at: usize| b.get(at..at.checked_add(2)?).map(|s| u16::from_be_bytes([s[0], s[1]]));
    let u32_at = |at: usize| b.get(at..at.checked_add(4)?).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]));
    // A single sfnt (TrueType outlines or OpenType), not a collection.
    let tag = u32_at(0)?;
    if tag != 0x0001_0000 && b.get(0..4)? != b"true" {
        return None;
    }
    let tables = usize::from(u16_at(4)?);
    let (mut head, mut maxp, mut loca_len) = (None, None, None);
    for i in 0..tables.min(256) {
        let rec = 12usize.checked_add(i.checked_mul(16)?)?;
        let name = b.get(rec..rec.checked_add(4)?)?;
        let offset = usize::try_from(u32_at(rec.checked_add(8)?)?).ok()?;
        let length = usize::try_from(u32_at(rec.checked_add(12)?)?).ok()?;
        match name {
            b"head" => head = Some(offset),
            b"maxp" => maxp = Some(offset),
            b"loca" => loca_len = Some(length),
            _ => {}
        }
    }
    let field = head?.checked_add(50)?;
    let current = u16_at(field)?;
    if current <= 1 {
        return None;
    }
    let glyphs = usize::from(u16_at(maxp?.checked_add(4)?)?);
    let entries = glyphs.checked_add(1)?;
    let loca_len = loca_len?;
    let format = if loca_len >= entries.checked_mul(4)? {
        1
    } else if loca_len >= entries.checked_mul(2)? {
        0
    } else {
        return None;
    };
    Some((field, format))
}


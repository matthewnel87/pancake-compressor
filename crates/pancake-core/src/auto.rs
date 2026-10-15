//! Presets, "keep format" and rule-based Smart Auto (docs/07, PNK-014).
//!
//! This module turns a user's choice (target format + preset) into concrete
//! encoder settings, runs a **bounded** list of candidates, and keeps the
//! smallest valid result. There is no AI and no perceptual quality metric:
//! the choice is rule-based and deterministic, so the same file and settings
//! always give the same answer. It must be described to users as "rule-based
//! automatic choice", never as perceptually guaranteed (docs/07).
//!
//! **Never larger (ADR-015):** if no candidate is smaller than the original
//! file, the original bytes are returned with [`Outcome::kept_original`] set,
//! and the caller copies the original instead of writing a bigger file.
//!
//! **Not here yet:** resizing (the "Website" preset) and the batch/output
//! decisions (file names, folders), which are M2.

use std::fmt;

use crate::decode::DecodedImage;
use crate::encode::{Adjustments, EncodeError, Encoded, avif, jpeg, png, webp};
use crate::format::ImageFormat;

/// What output format the user asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Same format as the source.
    Keep,
    Png,
    Jpeg,
    WebP,
    Avif,
    /// Rule-based choice among a few candidates.
    Auto,
}

/// How to trade size against quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Moderate quality and effort; the default.
    Balanced,
    /// Stronger lossy savings; visible artefacts are more likely.
    Smaller,
    /// Conservative settings; files may be larger.
    BestQuality,
    /// Only pixel-identical outputs. Formats without a lossless mode here
    /// (JPEG, AVIF) keep the original.
    Lossless,
}

/// Choices that need the user's explicit consent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Consent {
    /// Allow AVIF output even though the source's colour profile will be lost.
    pub allow_colour_profile_loss: bool,
    /// Include AVIF among Smart Auto candidates (slow; off by default).
    pub try_avif_in_auto: bool,
}

/// The chosen result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub output_format: ImageFormat,
    pub bytes: Vec<u8>,
    pub adjustments: Adjustments,
    /// `true` when no candidate beat the original, so `bytes` is the
    /// original file unchanged.
    pub kept_original: bool,
    /// Short, stable explanation of the choice for the results list.
    pub reason: Reason,
}

/// Why a result was chosen. Stable codes for the UI and progress events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The only candidate for the requested format and preset.
    Requested,
    /// Smallest of several candidates.
    Smallest,
    /// Nothing smaller than the original was found (ADR-015).
    OriginalSmaller,
    /// The preset has no mode for this format (for example lossless JPEG).
    NoSuitableMode,
}

impl Reason {
    pub fn code(&self) -> &'static str {
        match self {
            Reason::Requested => "requested",
            Reason::Smallest => "smallest-candidate",
            Reason::OriginalSmaller => "original-already-smaller",
            Reason::NoSuitableMode => "no-suitable-mode",
        }
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Reason::Requested => "Compressed with the chosen settings.",
            Reason::Smallest => "Smallest of the formats tried.",
            Reason::OriginalSmaller => "The original was already smaller, so it was kept.",
            Reason::NoSuitableMode => {
                "This preset cannot be applied to this format, so the original was kept."
            }
        })
    }
}

/// One encoder run with concrete settings.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Candidate {
    PngLossless(png::LosslessOptions),
    PngPalette(png::LossyOptions),
    Jpeg(jpeg::JpegOptions),
    WebP(webp::WebpOptions),
    Avif(avif::AvifOptions),
}

impl Candidate {
    fn output_format(&self) -> ImageFormat {
        match self {
            Candidate::PngLossless(_) | Candidate::PngPalette(_) => ImageFormat::Png,
            Candidate::Jpeg(_) => ImageFormat::Jpeg,
            Candidate::WebP(_) => ImageFormat::WebP,
            Candidate::Avif(_) => ImageFormat::Avif,
        }
    }

    fn run(&self, source: &[u8], decoded: &DecodedImage) -> Result<Encoded, EncodeError> {
        match *self {
            // Lossless recompression changes nothing visible.
            Candidate::PngLossless(options) => png::optimise_lossless(source, decoded, options)
                .map(|bytes| Encoded {
                    bytes,
                    adjustments: Adjustments::default(),
                }),
            Candidate::PngPalette(options) => png::quantise_lossy(decoded, options),
            Candidate::Jpeg(options) => jpeg::encode(decoded, options),
            Candidate::WebP(options) => webp::encode(decoded, options),
            Candidate::Avif(options) => avif::encode(decoded, options),
        }
    }
}

// Preset tables. Numbers are starting points recorded in ADR-035; tune them
// with real-photo benchmarks before v0.1, not by intuition.
fn jpeg_settings(preset: Preset) -> Option<jpeg::JpegOptions> {
    let base = jpeg::JpegOptions::default();
    match preset {
        Preset::Balanced => Some(base),
        Preset::Smaller => Some(jpeg::JpegOptions {
            quality: 65,
            ..base
        }),
        Preset::BestQuality => Some(jpeg::JpegOptions {
            quality: 90,
            subsampling: jpeg::Subsampling::Full,
            ..base
        }),
        Preset::Lossless => None,
    }
}

fn webp_settings(preset: Preset) -> webp::WebpOptions {
    let base = webp::WebpOptions::default();
    match preset {
        Preset::Balanced => base,
        Preset::Smaller => webp::WebpOptions {
            quality: 65,
            ..base
        },
        Preset::BestQuality => webp::WebpOptions {
            quality: 92,
            method: 6,
            ..base
        },
        Preset::Lossless => webp::WebpOptions {
            lossless: true,
            quality: 75,
            method: 4,
        },
    }
}

fn palette_settings(preset: Preset) -> Option<png::LossyOptions> {
    let base = png::LossyOptions::default();
    match preset {
        Preset::Balanced => Some(base),
        Preset::Smaller => Some(png::LossyOptions {
            min_quality: 40,
            target_quality: 65,
            ..base
        }),
        Preset::BestQuality => Some(png::LossyOptions {
            min_quality: 80,
            target_quality: 95,
            ..base
        }),
        Preset::Lossless => None,
    }
}

fn avif_settings(preset: Preset, consent: Consent) -> Option<avif::AvifOptions> {
    let base = avif::AvifOptions {
        allow_colour_profile_loss: consent.allow_colour_profile_loss,
        ..avif::AvifOptions::default()
    };
    match preset {
        Preset::Balanced => Some(base),
        Preset::Smaller => Some(avif::AvifOptions {
            quality: 55.0,
            ..base
        }),
        Preset::BestQuality => Some(avif::AvifOptions {
            quality: 85.0,
            ..base
        }),
        Preset::Lossless => None,
    }
}

/// Rough "is this a graphic rather than a photo" test: transparency, or few
/// distinct colours in a sample. Deterministic and cheap; it only chooses
/// which candidates to try, never the final result.
fn looks_like_graphic(decoded: &DecodedImage) -> bool {
    let rgba = decoded.pixels.to_rgba8();
    if rgba.pixels().any(|p| p[3] < 255) {
        return true;
    }
    let mut seen = std::collections::HashSet::new();
    // Every 7th pixel (at most ~64k samples) is enough to tell flat graphics
    // from photographic noise.
    for pixel in rgba.pixels().step_by(7).take(65_536) {
        seen.insert(pixel.0);
        if seen.len() > 256 {
            return false;
        }
    }
    true
}

fn candidates(
    decoded: &DecodedImage,
    target: Target,
    preset: Preset,
    consent: Consent,
) -> Vec<Candidate> {
    let source = decoded.info.format;
    let palette = palette_settings(preset).map(Candidate::PngPalette);
    let png_lossless = (source == ImageFormat::Png)
        .then_some(Candidate::PngLossless(png::LosslessOptions::default()));
    let webp_candidate = Candidate::WebP(webp_settings(preset));

    let target = match target {
        Target::Keep => match source {
            ImageFormat::Png => Target::Png,
            ImageFormat::Jpeg => Target::Jpeg,
            ImageFormat::WebP => Target::WebP,
            ImageFormat::Avif => Target::Avif,
        },
        other => other,
    };

    match target {
        Target::Png => {
            // Best quality keeps PNG lossless; palette may visibly change a
            // photo, so it is only tried for Balanced and Smaller.
            let use_palette = matches!(preset, Preset::Balanced | Preset::Smaller);
            png_lossless
                .into_iter()
                .chain(palette.filter(|_| use_palette))
                .collect()
        }
        Target::Jpeg => jpeg_settings(preset)
            .map(Candidate::Jpeg)
            .into_iter()
            .collect(),
        Target::WebP => vec![webp_candidate],
        Target::Avif => avif_settings(preset, consent)
            .map(Candidate::Avif)
            .into_iter()
            .collect(),
        Target::Auto => {
            let mut list = Vec::new();
            if preset == Preset::Lossless {
                list.extend(png_lossless);
                list.push(webp_candidate);
            } else if looks_like_graphic(decoded) {
                list.extend(png_lossless);
                list.extend(palette);
                list.push(Candidate::WebP(webp_settings(Preset::Lossless)));
                list.push(webp_candidate);
            } else {
                list.push(webp_candidate);
                list.extend(jpeg_settings(preset).map(Candidate::Jpeg));
            }
            if consent.try_avif_in_auto && preset != Preset::Lossless {
                list.extend(avif_settings(preset, consent).map(Candidate::Avif));
            }
            list
        }
        Target::Keep => Vec::new(), // resolved above
    }
}

/// Compresses one decoded image according to `target` and `preset`.
///
/// `source` must be the original file bytes (needed for lossless PNG and
/// for the "never larger" comparison). Candidate failures that are expected
/// outcomes (`quality-not-reached`, refused colour-profile loss) skip that
/// candidate; if every candidate fails, the last error is returned.
pub fn compress(
    source: &[u8],
    decoded: &DecodedImage,
    target: Target,
    preset: Preset,
    consent: Consent,
) -> Result<Outcome, EncodeError> {
    let list = candidates(decoded, target, preset, consent);
    let keep_original = |reason| Outcome {
        output_format: decoded.info.format,
        bytes: source.to_vec(),
        adjustments: Adjustments::default(),
        kept_original: true,
        reason,
    };
    if list.is_empty() {
        return Ok(keep_original(Reason::NoSuitableMode));
    }

    let several = list.len() > 1;
    let mut best: Option<(ImageFormat, Encoded)> = None;
    let mut last_error = None;
    for candidate in list {
        match candidate.run(source, decoded) {
            Ok(encoded) => {
                // Strictly smaller wins; ties keep the earlier candidate, so
                // the order above is the deterministic tie-break.
                let better = best
                    .as_ref()
                    .is_none_or(|(_, current)| encoded.bytes.len() < current.bytes.len());
                if better {
                    best = Some((candidate.output_format(), encoded));
                }
            }
            Err(error) => last_error = Some(error),
        }
    }

    match best {
        Some((_, encoded)) if encoded.bytes.len() >= source.len() => {
            Ok(keep_original(Reason::OriginalSmaller))
        }
        Some((output_format, encoded)) => Ok(Outcome {
            output_format,
            bytes: encoded.bytes,
            adjustments: encoded.adjustments,
            kept_original: false,
            reason: if several {
                Reason::Smallest
            } else {
                Reason::Requested
            },
        }),
        None => Err(last_error.unwrap_or(EncodeError::Failed(decoded.info.format))),
    }
}

#[cfg(test)]
mod tests;

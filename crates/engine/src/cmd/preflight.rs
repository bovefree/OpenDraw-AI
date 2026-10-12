//! OpenDraw AI: preliminary print checks before PDF export.
//! This report is NOT a PDF/X validator and does not inspect effective image PPI,
//! font embedding, output intents, or print-provider requirements.

use serde_json::{Value, json};
use vectorcraft_doc::{ColorMode, Unit};

use super::*;

const C: &str = "document.preflight";

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        query "document.preflight",
        "Print Preflight",
        [],
        None,
        "{profile?: genericPrint|poster140x200, minBleedMm?: 0..50} checks document dimensions, CMYK mode, document bleed and raster-effect resolution; never certifies a PDF/X file",
        has_doc,
        run
    )]
}

fn run(s: &mut Session, p: &Value) -> Result<Value> {
    let empty = serde_json::Map::new();
    let o = match p {
        Value::Null => &empty,
        Value::Object(o) => o,
        _ => return Err(bad(C, "params must be an object")),
    };
    for key in o.keys() {
        if key != "profile" && key != "minBleedMm" {
            return Err(bad(C, format!("unknown preflight option `{key}`")));
        }
    }
    let profile = o.get("profile").map(Value::as_str).unwrap_or(Some("genericPrint")).ok_or_else(|| bad(C, "profile must be text"))?;
    if !matches!(profile, "genericPrint" | "poster140x200") {
        return Err(bad(C, "profile must be genericPrint or poster140x200"));
    }
    let default_bleed = if profile == "poster140x200" { 5.0 } else { 3.0 };
    let min_bleed = match o.get("minBleedMm") {
        Some(v) => v.as_f64().filter(|n| n.is_finite() && (0.0..=50.0).contains(n)).ok_or_else(|| bad(C, "minBleedMm must be 0..50"))?,
        None => default_bleed,
    };
    let d = &s.doc()?.doc;
    let mm = Unit::Millimeters;
    let bleed_mm: Vec<f64> = d.setup.bleed.iter().map(|pt| mm.from_pt(*pt)).collect();
    let mut findings = Vec::new();
    if d.artboards.is_empty() {
        findings.push(json!({"code":"noArtboards","severity":"error","message":"Document has no pages to export"}));
    }
    if d.color_mode != ColorMode::Cmyk {
        findings.push(json!({"code":"rgbDocument","severity":"warning","message":"Document is RGB; check printer's required color conversion"}));
    }
    if bleed_mm.iter().any(|b| *b + 0.0001 < min_bleed) {
        findings.push(json!({
            "code":"insufficientBleed","severity":"warning",
            "message":"At least one side has less than the required document bleed",
            "requiredMm":min_bleed,"actualMm":bleed_mm
        }));
    }
    if d.raster_effects_ppi + 0.0001 < 300.0 {
        findings.push(json!({
            "code":"lowRasterEffectsPpi","severity":"warning",
            "message":"Raster-effect resolution is below 300 ppi; this does not measure the effective PPI of embedded images",
            "rasterEffectsPpi":d.raster_effects_ppi
        }));
    }
    let artboards: Vec<Value> = d.artboards.iter().enumerate().map(|(i, a)| {
        let width = mm.from_pt(a.rect.width());
        let height = mm.from_pt(a.rect.height());
        if profile == "poster140x200" && ((width - 1400.0).abs() > 0.5 || (height - 2000.0).abs() > 0.5) {
            findings.push(json!({
                "code":"posterSizeMismatch","severity":"error","page":i + 1,
                "message":"Page dimensions differ from 1400 by 2000 mm",
                "widthMm":width,"heightMm":height
            }));
        }
        json!({"page":i + 1,"widthMm":width,"heightMm":height})
    }).collect();
    let preliminary_pass = findings.is_empty();
    Ok(json!({
        "profile":profile,
        "preliminaryPass":preliminary_pass,
        "pdfCertified":false,
        "artboards":artboards,
        "colorMode":if d.color_mode == ColorMode::Cmyk {"cmyk"} else {"rgb"},
        "documentCmykProfile":d.color_profiles.cmyk,
        "bleedMm":bleed_mm,
        "rasterEffectsPpi":d.raster_effects_ppi,
        "findings":findings,
        "notVerified":["PDF/X conformance","PDF OutputIntent","actual exported BleedBox/TrimBox","font embedding","embedded image effective PPI","overprints and spot-color separations"]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poster_preset_passes_document_checks_without_pdf_certification() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"preset":"OpenDraw Manifesto 140×200 cm"})).unwrap();
        let r = s.execute(C, &json!({"profile":"poster140x200"})).unwrap();
        assert_eq!(r["preliminaryPass"], true, "{r}");
        assert_eq!(r["pdfCertified"], false);
        assert_eq!(r["colorMode"], "cmyk");
        assert_eq!(r["artboards"][0]["page"], 1);
        assert!((r["artboards"][0]["widthMm"].as_f64().unwrap() - 1400.0).abs() < 0.0001);
        assert!((r["artboards"][0]["heightMm"].as_f64().unwrap() - 2000.0).abs() < 0.0001);
        assert!(r["notVerified"].as_array().unwrap().len() >= 5);
    }

    #[test]
    fn default_page_warns_about_bleed_color_and_raster_resolution() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width":100,"height":100})).unwrap();
        let r = s.execute(C, &json!({"profile":"poster140x200"})).unwrap();
        let codes: Vec<&str> = r["findings"].as_array().unwrap().iter().filter_map(|x| x["code"].as_str()).collect();
        assert!(codes.contains(&"rgbDocument"), "{r}");
        assert!(codes.contains(&"insufficientBleed"), "{r}");
        assert!(codes.contains(&"posterSizeMismatch"), "{r}");
        assert_eq!(r["preliminaryPass"], false);
        assert_eq!(r["pdfCertified"], false);
    }

    #[test]
    fn bad_options_are_rejected() {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        for params in [json!({"profile":"invalid"}),json!({"minBleedMm":-1}),json!({"minBleedMm":"abc"}),json!({"hidden":true}),json!([1])] {
            assert!(s.execute(C, &params).is_err(), "{params}");
        }
    }
}

//! Mini image editor — `ll image <command> [args] [options]`.
//!
//! ImageMagick-flavoured operations implemented in pure Rust on top of the
//! `image` crate: crop, resize, thumbnail, rotate, flip, pad, border, trim,
//! slice (grid tiling), montage (contact sheet), color adjustments, filters,
//! format conversion and a terminal preview.
//!
//! Every pixel operation is registered in [`OPS`], so the same operators can be
//! chained with `ll image apply in.png out.webp crop:80%x80% grayscale blur:2`.

use crate::cli::{arg_or, err, merge, normalize, parse_f64, parse_opts, parse_u32, pos, req, Opts};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn main_with_args(args: &[String]) -> io::Result<()> {
    let sub = args.first().map(String::as_str).unwrap_or("");
    let rest: &[String] = if args.is_empty() { &[] } else { &args[1..] };

    match sub {
        "" | "help" | "--help" | "-h" => {
            usage();
            Ok(())
        }
        "list" | "--list" => {
            list();
            Ok(())
        }
        _ => {
            if let Some(cmd) = find_cmd(sub) {
                return run_cmd(cmd, rest);
            }
            match normalize(sub).as_str() {
                "info" | "identify" => cmd_info(rest),
                "slice" | "tiles" | "tile" => cmd_slice(rest),
                "montage" | "contactsheet" | "sheet" => cmd_montage(rest),
                "convert" => cmd_convert(rest),
                "view" | "preview" => cmd_view(rest),
                "apply" | "chain" | "pipeline" => cmd_apply(rest),
                _ => {
                    eprintln!("Unknown image command: {sub}\n");
                    list();
                    Ok(())
                }
            }
        }
    }
}

fn usage() {
    println!("ll image — mini image editor (crop, resize, slice, montage, filters …)");
    println!();
    println!("Usage: ll image <command> [args] [options]");
    println!();
    println!("  ll image list                All commands + 'apply' operators");
    println!("  ll image <command> --help    Options for one command");
    println!();
    println!("Examples:");
    println!("  ll image info photo.png --json");
    println!("  ll image crop photo.png out.png -g 400x300+50+50");
    println!("  ll image resize photo.png small.jpg -p 50% -q 85");
    println!("  ll image slice photo.png tiles/part --cols 3 --rows 2");
    println!("  ll image montage photos/ --out sheet.png --cols 4 --thumb 320x320");
    println!("  ll image convert photo.png photo.webp");
    println!("  ll image apply photo.png out.webp crop:80%x80% resize:1200x_ grayscale sharpen:2");
    println!("  ll image view photo.png");
}

fn list() {
    println!("Inspect");
    println!("  {:<36} format, size, color, file size (--json)", "info <file>...");
    println!();
    println!("Files & tools");
    println!("  {:<36} --cols N --rows N | --tile WxH", "slice <in> <prefix>");
    println!("  {:<36} --cols N --gap PX --thumb WxH", "montage <in...> --out <sheet>");
    println!("  {:<36} --out-dir DIR, --format, --quality", "convert <in> <out>");
    println!("  {:<36} terminal preview (--max-width, --background)", "view <file>");
    println!("  {:<36} chain operators, e.g. crop:400x300+0+0", "apply <in> <out> <op:spec>...");
    println!();
    println!("Pixel operations (each is also an 'apply' operator):");
    let mut group = "";
    for c in CMDS {
        if c.group != group {
            group = c.group;
            println!("  {group}");
        }
        let desc = find_op(c.name).map(|o| o.desc).unwrap_or("");
        println!("    {:<58} {}", c.usage, desc);
    }
    println!();
    println!("Operator spec syntax (after 'op:' inside apply):");
    for op in OPS {
        let spec = if op.spec.is_empty() { "(no arguments)" } else { op.spec };
        println!("  {:<12} {}", op.name, spec);
    }
}

// ─── Command table ───────────────────────────────────────────────────────────

const SAVE_VALS: &[&str] = &["--quality", "-q", "--format", "-f", "--background", "-b"];
const SAVE_BOOLS: &[&str] = &[];

type BuildSpec = fn(&Opts) -> io::Result<String>;

struct Cmd {
    name: &'static str,
    usage: &'static str,
    group: &'static str,
    vals: &'static [&'static str],
    bools: &'static [&'static str],
    build: BuildSpec,
}

static CMDS: &[Cmd] = &[
    // ── Geometry ──
    Cmd {
        name: "crop",
        usage: "crop <in> <out> -g WxH+X+Y | -w W -h H [-x X -y Y]",
        group: "Geometry",
        vals: &["--geometry", "-g", "--width", "-w", "--height", "-h", "--x", "-x", "--y", "-y"],
        bools: &[],
        build: b_crop,
    },
    Cmd {
        name: "resize",
        usage: "resize <in> <out> -p 50% | -w W [-h H] [--filter F] [--mode m]",
        group: "Geometry",
        vals: &["--percent", "-p", "--width", "-w", "--height", "-h", "--filter", "--mode"],
        bools: &[],
        build: b_resize,
    },
    Cmd {
        name: "thumbnail",
        usage: "thumbnail <in> <out> -s N|WxH",
        group: "Geometry",
        vals: &["--size", "-s"],
        bools: &[],
        build: b_thumbnail,
    },
    Cmd {
        name: "rotate",
        usage: "rotate <in> <out> -a DEGREES [--background COLOR]",
        group: "Geometry",
        vals: &["--angle", "-a"],
        bools: &[],
        build: b_rotate,
    },
    Cmd {
        name: "flip",
        usage: "flip <in> <out> --horizontal | --vertical | -d h|v|both",
        group: "Geometry",
        vals: &["--direction", "-d"],
        bools: &["--horizontal", "--vertical", "--both"],
        build: b_flip,
    },
    Cmd {
        name: "pad",
        usage: "pad <in> <out> [-w W] [-h H] [--color C] [--gravity g]",
        group: "Geometry",
        vals: &["--width", "-w", "--height", "-h", "--color", "-c", "--gravity"],
        bools: &[],
        build: b_pad,
    },
    Cmd {
        name: "border",
        usage: "border <in> <out> -w PX [--color C]",
        group: "Geometry",
        vals: &["--width", "-w", "--color", "-c"],
        bools: &[],
        build: b_border,
    },
    Cmd {
        name: "trim",
        usage: "trim <in> <out> [--color C|auto] [--tolerance N]",
        group: "Geometry",
        vals: &["--color", "-c", "--tolerance"],
        bools: &[],
        build: b_trim,
    },
    // ── Color ──
    Cmd {
        name: "grayscale",
        usage: "grayscale <in> <out>",
        group: "Color",
        vals: &[],
        bools: &[],
        build: b_none,
    },
    Cmd {
        name: "invert",
        usage: "invert <in> <out>",
        group: "Color",
        vals: &[],
        bools: &[],
        build: b_none,
    },
    Cmd {
        name: "sepia",
        usage: "sepia <in> <out>",
        group: "Color",
        vals: &[],
        bools: &[],
        build: b_none,
    },
    Cmd {
        name: "brightness",
        usage: "brightness <in> <out> -v -255..255",
        group: "Color",
        vals: &["--value", "-v"],
        bools: &[],
        build: b_value,
    },
    Cmd {
        name: "contrast",
        usage: "contrast <in> <out> -v -100..100",
        group: "Color",
        vals: &["--value", "-v"],
        bools: &[],
        build: b_value,
    },
    Cmd {
        name: "saturate",
        usage: "saturate <in> <out> -v -100..300",
        group: "Color",
        vals: &["--value", "-v"],
        bools: &[],
        build: b_value,
    },
    Cmd {
        name: "hue",
        usage: "hue <in> <out> -v -180..180",
        group: "Color",
        vals: &["--value", "-v"],
        bools: &[],
        build: b_value,
    },
    Cmd {
        name: "opacity",
        usage: "opacity <in> <out> -p 0..100",
        group: "Color",
        vals: &["--percent", "-p", "--value", "-v"],
        bools: &[],
        build: b_opacity,
    },
    Cmd {
        name: "posterize",
        usage: "posterize <in> <out> -l LEVELS",
        group: "Color",
        vals: &["--levels", "-l"],
        bools: &[],
        build: b_levels,
    },
    // ── Filters ──
    Cmd {
        name: "blur",
        usage: "blur <in> <out> [-s SIGMA] [--fast]",
        group: "Filters",
        vals: &["--sigma", "-s"],
        bools: &["--fast"],
        build: b_blur,
    },
    Cmd {
        name: "sharpen",
        usage: "sharpen <in> <out> [-s SIGMA] [-t THRESHOLD]",
        group: "Filters",
        vals: &["--sigma", "-s", "--threshold", "-t"],
        bools: &[],
        build: b_sharpen,
    },
    Cmd {
        name: "pixelate",
        usage: "pixelate <in> <out> [-s BLOCK]",
        group: "Filters",
        vals: &["--size", "-s"],
        bools: &[],
        build: b_pixelate,
    },
    Cmd {
        name: "grain",
        usage: "grain <in> <out> [-a AMOUNT]",
        group: "Filters",
        vals: &["--amount", "-a"],
        bools: &[],
        build: b_grain,
    },
    Cmd {
        name: "edge",
        usage: "edge <in> <out>",
        group: "Filters",
        vals: &[],
        bools: &[],
        build: b_none,
    },
    Cmd {
        name: "emboss",
        usage: "emboss <in> <out> [--bias N]",
        group: "Filters",
        vals: &["--bias"],
        bools: &[],
        build: b_emboss,
    },
];

fn find_cmd(name: &str) -> Option<&'static Cmd> {
    let key = normalize(name);
    CMDS.iter().find(|c| c.name == key)
}

const OP_ALIASES: &[(&str, &str)] = &[
    ("gray", "grayscale"),
    ("greyscale", "grayscale"),
    ("negate", "invert"),
    ("mosaic", "pixelate"),
    ("unsharpen", "sharpen"),
    ("fade", "opacity"),
    ("scale", "resize"),
    ("mirror", "flip"),
    ("noise", "grain"),
];

fn find_op(name: &str) -> Option<&'static Op> {
    let mut key = normalize(name);
    if let Some((_, target)) = OP_ALIASES.iter().find(|(a, _)| normalize(a) == key) {
        key = target.to_string();
    }
    OPS.iter().find(|o| o.name == key)
}

/// Load an op for a table command (command names are op names).
fn cmd_op(name: &str) -> io::Result<&'static Op> {
    find_op(name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("op '{name}' not registered")))
}

fn run_cmd(c: &'static Cmd, args: &[String]) -> io::Result<()> {
    if args.iter().any(|a| a == "--help") {
        println!("ll image {}", c.usage);
        let op = cmd_op(c.name)?;
        if !op.spec.is_empty() {
            println!("  apply spec:  {}:{}", op.name, op.spec);
        }
        println!("  {}", op.desc);
        println!();
        println!("  Common: --quality N (jpeg), --format NAME, --background COLOR (flatten alpha)");
        println!("  Positional shortcut: ll image <op> <in> <out> <arg> (flags are optional)");
        return Ok(());
    }

    let vals = merge(SAVE_VALS, c.vals);
    let bools = merge(SAVE_BOOLS, c.bools);
    let o = parse_opts(args, &vals, &bools)?;

    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    let spec = (c.build)(&o)?;

    let img = read_image(input)?;
    let (w0, h0) = (img.width(), img.height());
    let out = (cmd_op(c.name)?.run)(img, &spec)?;
    Save::from_opts(&o)?.write(&out, output)?;
    println!("{input} ({w0}x{h0}) -> {output} ({}x{})", out.width(), out.height());
    Ok(())
}

// ─── Spec builders ───────────────────────────────────────────────────────────

fn signed(v: &str) -> String {
    if v.starts_with('-') || v.starts_with('+') {
        v.to_string()
    } else {
        format!("+{v}")
    }
}

fn b_none(_: &Opts) -> io::Result<String> {
    Ok(String::new())
}

fn b_value(o: &Opts) -> io::Result<String> {
    arg_or(o, &["--value", "-v"], 2)
        .map(str::to_string)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing --value (e.g. -v 30)"))
}

fn b_crop(o: &Opts) -> io::Result<String> {
    if let Some(g) = arg_or(o, &["--geometry", "-g"], 2) {
        return Ok(g.to_string());
    }
    let w = req(o, &["--width", "-w"], "crop needs -g WxH+X+Y (or -w/-h)")?;
    let h = req(o, &["--height", "-h"], "crop needs -h HEIGHT (or -g WxH+X+Y)")?;
    let x = o.get(&["--x", "-x"]).unwrap_or("0");
    let y = o.get(&["--y", "-y"]).unwrap_or("0");
    Ok(format!("{w}x{h}{}{}", signed(x), signed(y)))
}

fn b_resize(o: &Opts) -> io::Result<String> {
    let geom = if let Some(p) = o.get(&["--percent", "-p"]) {
        format!("{}%", p.trim().trim_end_matches('%'))
    } else {
        match (o.get(&["--width", "-w"]), o.get(&["--height", "-h"])) {
            (None, None) => match o.pos.get(2) {
                Some(g) => g.clone(),
                None => return err("resize needs -p PERCENT, -w WIDTH, -h HEIGHT or a size (800x600)"),
            },
            (Some(w), None) => format!("{w}x_"),
            (None, Some(h)) => format!("_{h}"),
            (Some(w), Some(h)) => format!("{w}x{h}"),
        }
    };
    let filter = o.get(&["--filter"]).unwrap_or("lanczos3");
    let mode = o.get(&["--mode"]).unwrap_or("fit");
    Ok(format!("{geom}:{filter}:{mode}"))
}

fn b_thumbnail(o: &Opts) -> io::Result<String> {
    arg_or(o, &["--size", "-s"], 2)
        .map(str::to_string)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "thumbnail needs -s N|WxH (e.g. -s 256)"))
}

fn b_rotate(o: &Opts) -> io::Result<String> {
    let a = arg_or(o, &["--angle", "-a"], 2).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "rotate needs -a DEGREES (e.g. -a 90)")
    })?;
    match o.get(&["--background", "-b"]) {
        Some(bg) if !bg.trim().is_empty() => Ok(format!("{a}:{bg}")),
        _ => Ok(a.to_string()),
    }
}

fn b_flip(o: &Opts) -> io::Result<String> {
    if let Some(d) = arg_or(o, &["--direction", "-d"], 2) {
        return Ok(d.to_string());
    }
    let both = o.has(&["--both"]);
    let h = o.has(&["--horizontal"]) || both;
    let v = o.has(&["--vertical"]) || both;
    match (h, v) {
        (true, true) => Ok("both".into()),
        (true, false) => Ok("h".into()),
        (false, true) => Ok("v".into()),
        (false, false) => err("flip needs --horizontal and/or --vertical (or -d h|v|both)"),
    }
}

fn b_pad(o: &Opts) -> io::Result<String> {
    let w = o.get(&["--width", "-w"]);
    let h = o.get(&["--height", "-h"]);
    let geom = if w.is_none() && h.is_none() {
        match o.pos.get(2) {
            Some(g) => g.clone(),
            None => return err("pad needs -w WIDTH and/or -h HEIGHT (or a size like 1000x800)"),
        }
    } else {
        format!("{}x{}", w.unwrap_or("_"), h.unwrap_or("_"))
    };
    let color = o.get(&["--color", "-c"]).unwrap_or("white");
    let gravity = o.get(&["--gravity"]).unwrap_or("center");
    Ok(format!("{geom}:{color}:{gravity}"))
}

fn b_border(o: &Opts) -> io::Result<String> {
    let w = arg_or(o, &["--width", "-w"], 2).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "border needs -w PX (e.g. -w 10)")
    })?;
    let color = o.get(&["--color", "-c"]).unwrap_or("black");
    Ok(format!("{w}:{color}"))
}

fn b_trim(o: &Opts) -> io::Result<String> {
    let no_flags = o.get(&["--color", "-c"]).is_none() && o.get(&["--tolerance"]).is_none();
    if let Some(spec) = o.pos.get(2).filter(|_| no_flags) {
        return Ok(spec.clone());
    }
    let color = o.get(&["--color", "-c"]).unwrap_or("auto");
    let tol = o.get(&["--tolerance"]).unwrap_or("10");
    Ok(format!("{color}:{tol}"))
}

fn b_opacity(o: &Opts) -> io::Result<String> {
    arg_or(o, &["--percent", "-p", "--value", "-v"], 2)
        .map(str::to_string)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "opacity needs -p PERCENT (0..100)"))
}

fn b_levels(o: &Opts) -> io::Result<String> {
    arg_or(o, &["--levels", "-l"], 2)
        .map(str::to_string)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "posterize needs -l LEVELS (>= 2)"))
}

fn b_blur(o: &Opts) -> io::Result<String> {
    let sigma = arg_or(o, &["--sigma", "-s"], 2).unwrap_or("2");
    let fast = if o.has(&["--fast"]) { ":fast" } else { "" };
    Ok(format!("{sigma}{fast}"))
}

fn b_sharpen(o: &Opts) -> io::Result<String> {
    let no_flags = o.get(&["--sigma", "-s"]).is_none() && o.get(&["--threshold", "-t"]).is_none();
    if let Some(spec) = o.pos.get(2).filter(|_| no_flags) {
        return Ok(spec.clone());
    }
    let sigma = o.get(&["--sigma", "-s"]).unwrap_or("2");
    let threshold = o.get(&["--threshold", "-t"]).unwrap_or("3");
    Ok(format!("{sigma}:{threshold}"))
}

fn b_pixelate(o: &Opts) -> io::Result<String> {
    Ok(arg_or(o, &["--size", "-s"], 2).unwrap_or("16").to_string())
}

fn b_grain(o: &Opts) -> io::Result<String> {
    Ok(arg_or(o, &["--amount", "-a"], 2).unwrap_or("40").to_string())
}

fn b_emboss(o: &Opts) -> io::Result<String> {
    Ok(arg_or(o, &["--bias"], 2).unwrap_or("128").to_string())
}

// ─── Multi-file / tool commands ──────────────────────────────────────────────

fn cmd_info(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, &[], &["--json"])?;
    if o.pos.is_empty() {
        return err("info: give at least one file (ll image info photo.png)");
    }
    let json_mode = o.has(&["--json"]);
    let mut objects = Vec::new();

    for (i, p) in o.pos.iter().enumerate() {
        let img = read_image(p)?;
        let (w, h) = (img.width(), img.height());
        let color = img.color();
        let format = ImageFormat::from_path(p)
            .ok()
            .map(|f| format!("{f:?}").to_uppercase());
        let bytes = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        let mp = (w as f64 * h as f64) / 1_000_000.0;
        let aspect = aspect_str(w, h);

        if json_mode {
            objects.push(json!({
                "path": p,
                "format": format,
                "width": w,
                "height": h,
                "megapixels": (mp * 100.0).round() / 100.0,
                "aspect": aspect,
                "color": format!("{color:?}"),
                "alpha": color.has_alpha(),
                "bytes": bytes,
            }));
        } else {
            if i > 0 {
                println!();
            }
            println!("{p}");
            println!("  format    {}", format.as_deref().unwrap_or("unknown"));
            println!("  size      {w} x {h} px  ({mp:.2} MP, aspect {aspect})");
            println!(
                "  color     {color:?}{}",
                if color.has_alpha() { " (alpha)" } else { "" }
            );
            println!("  file      {}", fmt_bytes(bytes));
        }
    }

    if json_mode {
        let text = serde_json::to_string_pretty(&objects).map_err(io::Error::other)?;
        println!("{text}");
    }
    Ok(())
}

fn aspect_str(w: u32, h: u32) -> String {
    if w == 0 || h == 0 {
        return "0:0".into();
    }
    let g = gcd(w, h);
    let (a, b) = (w / g, h / g);
    if a <= 999 && b <= 999 {
        format!("{a}:{b}")
    } else {
        format!("{:.2}:1", w as f64 / h as f64)
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn fmt_bytes(b: u64) -> String {
    const KB: u64 = 1 << 10;
    const MB: u64 = 1 << 20;
    const GB: u64 = 1 << 30;
    if b >= GB {
        format!("{:.2} GB ({b} bytes)", b as f64 / GB as f64)
    } else if b >= MB {
        format!("{:.1} MB ({b} bytes)", b as f64 / MB as f64)
    } else if b >= KB {
        format!("{:.1} KB ({b} bytes)", b as f64 / KB as f64)
    } else {
        format!("{b} bytes")
    }
}

fn cmd_slice(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--cols", "--rows", "--tile"], SAVE_VALS);
    let o = parse_opts(args, &vals, &[])?;
    let input = pos(&o, 0, "input file")?;
    let img = read_image(input)?;
    let (w, h) = (img.width(), img.height());

    let (tw, th, cols, rows) = if let Some(t) = o.get(&["--tile"]) {
        let (a, b) = match t.split_once('x') {
            Some((x, y)) => (x.to_string(), y.to_string()),
            None => (t.to_string(), t.to_string()),
        };
        let tw = resolve_len(&a, w)?;
        let th = resolve_len(&b, h)?;
        (tw, th, w.div_ceil(tw).max(1), h.div_ceil(th).max(1))
    } else {
        let cols = match o.get(&["--cols"]) {
            Some(v) => parse_u32(v, "--cols")?.max(1),
            None => 1,
        };
        let rows = match o.get(&["--rows"]) {
            Some(v) => parse_u32(v, "--rows")?.max(1),
            None => 1,
        };
        if o.get(&["--cols"]).is_none() && o.get(&["--rows"]).is_none() {
            return err("slice needs --cols/--rows (grid) or --tile WxH");
        }
        (w.div_ceil(cols).max(1), h.div_ceil(rows).max(1), cols, rows)
    };

    let fallback_ext = Path::new(input)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_else(|| "png".into());
    let ext = match o.get(&["--format", "-f"]) {
        Some(f) => format_extension(parse_format(f)?)?.to_string(),
        None => fallback_ext,
    };
    let prefix = match o.pos.get(1) {
        Some(p) => p.clone(),
        None => {
            let stem = Path::new(input)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "tile".into());
            format!("{stem}-tile")
        }
    };

    let save = Save::from_opts(&o)?;
    let mut written = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let (x, y) = (c * tw, r * th);
            if x >= w || y >= h {
                continue;
            }
            let cw = tw.min(w - x);
            let ch = th.min(h - y);
            let tile = img.crop_imm(x, y, cw, ch);
            let path = tile_path(&prefix, r, c, &ext);
            save.write(&tile, &path)?;
            written.push(path);
        }
    }

    println!(
        "sliced {input} ({w}x{h}) into {} tiles, {cols}x{rows} grid of {tw}x{th}",
        written.len()
    );
    if written.len() <= 16 {
        for p in &written {
            println!("  {p}");
        }
    } else {
        println!("  first: {}", written[0]);
    }
    Ok(())
}

fn tile_path(prefix: &str, row: u32, col: u32, ext: &str) -> String {
    // `prefix` may carry its own image extension: "tiles/x.jpg" -> "tiles/x-0-1.jpg".
    let (base, ext) = match Path::new(prefix).extension() {
        Some(e) if ImageFormat::from_extension(e).is_some() => {
            let e = e.to_string_lossy().into_owned();
            let path = Path::new(prefix);
            let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned());
            let dir = path.parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
            let base = match stem {
                Some(s) if dir.is_empty() => s,
                Some(s) => format!("{dir}/{s}"),
                None => prefix.to_string(),
            };
            (base, e)
        }
        _ => (prefix.to_string(), ext.to_string()),
    };

    let name = if base.contains("{r}") || base.contains("{c}") {
        base.replace("{r}", &row.to_string()).replace("{c}", &col.to_string())
    } else {
        format!("{base}-{row}-{col}")
    };
    format!("{name}.{ext}")
}

fn cmd_montage(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--out", "-o", "--cols", "--gap", "--thumb"], SAVE_VALS);
    let o = parse_opts(args, &vals, &[])?;
    let output = req(&o, &["--out", "-o"], "montage needs --out <sheet file>")?.to_string();
    let inputs = expand_inputs(&o.pos)?;
    if inputs.is_empty() {
        return err("montage: no input images");
    }

    let mut imgs = Vec::with_capacity(inputs.len());
    for p in &inputs {
        imgs.push(read_image(p)?);
    }
    if let Some(t) = o.get(&["--thumb"]) {
        let (tw, th) = parse_box(t)?;
        imgs = imgs.iter().map(|i| i.thumbnail(tw, th)).collect();
    }

    let n = imgs.len();
    let cell_w = imgs.iter().map(|i| i.width()).max().unwrap_or(1);
    let cell_h = imgs.iter().map(|i| i.height()).max().unwrap_or(1);
    let cols = match o.get(&["--cols"]) {
        Some(v) => parse_u32(v, "--cols")?.max(1) as usize,
        None => ((n as f64).sqrt().ceil() as usize).max(1),
    };
    let rows = n.div_ceil(cols);
    let gap: u32 = match o.get(&["--gap"]) {
        Some(v) => parse_u32(v, "--gap")?,
        None => 8,
    };
    let bg = parse_color(o.get(&["--background", "-b"]).unwrap_or("transparent"))?;

    let canvas_w = gap * (cols as u32 + 1) + cols as u32 * cell_w;
    let canvas_h = gap * (rows as u32 + 1) + rows as u32 * cell_h;
    let mut canvas = RgbaImage::from_pixel(canvas_w, canvas_h, bg);

    for (i, im) in imgs.iter().enumerate() {
        let r = (i / cols) as i64;
        let c = (i % cols) as i64;
        let x = gap as i64 + c * (cell_w as i64 + gap as i64) + (cell_w - im.width()) as i64 / 2;
        let y = gap as i64 + r * (cell_h as i64 + gap as i64) + (cell_h - im.height()) as i64 / 2;
        imageops::overlay(&mut canvas, &im.to_rgba8(), x, y);
    }

    Save::from_opts(&o)?.write(&DynamicImage::ImageRgba8(canvas), &output)?;
    println!(
        "montage: {n} image(s) -> {output} ({canvas_w}x{canvas_h}, {cols}x{rows} grid)"
    );
    Ok(())
}

fn cmd_convert(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--out-dir"], SAVE_VALS);
    let o = parse_opts(args, &vals, &[])?;
    if o.pos.is_empty() {
        return err("usage: ll image convert <in> <out>   |   ll image convert <in...> --out-dir DIR");
    }
    let save = Save::from_opts(&o)?;

    if let Some(dir) = o.get(&["--out-dir"]).map(str::to_string) {
        fs::create_dir_all(&dir)
            .map_err(|e| io::Error::new(e.kind(), format!("cannot create '{dir}': {e}")))?;
        let mut n = 0;
        for p in &o.pos {
            let img = read_image(p)?;
            let ext = match &save.format {
                Some(f) => format_extension(parse_format(f)?)?.to_string(),
                None => Path::new(p)
                    .extension()
                    .map(|e| e.to_string_lossy().into_owned())
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("cannot infer format of '{p}', pass --format"),
                        )
                    })?,
            };
            let stem = Path::new(p)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| format!("image{n}"));
            let out = format!("{dir}/{stem}.{ext}");
            save.write(&img, &out)?;
            println!("{p} -> {out}");
            n += 1;
        }
        println!("converted {n} file(s) into {dir}/");
    } else {
        if o.pos.len() != 2 {
            return err("convert needs <in> <out> (or several inputs with --out-dir DIR)");
        }
        let img = read_image(&o.pos[0])?;
        let (w, h) = (img.width(), img.height());
        save.write(&img, &o.pos[1])?;
        println!("{} ({w}x{h}) -> {} ({w}x{h})", o.pos[0], o.pos[1]);
    }
    Ok(())
}

fn cmd_view(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--max-width", "--max-height"], SAVE_VALS);
    let o = parse_opts(args, &vals, &["--nearest", "--upscale"])?;
    let input = pos(&o, 0, "input file")?;
    let img = read_image(input)?;

    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let max_w: u32 = match o.get(&["--max-width"]) {
        Some(v) => parse_u32(v, "--max-width")?,
        None => cols as u32,
    };
    let max_h: u32 = match o.get(&["--max-height"]) {
        Some(v) => parse_u32(v, "--max-height")?,
        None => (rows.saturating_sub(2) as u32) * 2,
    };

    let (w, h) = (img.width(), img.height());
    let mut scale = (max_w as f64 / w as f64).min(max_h as f64 / h as f64);
    if !o.has(&["--upscale"]) {
        scale = scale.min(1.0);
    }
    let (sw, sh) = if scale >= 1.0 {
        (w, h)
    } else {
        (
            ((w as f64 * scale).round() as u32).max(1),
            ((h as f64 * scale).round() as u32).max(1),
        )
    };

    let filter = if o.has(&["--nearest"]) {
        FilterType::Nearest
    } else {
        FilterType::Triangle
    };
    let shown = if (sw, sh) == (w, h) {
        img
    } else {
        imageops::resize(&img, sw, sh, filter).into()
    };

    let bg = parse_color(o.get(&["--background", "-b"]).unwrap_or("black"))?;
    let flat = flatten(&shown, bg);
    let buf = flat.to_rgba8();

    println!("{input}  {w}x{h}  ->  shown {sw}x{sh}  (truecolor ANSI preview, ▀ half blocks)");
    let mut line = String::with_capacity(sw as usize * 24 + 8);
    for y in (0..sh).step_by(2) {
        line.clear();
        for x in 0..sw {
            let top = buf.get_pixel(x, y).0;
            let bot = if y + 1 < sh { buf.get_pixel(x, y + 1).0 } else { top };
            use std::fmt::Write as _;
            let _ = write!(
                line,
                "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
                top[0], top[1], top[2], bot[0], bot[1], bot[2]
            );
        }
        line.push_str("\x1b[0m");
        println!("{line}");
    }
    Ok(())
}

fn cmd_apply(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, SAVE_VALS, SAVE_BOOLS)?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    let steps = &o.pos[2..];
    if steps.is_empty() {
        return err(
            "apply: no operations given\n  example: ll image apply in.png out.webp crop:80%x80% grayscale blur:2",
        );
    }

    let mut img = read_image(input)?;
    let (w0, h0) = (img.width(), img.height());
    for step in steps {
        let (name, spec) = match step.split_once(':') {
            Some((n, s)) => (n, s),
            None => (step.as_str(), ""),
        };
        let op = find_op(name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown op '{name}' — see 'll image list'"),
            )
        })?;
        let (iw, ih) = (img.width(), img.height());
        img = (op.run)(img, spec)?;
        println!("  {:<12} {iw}x{ih} -> {}x{}", op.name, img.width(), img.height());
    }
    Save::from_opts(&o)?.write(&img, output)?;
    println!("{input} ({w0}x{h0}) -> {output} ({}x{}) in {} step(s)", img.width(), img.height(), steps.len());
    Ok(())
}

fn expand_inputs(pos: &[String]) -> io::Result<Vec<String>> {
    let mut out = Vec::new();
    for p in pos {
        let path = Path::new(p);
        if path.is_dir() {
            let mut files: Vec<String> = fs::read_dir(path)
                .map_err(|e| io::Error::new(e.kind(), format!("cannot read '{p}': {e}")))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|q| is_image_path(q))
                .map(|q| q.to_string_lossy().into_owned())
                .collect();
            files.sort();
            if files.is_empty() {
                return err(format!("no images found in '{p}'"));
            }
            out.extend(files);
        } else {
            out.push(p.clone());
        }
    }
    Ok(out)
}

fn is_image_path(p: &Path) -> bool {
    p.extension()
        .map(|e| ImageFormat::from_extension(e).is_some())
        .unwrap_or(false)
}

// ─── I/O ─────────────────────────────────────────────────────────────────────

fn read_image(path: &str) -> io::Result<DynamicImage> {
    image::open(path)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("cannot read '{path}': {e}")))
}

struct Save {
    quality: Option<u8>,
    format: Option<String>,
    background: Rgba<u8>,
}

impl Save {
    fn from_opts(o: &Opts) -> io::Result<Self> {
        let quality = match o.get(&["--quality", "-q"]) {
            Some(v) => {
                let q = parse_u32(v, "--quality")?;
                if !(1..=100).contains(&q) {
                    return err(format!("--quality must be 1..100, got '{v}'"));
                }
                Some(q as u8)
            }
            None => None,
        };
        let format = o.get(&["--format", "-f"]).map(str::to_string);
        if let Some(f) = &format {
            parse_format(f)?;
        }
        let background = match o.get(&["--background", "-b"]) {
            Some(c) => parse_color(c)?,
            None => Rgba([255, 255, 255, 255]),
        };
        Ok(Save { quality, format, background })
    }

    fn write(&self, img: &DynamicImage, path: &str) -> io::Result<()> {
        if let Some(parent) = Path::new(path).parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|e| {
                io::Error::new(e.kind(), format!("cannot create directory '{}': {e}", parent.display()))
            })?;
        }        let fmt = match &self.format {
            Some(f) => parse_format(f)?,
            None => ImageFormat::from_path(path).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("cannot infer output format from '{path}' — use --format or a known extension"),
                )
            })?,
        };

        let flat;
        let src: &DynamicImage = if !format_has_alpha(fmt) && img.has_alpha() {
            flat = flatten(img, self.background);
            &flat
        } else {
            img
        };

        let file = BufWriter::new(
            File::create(path)
                .map_err(|e| io::Error::new(e.kind(), format!("cannot create '{path}': {e}")))?,
        );
        let result = match (fmt, self.quality) {
            (ImageFormat::Jpeg, Some(q)) => src.write_with_encoder(JpegEncoder::new_with_quality(file, q)),
            _ => src.write_to(file, fmt),
        };
        result.map_err(|e| io::Error::other(format!("failed to write '{path}': {e}")))
    }
}

fn format_has_alpha(fmt: ImageFormat) -> bool {
    !matches!(fmt, ImageFormat::Jpeg | ImageFormat::Bmp | ImageFormat::Pnm | ImageFormat::Hdr)
}

fn parse_format(s: &str) -> io::Result<ImageFormat> {
    match normalize(s).as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "webp" => Ok(ImageFormat::WebP),
        "gif" => Ok(ImageFormat::Gif),
        "bmp" => Ok(ImageFormat::Bmp),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        "ico" => Ok(ImageFormat::Ico),
        "pnm" | "ppm" | "pgm" | "pam" => Ok(ImageFormat::Pnm),
        "qoi" => Ok(ImageFormat::Qoi),
        "tga" => Ok(ImageFormat::Tga),
        other => err(format!(
            "unsupported format '{other}' (png, jpg, webp, gif, bmp, tiff, ico, pnm, qoi, tga)"
        )),
    }
}

fn format_extension(fmt: ImageFormat) -> io::Result<&'static str> {
    match fmt {
        ImageFormat::Png => Ok("png"),
        ImageFormat::Jpeg => Ok("jpg"),
        ImageFormat::WebP => Ok("webp"),
        ImageFormat::Gif => Ok("gif"),
        ImageFormat::Bmp => Ok("bmp"),
        ImageFormat::Tiff => Ok("tiff"),
        ImageFormat::Ico => Ok("ico"),
        ImageFormat::Pnm => Ok("pnm"),
        ImageFormat::Qoi => Ok("qoi"),
        ImageFormat::Tga => Ok("tga"),
        other => err(format!("no file extension for {other:?}")),
    }
}

fn flatten(img: &DynamicImage, bg: Rgba<u8>) -> DynamicImage {
    let src = img.to_rgba8();
    let mut out = RgbaImage::from_pixel(src.width(), src.height(), bg);
    imageops::overlay(&mut out, &src, 0, 0);
    DynamicImage::ImageRgba8(out)
}

fn parse_color(s: &str) -> io::Result<Rgba<u8>> {
    let t = s.trim().to_ascii_lowercase();
    let named: Option<[u8; 4]> = match t.as_str() {
        "transparent" | "none" => Some([0, 0, 0, 0]),
        "black" => Some([0, 0, 0, 255]),
        "white" => Some([255, 255, 255, 255]),
        "red" => Some([255, 0, 0, 255]),
        "green" => Some([0, 128, 0, 255]),
        "lime" => Some([0, 255, 0, 255]),
        "blue" => Some([0, 0, 255, 255]),
        "yellow" => Some([255, 255, 0, 255]),
        "cyan" | "aqua" => Some([0, 255, 255, 255]),
        "magenta" | "fuchsia" => Some([255, 0, 255, 255]),
        "gray" | "grey" => Some([128, 128, 128, 255]),
        "silver" => Some([192, 192, 192, 255]),
        "maroon" => Some([128, 0, 0, 255]),
        "olive" => Some([128, 128, 0, 255]),
        "navy" => Some([0, 0, 128, 255]),
        "teal" => Some([0, 128, 128, 255]),
        "purple" => Some([128, 0, 128, 255]),
        "orange" => Some([255, 165, 0, 255]),
        "gold" => Some([255, 215, 0, 255]),
        "pink" => Some([255, 192, 203, 255]),
        "brown" => Some([165, 42, 42, 255]),
        "violet" => Some([238, 130, 238, 255]),
        "indigo" => Some([75, 0, 130, 255]),
        "coral" => Some([255, 127, 80, 255]),
        "crimson" => Some([220, 20, 60, 255]),
        "tan" => Some([210, 180, 140, 255]),
        "beige" => Some([245, 245, 220, 255]),
        "khaki" => Some([240, 230, 140, 255]),
        "salmon" => Some([250, 128, 114, 255]),
        "chocolate" => Some([210, 105, 30, 255]),
        "turquoise" => Some([64, 224, 208, 255]),
        "slate" => Some([112, 128, 144, 255]),
        _ => None,
    };
    if let Some(c) = named {
        return Ok(Rgba(c));
    }

    let hex = t.strip_prefix('#').unwrap_or(&t);
    if !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        let n = u32::from_str_radix(hex, 16)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("bad hex color '{s}'")))?;
        return match hex.len() {
            3 => Ok(Rgba([
                ((n >> 8) & 0xF) as u8 * 17,
                ((n >> 4) & 0xF) as u8 * 17,
                (n & 0xF) as u8 * 17,
                255,
            ])),
            4 => Ok(Rgba([
                ((n >> 12) & 0xF) as u8 * 17,
                ((n >> 8) & 0xF) as u8 * 17,
                ((n >> 4) & 0xF) as u8 * 17,
                (n & 0xF) as u8 * 17,
            ])),
            6 => Ok(Rgba([
                ((n >> 16) & 0xFF) as u8,
                ((n >> 8) & 0xFF) as u8,
                (n & 0xFF) as u8,
                255,
            ])),
            8 => Ok(Rgba([
                ((n >> 24) & 0xFF) as u8,
                ((n >> 16) & 0xFF) as u8,
                ((n >> 8) & 0xFF) as u8,
                (n & 0xFF) as u8,
            ])),
            _ => err(format!("bad hex color '{s}' (use #rgb, #rrggbb or #rrggbbaa)")),
        };
    }

    if t.contains(',') {
        let parts: Vec<&str> = t.split(',').map(str::trim).collect();
        if parts.len() == 3 || parts.len() == 4 {
            let mut v = [255u8; 4];
            for (i, p) in parts.iter().enumerate() {
                v[i] = p.parse::<u8>().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("bad color '{s}' (components must be 0..255)"),
                    )
                })?;
            }
            return Ok(Rgba(v));
        }
    }

    err(format!(
        "cannot parse color '{s}' (try '#rrggbb', 'red', '255,0,0' or 'transparent')"
    ))
}

// ─── Geometry / value parsing ────────────────────────────────────────────────

/// Split a `WxH+X+Y` geometry into its four raw tokens.
/// Offsets keep their sign; missing offsets come back empty.
fn split_geometry(spec: &str) -> io::Result<(String, String, String, String)> {
    let s = spec.trim().replacen('X', "x", 1);
    let Some(ix) = s.find('x') else {
        return err(format!("geometry must look like WxH+X+Y, got '{spec}'"));
    };
    let (w, rest) = (s[..ix].to_string(), &s[ix + 1..]);
    let ih = rest.find(['+', '-']).unwrap_or(rest.len());
    let h = rest[..ih].to_string();
    if w.trim().is_empty() || h.trim().is_empty() {
        return err(format!("geometry needs both width and height, got '{spec}'"));
    }

    let mut offs: Vec<String> = Vec::new();
    let mut tail = rest[ih..].trim();
    while !tail.is_empty() {
        let (sign, rest) = match tail.strip_prefix('+') {
            Some(r) => ('+', r),
            None => match tail.strip_prefix('-') {
                Some(r) => ('-', r),
                None => return err(format!("bad offsets in geometry '{spec}'")),
            },
        };
        let end = rest.find(['+', '-']).unwrap_or(rest.len());
        offs.push(format!("{sign}{}", &rest[..end]));
        tail = rest[end..].trim();
        if offs.len() > 2 {
            return err(format!("too many offsets in geometry '{spec}'"));
        }
    }

    Ok((
        w,
        h,
        offs.first().cloned().unwrap_or_default(),
        offs.get(1).cloned().unwrap_or_default(),
    ))
}

/// Split `WxH`. A missing `x` means width only (`500` == `500x_`), while a
/// leading `_` marks an empty width (`_270`).
fn split_wh(spec: &str) -> io::Result<(String, String)> {
    if let Some(rest) = spec.trim().strip_prefix('_') {
        return Ok(("_".into(), rest.to_string()));
    }
    match spec.trim().split_once('x') {
        Some((w, h)) => Ok((w.to_string(), h.to_string())),
        None => Ok((spec.trim().to_string(), String::new())),
    }
}

/// Resolve `123` or `45%` (percent of `total`) to a pixel length.
fn resolve_len(tok: &str, total: u32) -> io::Result<u32> {
    let t = tok.trim().trim_start_matches('+');
    if t.is_empty() || t == "_" {
        return err("empty size");
    }
    if let Some(p) = t.strip_suffix('%') {
        let pct = parse_f64(p, "percentage")?;
        if pct <= 0.0 {
            return err(format!("percentage must be > 0, got '{p}%'"));
        }
        Ok(((total as f64 * pct / 100.0).round() as u32).max(1))
    } else {
        let v = parse_u32(t, "size")?;
        if v == 0 {
            return err("size must be > 0");
        }
        Ok(v)
    }
}

/// Resolve a signed offset. `-N` counts from the right/bottom edge
/// (ImageMagick style), percentages are relative to `total`.
fn resolve_off(tok: &str, total: u32, size: u32) -> io::Result<i64> {
    let t = tok.trim();
    if t.is_empty() {
        return Ok(0);
    }
    let (neg, rest) = if let Some(r) = t.strip_prefix('-') {
        (true, r)
    } else {
        (false, t.trim_start_matches('+'))
    };
    let v = if let Some(p) = rest.strip_suffix('%') {
        let pct = parse_f64(p, "percentage")?;
        (total as f64 * pct / 100.0).round() as i64
    } else {
        parse_u32(rest, "offset")? as i64
    };
    Ok(if neg {
        total as i64 - size as i64 - v
    } else {
        v
    })
}

fn parse_filter(s: &str) -> io::Result<FilterType> {
    match normalize(s).as_str() {
        "nearest" | "point" => Ok(FilterType::Nearest),
        "triangle" | "linear" | "bilinear" => Ok(FilterType::Triangle),
        "catmullrom" | "catmull" | "cubic" => Ok(FilterType::CatmullRom),
        "gaussian" => Ok(FilterType::Gaussian),
        "lanczos3" | "lanczos" => Ok(FilterType::Lanczos3),
        other => err(format!(
            "unknown filter '{other}' (nearest|triangle|catmullrom|gaussian|lanczos3)"
        )),
    }
}

/// Parse `256` (square) or `WxH` box bounds.
fn parse_box(s: &str) -> io::Result<(u32, u32)> {
    let (a, b) = match s.split_once('x') {
        Some((w, h)) => (resolve_len(w, u32::MAX)?, resolve_len(h, u32::MAX)?),
        None => (resolve_len(s, u32::MAX)?, resolve_len(s, u32::MAX)?),
    };
    Ok((a, b))
}

fn luma(c: [u8; 3]) -> f32 {
    0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32
}

fn clamp_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// Apply `f` to the color channels of every pixel, preserving alpha and
/// keeping the image non-alpha when it already was.
fn map_rgb(img: &DynamicImage, mut f: impl FnMut([u8; 3]) -> [u8; 3]) -> DynamicImage {
    if img.has_alpha() {
        let mut buf = img.to_rgba8();
        for p in buf.pixels_mut() {
            let q = f([p.0[0], p.0[1], p.0[2]]);
            p.0[0] = q[0];
            p.0[1] = q[1];
            p.0[2] = q[2];
        }
        DynamicImage::ImageRgba8(buf)
    } else {
        let mut buf = img.to_rgb8();
        for p in buf.pixels_mut() {
            p.0 = f(p.0);
        }
        DynamicImage::ImageRgb8(buf)
    }
}

// ─── Operator registry ───────────────────────────────────────────────────────

type OpFn = fn(DynamicImage, &str) -> io::Result<DynamicImage>;

struct Op {
    name: &'static str,
    spec: &'static str,
    desc: &'static str,
    run: OpFn,
}

static OPS: &[Op] = &[
    Op { name: "crop", spec: "WxH+X+Y", desc: "Cut a rectangle (px or %, -X from right)", run: op_crop },
    Op { name: "resize", spec: "[WxH|N%][:filter][:fit|fill|stretch]", desc: "Scale to a size or percent", run: op_resize },
    Op { name: "thumbnail", spec: "N|WxH", desc: "Fast aspect-preserving shrink to fit", run: op_thumbnail },
    Op { name: "rotate", spec: "degrees[:color]", desc: "Rotate any angle, alpha-aware fill", run: op_rotate },
    Op { name: "flip", spec: "h|v|both", desc: "Mirror horizontally / vertically / both", run: op_flip },
    Op { name: "grayscale", spec: "", desc: "Remove color, keep alpha", run: op_grayscale },
    Op { name: "invert", spec: "", desc: "Photographic negative", run: op_invert },
    Op { name: "sepia", spec: "", desc: "Warm vintage tone", run: op_sepia },
    Op { name: "brightness", spec: "-255..255", desc: "Linear brightness offset", run: op_brightness },
    Op { name: "contrast", spec: "-100..100", desc: "Percent contrast", run: op_contrast },
    Op { name: "saturate", spec: "-100..300", desc: "Saturation: -100 = gray, 0 = keep, 200 = double", run: op_saturate },
    Op { name: "hue", spec: "-180..180", desc: "Rotate the hue wheel", run: op_hue },
    Op { name: "opacity", spec: "0..100", desc: "Scale the alpha channel (fade)", run: op_opacity },
    Op { name: "posterize", spec: "levels>=2", desc: "Quantize channels to N levels", run: op_posterize },
    Op { name: "blur", spec: "sigma[:fast]", desc: "Gaussian blur", run: op_blur },
    Op { name: "sharpen", spec: "sigma[:threshold]", desc: "Unsharp mask", run: op_sharpen },
    Op { name: "pixelate", spec: "block", desc: "Mosaic / censored block look", run: op_pixelate },
    Op { name: "grain", spec: "amount", desc: "Deterministic film grain", run: op_grain },
    Op { name: "edge", spec: "", desc: "Sobel edge detector", run: op_edge },
    Op { name: "emboss", spec: "[bias]", desc: "Emboss relief (bias, default 128)", run: op_emboss },
    Op { name: "pad", spec: "WxH[:color][:gravity]", desc: "Extend the canvas (_ = keep)", run: op_pad },
    Op { name: "border", spec: "width[:color]", desc: "Add a solid frame", run: op_border },
    Op { name: "trim", spec: "[color][:tolerance]", desc: "Auto-crop uniform edges", run: op_trim },
];

// ─── Operators ───────────────────────────────────────────────────────────────

fn op_crop(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let (wt, ht, xt, yt) = split_geometry(spec)?;
    let (w, h) = (img.width(), img.height());
    let cw = resolve_len(&wt, w)?.min(w);
    let ch = resolve_len(&ht, h)?.min(h);
    let x = resolve_off(&xt, w, cw)?.clamp(0, (w - cw) as i64) as u32;
    let y = resolve_off(&yt, h, ch)?.clamp(0, (h - ch) as i64) as u32;
    Ok(img.crop_imm(x, y, cw, ch))
}

fn op_resize(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let parts: Vec<&str> = spec.split(':').map(str::trim).collect();
    let geom = parts.first().copied().unwrap_or("");
    if geom.is_empty() {
        return err("resize: missing size (e.g. '800x600', '50%' or '800x_')");
    }
    let filter = parts
        .get(1)
        .copied()
        .filter(|s| !s.is_empty())
        .map(parse_filter)
        .transpose()?
        .unwrap_or(FilterType::Lanczos3);
    let mode = parts.get(2).copied().filter(|s| !s.is_empty()).unwrap_or("fit");

    if let Some(p) = geom.strip_suffix('%') {
        let pct = parse_f64(p, "percentage")?;
        if pct <= 0.0 {
            return err(format!("resize: percentage must be > 0, got '{p}%'"));
        }
        let nw = ((img.width() as f64 * pct / 100.0).round() as u32).max(1);
        let nh = ((img.height() as f64 * pct / 100.0).round() as u32).max(1);
        return Ok(DynamicImage::ImageRgba8(imageops::resize(&img, nw, nh, filter)));
    }

    let (wt, ht) = split_wh(geom)?;
    let has_w = !wt.is_empty() && wt != "_";
    let has_h = !ht.is_empty() && ht != "_";
    let (w, h) = (img.width(), img.height());

    match (has_w, has_h) {
        (true, true) => {
            let tw = resolve_len(&wt, w)?;
            let th = resolve_len(&ht, h)?;
            match mode {
                "fit" => Ok(img.resize(tw, th, filter)),
                "fill" => Ok(img.resize_to_fill(tw, th, filter)),
                "stretch" | "exact" => Ok(img.resize_exact(tw, th, filter)),
                other => err(format!("resize: unknown mode '{other}' (fit|fill|stretch)")),
            }
        }
        (true, false) => {
            let nw = resolve_len(&wt, w)?;
            let nh = scale_dim(h, w, nw);
            Ok(DynamicImage::ImageRgba8(imageops::resize(&img, nw, nh, filter)))
        }
        (false, true) => {
            let nh = resolve_len(&ht, h)?;
            let nw = scale_dim(w, h, nh);
            Ok(DynamicImage::ImageRgba8(imageops::resize(&img, nw, nh, filter)))
        }
        (false, false) => err("resize: nothing to resize (use '_' to keep one side)"),
    }
}

fn scale_dim(src: u32, base: u32, target: u32) -> u32 {
    ((src as f64 * target as f64 / base as f64).round() as u32).max(1)
}

fn op_thumbnail(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let g = spec.trim();
    if g.is_empty() {
        return err("thumbnail: missing box (e.g. '256' or '800x600')");
    }
    if let Some(p) = g.strip_suffix('%') {
        let pct = parse_f64(p, "percentage")?;
        let nw = ((img.width() as f64 * pct / 100.0).round() as u32).max(1);
        let nh = ((img.height() as f64 * pct / 100.0).round() as u32).max(1);
        return Ok(img.thumbnail(nw, nh));
    }
    let (wt, ht) = split_wh(g)?;
    let w_active = !wt.is_empty() && wt != "_";
    let h_active = !ht.is_empty() && ht != "_";
    // Never enlarge: clamp the fit-box to the current size (ImageMagick -thumbnail style).
    let (bw, bh) = (
        if w_active { resolve_len(&wt, img.width())?.min(img.width()) } else { img.width() },
        if h_active { resolve_len(&ht, img.height())?.min(img.height()) } else { img.height() },
    );
    match (w_active, h_active) {
        (true, true) => Ok(img.thumbnail(bw, bh)),
        (true, false) => Ok(img.thumbnail(bw, img.height())),
        (false, true) => Ok(img.thumbnail(img.width(), bh)),
        (false, false) => err("thumbnail: empty box"),
    }
}

fn op_rotate(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let mut parts = spec.split(':');
    let angle_s = parts.next().unwrap_or("").trim();
    if angle_s.is_empty() {
        return err("rotate: missing angle (e.g. '90' or '45')");
    }
    let angle = parse_f64(angle_s, "angle")? as f32;
    let bg = match parts.next() {
        Some(s) if !s.trim().is_empty() => parse_color(s)?,
        _ => Rgba([0, 0, 0, 0]),
    };

    let a = angle.rem_euclid(360.0);
    let eps = 1e-3;
    if a < eps || (360.0 - a) < eps {
        return Ok(img);
    }
    if (a - 90.0).abs() < eps {
        return Ok(img.rotate90());
    }
    if (a - 180.0).abs() < eps {
        return Ok(img.rotate180());
    }
    if (a - 270.0).abs() < eps {
        return Ok(img.rotate270());
    }
    Ok(DynamicImage::ImageRgba8(rotate_free(&img, a, bg)))
}

fn rotate_free(img: &DynamicImage, deg: f32, bg: Rgba<u8>) -> RgbaImage {
    let src = img.to_rgba8();
    let (w, h) = (src.width(), src.height());
    let rad = deg.to_radians();
    let (s, c) = (rad.sin() as f64, rad.cos() as f64);
    let nw = ((w as f64 * c.abs() + h as f64 * s.abs()).round() as u32).max(1);
    let nh = ((w as f64 * s.abs() + h as f64 * c.abs()).round() as u32).max(1);
    let (cx, cy) = ((w as f64 - 1.0) / 2.0, (h as f64 - 1.0) / 2.0);
    let (dx0, dy0) = ((nw as f64 - 1.0) / 2.0, (nh as f64 - 1.0) / 2.0);

    let mut out = RgbaImage::from_pixel(nw, nh, bg);
    for y in 0..nh {
        for x in 0..nw {
            let dx = x as f64 - dx0;
            let dy = y as f64 - dy0;
            let sx = dx * c + dy * s + cx;
            let sy = -dx * s + dy * c + cy;
            if sx >= 0.0 && sy >= 0.0 && sx <= (w - 1) as f64 && sy <= (h - 1) as f64 {
                out.put_pixel(x, y, sample_bilinear(&src, sx as f32, sy as f32));
            }
        }
    }
    out
}

fn sample_bilinear(img: &RgbaImage, x: f32, y: f32) -> Rgba<u8> {
    let (w, h) = (img.width(), img.height());
    let x = x.clamp(0.0, (w - 1) as f32);
    let y = y.clamp(0.0, (h - 1) as f32);
    let (x0, y0) = (x.floor() as u32, y.floor() as u32);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let p00 = img.get_pixel(x0, y0).0;
    let p10 = img.get_pixel(x1, y0).0;
    let p01 = img.get_pixel(x0, y1).0;
    let p11 = img.get_pixel(x1, y1).0;
    let mut out = [0u8; 4];
    for i in 0..4 {
        let top = p00[i] as f32 * (1.0 - fx) + p10[i] as f32 * fx;
        let bot = p01[i] as f32 * (1.0 - fx) + p11[i] as f32 * fx;
        out[i] = clamp_u8(top * (1.0 - fy) + bot * fy);
    }
    Rgba(out)
}

fn op_flip(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    match normalize(spec).as_str() {
        "" | "h" | "horizontal" | "x" => Ok(img.fliph()),
        "v" | "vertical" | "y" => Ok(img.flipv()),
        "both" | "hv" | "xy" | "180" => Ok(img.rotate180()),
        other => err(format!("flip: unknown direction '{other}' (h|v|both)")),
    }
}

fn op_grayscale(img: DynamicImage, _: &str) -> io::Result<DynamicImage> {
    if img.has_alpha() {
        Ok(map_rgb(&img, |c| {
            let l = clamp_u8(luma(c));
            [l, l, l]
        }))
    } else {
        Ok(img.grayscale())
    }
}

fn op_invert(img: DynamicImage, _: &str) -> io::Result<DynamicImage> {
    if img.has_alpha() {
        Ok(map_rgb(&img, |c| [255 - c[0], 255 - c[1], 255 - c[2]]))
    } else {
        let mut img = img;
        img.invert();
        Ok(img)
    }
}

fn op_sepia(img: DynamicImage, _: &str) -> io::Result<DynamicImage> {
    Ok(map_rgb(&img, |c| {
        let (r, g, b) = (c[0] as f32, c[1] as f32, c[2] as f32);
        [
            clamp_u8(0.393 * r + 0.769 * g + 0.189 * b),
            clamp_u8(0.349 * r + 0.686 * g + 0.168 * b),
            clamp_u8(0.272 * r + 0.534 * g + 0.131 * b),
        ]
    }))
}

fn op_brightness(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let v = spec.trim();
    if v.is_empty() {
        return err("brightness: missing value (-255..255)");
    }
    let n: i32 = v
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("brightness: bad value '{v}'")))?;
    Ok(img.brighten(n))
}

fn op_contrast(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let v = spec.trim();
    if v.is_empty() {
        return err("contrast: missing value (-100..100)");
    }
    let n = parse_f64(v, "contrast")? as f32;
    Ok(img.adjust_contrast(n))
}

fn op_hue(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let v = spec.trim();
    if v.is_empty() {
        return err("hue: missing degrees (-180..180)");
    }
    let n: i32 = v
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("hue: bad value '{v}'")))?;
    Ok(img.huerotate(n))
}

fn op_saturate(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let v = spec.trim();
    if v.is_empty() {
        return err("saturate: missing percent (-100..300)");
    }
    let pct = parse_f64(v, "saturate")?;
    let f = (1.0 + pct as f32 / 100.0).max(0.0);
    Ok(map_rgb(&img, |c| {
        let l = luma(c);
        [
            clamp_u8(l + (c[0] as f32 - l) * f),
            clamp_u8(l + (c[1] as f32 - l) * f),
            clamp_u8(l + (c[2] as f32 - l) * f),
        ]
    }))
}

fn op_opacity(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let v = spec.trim();
    if v.is_empty() {
        return err("opacity: missing percent (0..100)");
    }
    let pct = parse_f64(v, "opacity")?.clamp(0.0, 100.0);
    let k = pct as f32 / 100.0;
    let mut buf = img.to_rgba8();
    for p in buf.pixels_mut() {
        p.0[3] = clamp_u8(p.0[3] as f32 * k);
    }
    Ok(DynamicImage::ImageRgba8(buf))
}

fn op_posterize(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let v = spec.trim();
    if v.is_empty() {
        return err("posterize: missing levels (>= 2)");
    }
    let levels = parse_u32(v, "posterize levels")?;
    if levels < 2 {
        return err("posterize: levels must be >= 2");
    }
    let step = 255.0 / (levels - 1) as f32;
    Ok(map_rgb(&img, |c| {
        let f = |x: u8| clamp_u8((x as f32 / step).round() * step);
        [f(c[0]), f(c[1]), f(c[2])]
    }))
}

fn op_blur(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let parts: Vec<&str> = spec.split(':').collect();
    let sigma = if parts[0].trim().is_empty() {
        2.0
    } else {
        parse_f64(parts[0], "sigma")?
    };
    if sigma <= 0.0 {
        return err("blur: sigma must be > 0");
    }
    let fast = matches!(parts.get(1), Some(s) if normalize(s) == "fast");
    if fast {
        Ok(img.fast_blur(sigma as f32))
    } else {
        Ok(img.blur(sigma as f32))
    }
}

fn op_sharpen(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let parts: Vec<&str> = spec.split(':').collect();
    let sigma = if parts[0].trim().is_empty() {
        2.0
    } else {
        parse_f64(parts[0], "sigma")?
    };
    if sigma <= 0.0 {
        return err("sharpen: sigma must be > 0");
    }
    let threshold: i32 = match parts.get(1) {
        Some(t) if !t.trim().is_empty() => t.trim().parse().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("sharpen: bad threshold '{t}'"))
        })?,
        _ => 3,
    };
    Ok(img.unsharpen(sigma as f32, threshold))
}

fn op_pixelate(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let v = spec.trim();
    let block = if v.is_empty() { 16 } else { parse_u32(v, "pixelate block")? };
    if block <= 1 {
        return Ok(img);
    }
    let (w, h) = (img.width(), img.height());
    let (dw, dh) = (w.div_ceil(block).max(1), h.div_ceil(block).max(1));
    let small = imageops::resize(&img, dw, dh, FilterType::Triangle);
    Ok(DynamicImage::ImageRgba8(imageops::resize(&small, w, h, FilterType::Nearest)))
}

fn grain_delta(x: u32, y: u32, ch: u32) -> f32 {
    let mut n = x
        .wrapping_mul(374_761_393)
        ^ y.wrapping_mul(668_265_263)
        ^ ch.wrapping_mul(1_274_126_177);
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    let v = (n ^ (n >> 16)) % 2001; // 0..2000
    (v as f32 - 1000.0) / 1000.0 // -1.0..1.0
}

fn op_grain(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let v = spec.trim();
    let amount = if v.is_empty() { 40.0 } else { parse_f64(v, "grain amount")? };
    if amount <= 0.0 {
        return Ok(img);
    }
    let amp = amount as f32 * 1.28; // amount 100 -> +-128
    if img.has_alpha() {
        let mut buf = img.to_rgba8();
        for (x, y, p) in buf.enumerate_pixels_mut() {
            for ch in 0..3 {
                p.0[ch] = clamp_u8(p.0[ch] as f32 + grain_delta(x, y, ch as u32) * amp);
            }
        }
        Ok(DynamicImage::ImageRgba8(buf))
    } else {
        let mut buf = img.to_rgb8();
        for (x, y, p) in buf.enumerate_pixels_mut() {
            for ch in 0..3 {
                p.0[ch] = clamp_u8(p.0[ch] as f32 + grain_delta(x, y, ch as u32) * amp);
            }
        }
        Ok(DynamicImage::ImageRgb8(buf))
    }
}

fn op_edge(img: DynamicImage, _: &str) -> io::Result<DynamicImage> {
    let g = img.to_luma8();
    let (w, h) = (g.width(), g.height());
    let mut out = image::GrayImage::new(w, h);
    let at = |x: i32, y: i32| -> f32 {
        let cx = x.clamp(0, w as i32 - 1) as u32;
        let cy = y.clamp(0, h as i32 - 1) as u32;
        g.get_pixel(cx, cy).0[0] as f32
    };
    for y in 0..h {
        for x in 0..w {
            let (x, y) = (x as i32, y as i32);
            let gx = (at(x + 1, y - 1) + 2.0 * at(x + 1, y) + at(x + 1, y + 1))
                - (at(x - 1, y - 1) + 2.0 * at(x - 1, y) + at(x - 1, y + 1));
            let gy = (at(x - 1, y + 1) + 2.0 * at(x, y + 1) + at(x + 1, y + 1))
                - (at(x - 1, y - 1) + 2.0 * at(x, y - 1) + at(x + 1, y - 1));
            let mag = ((gx * gx + gy * gy).sqrt() * 0.25).min(255.0);
            out.put_pixel(x as u32, y as u32, image::Luma([mag as u8]));
        }
    }
    Ok(DynamicImage::ImageLuma8(out))
}

fn op_emboss(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let bias = if spec.trim().is_empty() {
        128.0
    } else {
        parse_f64(spec.trim(), "bias")?
    };
    let k = [-2.0f32, -1.0, 0.0, -1.0, 1.0, 1.0, 0.0, 1.0, 2.0];
    let src = img.to_rgba8();
    let (w, h) = (src.width(), src.height());
    let mut out = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 3];
            for ky in 0..3u32 {
                for kx in 0..3u32 {
                    let sx = (x as i32 + kx as i32 - 1).clamp(0, w as i32 - 1) as u32;
                    let sy = (y as i32 + ky as i32 - 1).clamp(0, h as i32 - 1) as u32;
                    let p = src.get_pixel(sx, sy).0;
                    let weight = k[(ky * 3 + kx) as usize];
                    for c in 0..3 {
                        acc[c] += p[c] as f32 * weight;
                    }
                }
            }
            let orig = src.get_pixel(x, y).0;
            let q = out.get_pixel_mut(x, y);
            q.0[0] = clamp_u8(acc[0] + bias as f32);
            q.0[1] = clamp_u8(acc[1] + bias as f32);
            q.0[2] = clamp_u8(acc[2] + bias as f32);
            q.0[3] = orig[3];
        }
    }
    Ok(DynamicImage::ImageRgba8(out))
}

fn op_pad(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let parts: Vec<&str> = spec.split(':').collect();
    let (wt, ht) = split_wh(parts.first().copied().unwrap_or(""))?;
    let has_w = !wt.is_empty() && wt != "_";
    let has_h = !ht.is_empty() && ht != "_";
    if !has_w && !has_h {
        return err("pad: missing target size (e.g. '1000x800' or '1000x_')");
    }
    let (iw, ih) = (img.width(), img.height());
    let target_w = if has_w { resolve_len(&wt, iw)? } else { iw };
    let target_h = if has_h { resolve_len(&ht, ih)? } else { ih };
    let (fw, fh) = (target_w.max(iw), target_h.max(ih));

    let color = match parts.get(1) {
        Some(s) if !s.trim().is_empty() => parse_color(s)?,
        _ => Rgba([255, 255, 255, 255]),
    };
    let gravity = normalize(parts.get(2).copied().unwrap_or("center"));
    let (gx, gy) = match gravity.as_str() {
        "nw" => ("start", "start"),
        "n" => ("mid", "start"),
        "ne" => ("end", "start"),
        "w" => ("start", "mid"),
        "center" | "centre" | "c" => ("mid", "mid"),
        "e" => ("end", "mid"),
        "sw" => ("start", "end"),
        "s" => ("mid", "end"),
        "se" => ("end", "end"),
        other => {
            return err(format!(
                "pad: unknown gravity '{other}' (nw|n|ne|w|center|e|sw|s|se)"
            ))
        }
    };
    let free_x = (fw - iw) as i64;
    let free_y = (fh - ih) as i64;
    let x = match gx {
        "start" => 0,
        "end" => free_x,
        _ => free_x / 2,
    };
    let y = match gy {
        "start" => 0,
        "end" => free_y,
        _ => free_y / 2,
    };

    let mut canvas = RgbaImage::from_pixel(fw, fh, color);
    imageops::overlay(&mut canvas, &img.to_rgba8(), x, y);
    Ok(DynamicImage::ImageRgba8(canvas))
}

fn op_border(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let parts: Vec<&str> = spec.split(':').collect();
    let w_tok = parts.first().copied().unwrap_or("").trim();
    if w_tok.is_empty() {
        return err("border: missing width in pixels");
    }
    let bw = parse_u32(w_tok, "border width")?;
    let color = match parts.get(1) {
        Some(s) if !s.trim().is_empty() => parse_color(s)?,
        _ => Rgba([0, 0, 0, 255]),
    };
    if bw == 0 {
        return Ok(img);
    }
    let (iw, ih) = (img.width(), img.height());
    let mut canvas = RgbaImage::from_pixel(iw + 2 * bw, ih + 2 * bw, color);
    imageops::overlay(&mut canvas, &img.to_rgba8(), bw as i64, bw as i64);
    Ok(DynamicImage::ImageRgba8(canvas))
}

fn op_trim(img: DynamicImage, spec: &str) -> io::Result<DynamicImage> {
    let parts: Vec<&str> = spec.split(':').collect();
    let color_tok = parts.first().copied().unwrap_or("").trim();
    let tol = (match parts.get(1) {
        Some(s) if !s.trim().is_empty() => parse_f64(s, "tolerance")?,
        _ => 10.0,
    }
    .clamp(0.0, 255.0)) as f32;

    let src = img.to_rgba8();
    let (w, h) = (src.width(), src.height());
    let bg = if color_tok.is_empty() || normalize(color_tok) == "auto" {
        src.get_pixel(0, 0).0
    } else {
        parse_color(color_tok)?.0
    };

    let diff = |p: [u8; 4]| -> f32 {
        (0..4)
            .map(|i| (p[i] as f32 - bg[i] as f32).abs())
            .fold(0.0f32, f32::max)
    };

    let (mut min_x, mut min_y, mut max_x, mut max_y) = (w, h, 0u32, 0u32);
    for y in 0..h {
        for x in 0..w {
            if diff(src.get_pixel(x, y).0) > tol {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if max_x < min_x || max_y < min_y {
        return Ok(img); // uniform image, nothing to trim
    }
    Ok(img.crop_imm(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([10, 20, 30, 255])))
    }

    // ── geometry ────────────────────────────────────────────────────────────

    #[test]
    fn splits_geometry_forms() {
        assert_eq!(split_geometry("400x300+50+50").unwrap(), ("400".into(), "300".into(), "+50".into(), "+50".into()));
        assert_eq!(split_geometry("50%x50%").unwrap(), ("50%".into(), "50%".into(), String::new(), String::new()));
        assert_eq!(split_geometry("200x200-10-10").unwrap(), ("200".into(), "200".into(), "-10".into(), "-10".into()));
        assert_eq!(split_geometry("100x80+10-20").unwrap(), ("100".into(), "80".into(), "+10".into(), "-20".into()));
        assert!(split_geometry("nonsense").is_err());
        assert!(split_geometry("x300").is_err());
    }

    #[test]
    fn resolves_lengths_and_offsets() {
        assert_eq!(resolve_len("50%", 700).unwrap(), 350);
        assert_eq!(resolve_len("120", 700).unwrap(), 120);
        assert!(resolve_len("0", 700).is_err());
        // ImageMagick style: negative offsets count from the far edge
        assert_eq!(resolve_off("-10", 700, 200).unwrap(), 490);
        assert_eq!(resolve_off("+10", 700, 200).unwrap(), 10);
        assert_eq!(resolve_off("+10%", 700, 200).unwrap(), 70);
        assert_eq!(resolve_off("", 700, 200).unwrap(), 0);
    }

    // ── colors & formats ────────────────────────────────────────────────────

    #[test]
    fn parses_colors() {
        assert_eq!(parse_color("#ffcc00").unwrap(), Rgba([255, 204, 0, 255]));
        assert_eq!(parse_color("#abc").unwrap(), Rgba([170, 187, 204, 255]));
        assert_eq!(parse_color("#11223344").unwrap(), Rgba([0x11, 0x22, 0x33, 0x44]));
        assert_eq!(parse_color("red").unwrap(), Rgba([255, 0, 0, 255]));
        assert_eq!(parse_color("transparent").unwrap(), Rgba([0, 0, 0, 0]));
        assert_eq!(parse_color("10,20,30").unwrap(), Rgba([10, 20, 30, 255]));
        assert_eq!(parse_color("10, 20, 30, 40").unwrap(), Rgba([10, 20, 30, 40]));
        assert!(parse_color("not-a-color").is_err());
        assert!(parse_color("300,0,0").is_err());
    }

    #[test]
    fn parses_formats() {
        assert_eq!(parse_format("JPG").unwrap(), ImageFormat::Jpeg);
        assert_eq!(parse_format("jpeg").unwrap(), ImageFormat::Jpeg);
        assert_eq!(parse_format("tif").unwrap(), ImageFormat::Tiff);
        assert!(parse_format("bmp").is_ok());
        assert!(parse_format("docx").is_err());
        assert_eq!(format_extension(ImageFormat::Jpeg).unwrap(), "jpg");
    }

    // ── names ───────────────────────────────────────────────────────────────

    #[test]
    fn resolves_operator_names() {
        assert_eq!(find_op("gray").unwrap().name, "grayscale");
        assert_eq!(find_op("Grey-Scale").unwrap().name, "grayscale");
        assert_eq!(find_op("MOSAIC").unwrap().name, "pixelate");
        assert!(find_op("crop").is_some());
        assert!(find_op("definitely-not-an-op").is_none());
        assert!(find_cmd("Crop").is_some());
    }

    #[test]
    fn builds_tile_paths() {
        assert_eq!(tile_path("part", 0, 1, "png"), "part-0-1.png");
        assert_eq!(tile_path("out/{r}_{c}", 1, 2, "png"), "out/1_2.png");
        // an image extension on the prefix wins over the default one
        assert_eq!(tile_path("tiles/x.jpg", 3, 4, "png"), "tiles/x-3-4.jpg");
    }

    #[test]
    fn computes_aspect_ratio() {
        assert_eq!(aspect_str(700, 540), "35:27");
        assert_eq!(aspect_str(1920, 1080), "16:9");
        assert!(aspect_str(10_000, 3).contains(":1"));
    }

    // ── operators ───────────────────────────────────────────────────────────

    #[test]
    fn crops_with_pixels_and_percentages() {
        let out = op_crop(flat(700, 540), "100x50+10+10").unwrap();
        assert_eq!((out.width(), out.height()), (100, 50));
        let out = op_crop(flat(700, 540), "50%x50%").unwrap();
        assert_eq!((out.width(), out.height()), (350, 270));
        // -10-10 counts from the right/bottom edge
        let out = op_crop(flat(700, 540), "200x200-10-10").unwrap();
        assert_eq!((out.width(), out.height()), (200, 200));
        // oversized crops are clamped instead of failing
        let out = op_crop(flat(700, 540), "9999x9999+0+0").unwrap();
        assert_eq!((out.width(), out.height()), (700, 540));
        assert!(op_crop(flat(700, 540), "100").is_err());
    }

    #[test]
    fn resizes_in_every_mode() {
        let out = op_resize(flat(700, 540), "50%").unwrap();
        assert_eq!((out.width(), out.height()), (350, 270));
        let out = op_resize(flat(700, 540), "320x_").unwrap();
        assert_eq!(out.width(), 320);
        assert_eq!(out.height(), 247); // aspect preserved
        let out = op_resize(flat(700, 540), "_270").unwrap();
        assert_eq!((out.width(), out.height()), (350, 270));
        let out = op_resize(flat(700, 540), "400x400:nearest:stretch").unwrap();
        assert_eq!((out.width(), out.height()), (400, 400));
        assert!(op_resize(flat(700, 540), "400x400:lanczos:tilt").is_err());
        assert!(op_resize(flat(700, 540), "").is_err());
    }

    #[test]
    fn thumbnails_fit_inside_the_box() {
        let out = op_thumbnail(flat(700, 540), "200").unwrap();
        assert_eq!((out.width(), out.height()), (200, 154));
        let out = op_thumbnail(flat(700, 540), "1000x1000").unwrap();
        assert_eq!((out.width(), out.height()), (700, 540)); // never upscales
    }

    #[test]
    fn rotates_by_right_and_free_angles() {
        let out = op_rotate(flat(700, 540), "90").unwrap();
        assert_eq!((out.width(), out.height()), (540, 700));
        let out = op_rotate(flat(700, 540), "180").unwrap();
        assert_eq!((out.width(), out.height()), (700, 540));
        let out = op_rotate(flat(700, 540), "45:#ffffff").unwrap();
        let diag = ((700.0 + 540.0) * std::f64::consts::FRAC_1_SQRT_2).round() as u32;
        assert_eq!((out.width(), out.height()), (diag, diag));
        assert!(op_rotate(flat(10, 10), "").is_err());
    }

    #[test]
    fn flips_and_color_ops() {
        assert_eq!(
            op_flip(flat(3, 5), "v").unwrap().height(),
            5
        );
        let out = op_grayscale(flat(10, 10), "").unwrap();
        let p = out.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(p[0], p[1]);
        assert_eq!(p[1], p[2]);
        assert_eq!(op_invert(flat(10, 10), "").unwrap().to_rgba8().get_pixel(0, 0).0[0], 245);
        assert!(op_posterize(flat(10, 10), "1").is_err());
        assert!(op_brightness(flat(10, 10), "").is_err());
    }

    #[test]
    fn alpha_ops_stay_transparent() {
        let faded = op_opacity(flat(10, 10), "50").unwrap();
        assert_eq!(faded.to_rgba8().get_pixel(0, 0).0[3], 128);
        let out = op_pad(flat(10, 10), "20x20:transparent:center").unwrap();
        assert_eq!((out.width(), out.height()), (20, 20));
        assert_eq!(out.to_rgba8().get_pixel(0, 0).0[3], 0);
        // gravity pushes the content to a corner
        let out = op_pad(flat(10, 10), "40x40:black:nw").unwrap();
        assert_eq!(out.to_rgba8().get_pixel(5, 5).0, [10, 20, 30, 255]);
        assert_eq!(out.to_rgba8().get_pixel(39, 39).0, [0, 0, 0, 255]);
    }

    #[test]
    fn borders_and_trimming() {
        let out = op_border(flat(10, 10), "3:white").unwrap();
        assert_eq!((out.width(), out.height()), (16, 16));
        assert_eq!(out.to_rgba8().get_pixel(0, 0).0, [255, 255, 255, 255]);
        assert_eq!(out.to_rgba8().get_pixel(8, 8).0, [10, 20, 30, 255]);

        // uniform image: nothing to trim
        let out = op_trim(flat(40, 40), "auto:10").unwrap();
        assert_eq!((out.width(), out.height()), (40, 40));
        // image with a uniform 5px frame around a different core
        let mut buf = RgbaImage::from_pixel(40, 40, Rgba([255, 255, 255, 255]));
        for y in 5..35 {
            for x in 5..35 {
                buf.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let out = op_trim(DynamicImage::ImageRgba8(buf), "auto:10").unwrap();
        assert_eq!((out.width(), out.height()), (30, 30));
    }

    #[test]
    fn filter_ops_run() {
        assert_eq!((op_blur(flat(20, 20), "1.5").unwrap().width()), 20);
        assert_eq!((op_blur(flat(20, 20), "1.5:fast").unwrap().height()), 20);
        assert_eq!(op_sharpen(flat(20, 20), "2:3").unwrap().width(), 20);
        assert_eq!(op_pixelate(flat(20, 20), "8").unwrap().width(), 20);
        assert_eq!(op_grain(flat(20, 20), "40").unwrap().height(), 20);
        assert_eq!(op_edge(flat(20, 20), "").unwrap().width(), 20);
        assert_eq!(op_emboss(flat(20, 20), "").unwrap().height(), 20);
        assert!(op_blur(flat(20, 20), "0").is_err());
        assert!(op_sharpen(flat(20, 20), "-1").is_err());
        // deterministic grain: same input, same output
        let a = op_grain(flat(8, 8), "50").unwrap().to_rgba8();
        let b = op_grain(flat(8, 8), "50").unwrap().to_rgba8();
        assert_eq!(a, b);
    }

    #[test]
    fn roundtrips_through_a_real_file() {
        let dir = std::env::temp_dir().join("ll_image_ops_test");
        fs::create_dir_all(&dir).unwrap();
        let input = dir.join("in.png");
        let output = dir.join("nested/out.jpg");
        flat(32, 16).save(&input).unwrap();

        let img = read_image(&input.to_string_lossy()).unwrap();
        let cropped = op_crop(img, "16x8+4+4").unwrap();
        let save = Save { quality: Some(85), format: None, background: Rgba([255, 255, 255, 255]) };
        save.write(&cropped, &output.to_string_lossy()).unwrap();

        let back = read_image(&output.to_string_lossy()).unwrap();
        assert_eq!((back.width(), back.height()), (16, 8));
        assert_eq!(ImageFormat::from_path(&output).unwrap(), ImageFormat::Jpeg);
        let _ = fs::remove_dir_all(&dir);
    }
}

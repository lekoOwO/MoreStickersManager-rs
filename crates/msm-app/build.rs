use std::{env, fs, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
    let workspace_dir = Path::new(&manifest_dir)
        .parent()
        .and_then(Path::parent)
        .expect("msm-app must live under crates/msm-app");
    let web_dist = workspace_dir.join("apps/web/dist");
    let placeholder = Path::new(&manifest_dir).join("web-dist-placeholder");
    let source = if web_dist.join("index.html").is_file() {
        web_dist
    } else {
        placeholder
    };
    let target = Path::new(&out_dir).join("web-dist-embed");

    if target.exists() {
        fs::remove_dir_all(&target).expect("failed to clear embedded web target directory");
    }
    copy_dir_all(&source, &target).expect("failed to prepare embedded web assets");

    println!("cargo:rerun-if-changed={}", source.display());
    println!(
        "cargo:rerun-if-changed={}",
        source.join("index.html").display()
    );
}

fn copy_dir_all(source: &Path, target: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target_path = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target_path)?;
        } else {
            fs::copy(entry.path(), target_path)?;
        }
    }
    Ok(())
}

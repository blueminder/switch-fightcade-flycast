use dircpy::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, prelude::*, Write};
use std::path::Path;

fn get_version_by_hash(bin_path: &str) -> String {
    let version_hashes = HashMap::from([
        (
            "c1ebe8d91f0e187a8cd2006837b2a29b1a03d612fe28aecccf551298e4b5fcea",
            "dojo-0.5.8",
        ),
        (
            "15ed2d49e037fd5f2596bc3d3fd39b22531c2d68c368f74338071bbc86e8d21d",
            "dojo-6.12",
        ),
        (
            "56f4a275d1b0141e91e473d94ef44908b72f98a123f214faf2d8bc875f623b4e",
            "dojo-6.11",
        ),
        (
            "70db9e3eb56c572623c2be6534e38134b9f7f6c1bd0bc83432854c6c00af21ae",
            "dojo-6.10",
        ),
        (
            "f7894e69921e8ff44122da02916f3b9c76dd7cabfd0821e54be2b293444a129d",
            "dojo-6.9",
        ),
        (
            "8fad59958a25790ecbe18f33e070060c84276961ff5736cee5ed0b32f8d35de8",
            "dojo-6.6",
        ),
    ]);

    let bytes = fs::read(bin_path).unwrap();
    let hash = sha256::digest(&bytes);
    match version_hashes.get(&hash as &str) {
        Some(val) => val.to_string(),
        None => "".to_string(),
    }
}

fn get_version_type(flycast_tag: &str) -> String {
    if flycast_tag == "dojo-0.5.8" {
        return String::from("Bundled");
    } else {
        return String::from("Prerelease");
    }
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    write!(stdout, "Press Enter to continue...").unwrap();
    stdout.flush().unwrap();

    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() -> std::io::Result<()> {
    let dirs = vec!["mappings", "ROMs"];
    for dir in dirs.iter() {
        let dir_path = format!("flycast\\{}", dir);
        let prev_dir_path = format!("flycast_previous\\{}", dir);
        if Path::new(&dir_path).exists() {
            println!("Copying {} folder to active install", dir);
            copy_dir(dir_path, prev_dir_path)?;
        }
    }

    let bios_files = vec!["awbios.zip", "naomi.zip", "naomi2.zip"];
    for bios_file in bios_files.iter() {
        let bios_path = format!("flycast\\data\\{}", bios_file);
        let prev_roms_path = format!("flycast_previous\\ROMs\\{}", bios_file);
        if Path::new(&bios_path).exists() {
            if !Path::new(&prev_roms_path).exists() {
                println!("Copying {} to the ROMs folder", bios_file);
                fs::copy(&bios_path, &prev_roms_path)?;
            }
        }
    }

    println!("Copying Netplay Savestates to active install");
    CopyBuilder::new("flycast\\data", "flycast_previous\\data")
        .with_include_filter(".state.net")
        .run()
        .unwrap();

    let old_version;
    let old_version_path = "flycast\\VERSION.txt";
    if Path::new(&old_version_path).exists() {
        old_version = fs::read_to_string(&old_version_path)?;
    } else {
        let old_flycast_path = "flycast\\flycast.exe";
        old_version = get_version_by_hash(old_flycast_path);
        if !old_version.is_empty() {
            let mut version_txt = File::create(old_version_path)?;
            write!(version_txt, "{}", old_version)?;
        }
    }

    let new_version;
    let new_version_path = "flycast_previous\\VERSION.txt";
    if Path::new(&new_version_path).exists() {
        new_version = fs::read_to_string(&new_version_path)?;
    } else {
        let new_flycast_path = "flycast_previous\\flycast.exe";
        new_version = get_version_by_hash(new_flycast_path);
        if !new_version.is_empty() {
            let mut version_txt = File::create(new_version_path)?;
            write!(version_txt, "{}", new_version)?;
        }
    }

    // replace flycast dojo version name in fightcade title bar
    let inject_path = "..\\fc2-electron\\resources\\app\\inject\\inject.js";
    if Path::new(&inject_path).exists() {
        let mut inject_contents = fs::read_to_string(&inject_path)?;

        let title_old_version_string = format!("(Flycast Version: {})", old_version);

        let title_old_version_type_string = format!(
            "(Flycast Version: {}, {})",
            old_version,
            get_version_type(&old_version)
        );
        let title_new_version_type_string = format!(
            "(Flycast Version: {}, {})",
            new_version,
            get_version_type(&new_version)
        );

        if inject_contents.contains(&title_old_version_string) {
            inject_contents =
                inject_contents.replace(&title_old_version_string, &title_old_version_type_string)
        }

        let inject_replaced = inject_contents.replace(
            &title_old_version_type_string,
            &title_new_version_type_string,
        );
        let tmp_inject_path = "..\\fc2-electron\\resources\\app\\inject\\inject.js.tmp";
        let mut tmp_inject = File::create(tmp_inject_path)?;
        write!(tmp_inject, "{}", inject_replaced)?;
        fs::remove_file(&inject_path)?;
        fs::rename(&tmp_inject_path, &inject_path)?;
    } else {
        // create new inject file with fightcade title modification
        let fc_title_inject_contents = format!(
            "const appendFlycastTitle = function (fcWindow) {{\n\
              const fcDoc = fcWindow.document\n\
              fcDoc.title += \" (Flycast Version: {}, {})\"\n\
            }}\n\n\
            appendFlycastTitle(window)\n",
            new_version,
            get_version_type(&new_version)
        );
        let mut inject = File::create(inject_path)?;
        write!(inject, "{}", fc_title_inject_contents)?;
    }

    if new_version.is_empty() {
        println!("Switching Flycast Dojo Version...");
    } else {
        let short_new_version = new_version.replace("dojo-", "");
        println!("Switching to Flycast Dojo Version {}...", short_new_version);
    }
    fs::rename("flycast", "flycast_tmp")?;
    fs::rename("flycast_previous", "flycast")?;
    fs::rename("flycast_tmp", "flycast_previous")?;
    println!("Success!");

    pause();

    Ok(())
}

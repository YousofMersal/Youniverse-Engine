use std::{
    ffi::OsStr,
    fs::{self, create_dir},
    io::Result,
    path::PathBuf,
    process::{Command, Output},
};

fn main() {
    // if should_recompile_shaders() {
    //     recompile_shaders();
    // }
}

fn should_recompile_shaders() -> bool {
    let dir = PathBuf::from("./shaders/spv");

    match dir.read_dir() {
        Ok(_items) => {
            if dir.read_dir().unwrap().count() > 0 {
                dir.read_dir()
                    .unwrap()
                    .map(Result::unwrap)
                    .filter(|entry| entry.path().is_file())
                    .for_each(|entry| {
                        fs::remove_file(entry.path()).unwrap();
                    });
            };

            true
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                create_dir("./shaders/spv").unwrap();
                true
            }
            _ => panic!("unknown error while raeding dir"),
        },
    }

    // .unwrap().next().is_none()
}

fn recompile_shaders() {
    let dir_path = dbg!(PathBuf::from("./shaders/spv").canonicalize().unwrap());

    fs::read_dir("./shaders")
        .unwrap()
        .map(Result::unwrap)
        .filter(|dir| dir.file_type().unwrap().is_file())
        .filter(|dir| dir.path().extension() != Some(OsStr::new("spv")))
        .for_each(|dir| {
            let path = dir.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            let output_name = format!("./spv/{}.spv", &name);
            println!("Found file {:?}", path.as_os_str());
            let res = Command::new("glslc")
                .current_dir(dir_path.parent().unwrap())
                .arg(name)
                .arg("-o")
                .arg(output_name)
                .output();

            handle_compile_result(res);
        });
}

fn handle_compile_result(res: Result<Output>) {
    match res {
        Ok(output) => {
            if output.status.success() {
                println!("Shader compilation succedeed.");
            } else {
                eprintln!("Shader compilation failed. Status: {}", output.status);
                eprint!(
                    "stdout: {}",
                    String::from_utf8(output.stdout).expect("Failed to print program stdout")
                );
                eprint!(
                    "stderr: {}",
                    String::from_utf8(output.stderr).expect("Failed to print program stderr")
                );
                panic!("Shader compilation failed. Status: {}", output.status);
            }
        }
        Err(err) => {
            panic!("Could not compile shaders. Cause: {}", err)
        }
    }
}

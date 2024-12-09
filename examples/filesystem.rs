extern crate sdl3;

use sdl3::filesystem::*;

pub fn main() -> Result<(), String> {
    sdl3::init().ok();
    let base_path = get_base_path().unwrap();
    println!("Base path: {base_path:?}");

    let path_info = get_path_info(base_path).unwrap();
    println!("Base path info: {path_info:?}");


    enumerate_directory(base_path, |directory, file| {
        println!("Enumerate {directory:?}: {file:?}");
        return EnumerationResult::CONTINUE;
    }).ok();

    if let Ok(results) = glob_directory(base_path, Some("filesystem*"), GlobFlags::NONE) {
        for result in &results {
            println!("Glob: {result:?}");
        }
    }

    let user_folder = get_user_folder(Folder::DOCUMENTS).unwrap();
    println!("Documents folder: {user_folder:?}");

    let test_path = base_path.join("testpath");
    let test_path2 = base_path.join("testpath2");
    match get_path_info(&test_path) {
        Ok(info) => println!("Test path info: {info:?}"),
        Err(e) => println!("Test path error: {e:?}")
    }
    create_directory(&test_path).ok();
    match get_path_info(&test_path) {
        Ok(info) => println!("Test path info: {info:?}"),
        Err(e) => println!("Test path error: {e:?}")
    }

    match rename_path(&test_path, &test_path2) {
        Ok(()) => println!("Renamed {test_path:?} to {test_path2:?}"),
        Err(e) => eprintln!("Failed to rename: {e:?}")
    }

    match remove_path(&test_path2) {
        Ok(()) => println!("Removed {test_path2:?}"),
        Err(e) => eprintln!("Failed to remove: {e:?}")
    }

    match get_pref_path("sdl-rs", "filesystem") {
        Ok(path) => {
            println!("Got pref path for org 'sdl-rs' app 'filesystem' as {path:?}");
            match remove_path(&path) {
                Ok(()) => println!("Removed {path:?}"),
                Err(e) => eprintln!("Failed to remove: {e:?}")
            }
        }, 
        Err(error) => {
            eprintln!("Failed to get pref path for org 'sdl-rs' app 'filesystem': {error:?}")
        }
    }


    Ok(())
}

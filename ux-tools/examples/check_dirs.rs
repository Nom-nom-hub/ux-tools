use directories::ProjectDirs;

fn main() {
    if let Some(proj_dirs) = ProjectDirs::from("com", "ux", "ux") {
        println!("cache_dir: {:?}", proj_dirs.cache_dir());
        println!("data_dir: {:?}", proj_dirs.data_dir());
        println!("config_dir: {:?}", proj_dirs.config_dir());
    } else {
        println!("No project dirs found");
    }
}
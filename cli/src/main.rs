use tagfs_core;
fn main() {
    let mut repo = tagfs_core::Repo::open().unwrap();
    println!("Tag ID: {}", repo.new_tag("example_tag").unwrap());
    println!(
        "Tag ID: {:?}",
        repo.update_tag("example_tag1", "example_tag").unwrap()
    );
    println!(
        "Tag ID: {:?}",
        repo.update_tag("example_tag5", "example_tag").unwrap()
    );
}

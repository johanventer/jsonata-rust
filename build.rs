fn main() {
    println!("cargo:rerun-if-changed=tests/testsuite/groups/*/*.json");
    println!("cargo:rerun-if-changed=tests/testsuite/groups");
}

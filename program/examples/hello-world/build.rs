fn main() {
    println!("cargo::rustc-check-cfg=cfg(target_os, values(\"solana\"))");
    println!("cargo::rustc-check-cfg=cfg(target_vendor, values(\"solana\"))");
}
fn main() {
	println!("cargo:rustc-link-lib=ffi");
	println!("cargo:rustc-link-lib=LLVMCore");
}

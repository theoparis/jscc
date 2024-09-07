fn main() {
	println!("cargo:rustc-link-search=/usr/lib/llvm-20/lib");
	println!("cargo:rustc-link-lib=ffi");
}

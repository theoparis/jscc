//! Input of the LLVM bitcode format.

use super::prelude::*;

extern "C" {
	/// Build a module from the bitcode in the specified memory buffer.
	///
	/// Returns the created module in OutModule, returns 0 on success.
	pub fn LLVMParseBitcode2(
		MemBuf: LLVMMemoryBufferRef,
		OutModule: *mut LLVMModuleRef,
	) -> LLVMBool;

	pub fn LLVMParseBitcodeInContext2(
		ContextRef: LLVMContextRef,
		MemBuf: LLVMMemoryBufferRef,
		OutModule: *mut LLVMModuleRef,
	) -> LLVMBool;

	/// Read a module from the specified path, returning a module provider
	/// performing lazy deserialization.
	///
	/// Returns 0 on success.
	pub fn LLVMGetBitcodeModuleInContext2(
		ContextRef: LLVMContextRef,
		MemBuf: LLVMMemoryBufferRef,
		OutM: *mut LLVMModuleRef,
	) -> LLVMBool;

	/// Read a module from the specified path.
	///
	/// Outputs a module provider which performs lazy deserialization.
	/// Returns 0 on success.
	pub fn LLVMGetBitcodeModule2(
		MemBuf: LLVMMemoryBufferRef,
		OutM: *mut LLVMModuleRef,
	) -> LLVMBool;
}

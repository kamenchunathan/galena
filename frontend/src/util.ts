export function unpackSlice(value: number) {
  const valueBigInt = BigInt(value);
  const lowBits = valueBigInt & 0xffffffffn;
  const highBits = (valueBigInt / 0x100000000n) | 0n;

  return {
    ptr: Number(lowBits), // Lower 32 bits as pointer
    len: Number(highBits), // Upper 32 bits as length
  };
}

export function packSlice(ptr: number, len: number) {
  const ptrBigInt = BigInt(ptr);
  const lenBigInt = BigInt(len);

  // Ensure values fit in 32 bits
  if (ptrBigInt > 0xffffffffn || lenBigInt > 0xffffffffn) {
    throw new Error("Pointer or length exceeds 32-bit maximum");
  }

  // Pack: lower 32 bits = pointer, upper 32 bits = length
  return (lenBigInt << 32n) | ptrBigInt;
}

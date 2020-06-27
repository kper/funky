#define WASM_EXPORT __attribute__((visibility("default")))

int sum(int e[], int length) {
  int sum = 0;
  for (int i = 0; i < length; i++) {
    sum += e[i];
  }
  return sum;
}

WASM_EXPORT
int demo_sum(int length) {
  int e[length];
  for (int i = 0; i < length; i++) {
    e[i] = i;
  }
  return sum(e, length);
}

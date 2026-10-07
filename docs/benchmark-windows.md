# Windows benchmark

C#: `sortini v4.0.2605.1521+d2259ec3a087e788bce9905063933ab9522f93e5`

Rust: `4.0.0`

C# SHA-256: `0752682ea7e13b49c798df03e9aa7961fbbd5f33f5785e7a2c1fb87f519fd714`

Rust SHA-256: `e2b9e2a7e3d31e4fb0e93b9b09b4cb19634103128018b0b28a6714589c6ee48e`

Architecture: x86_64; logical CPUs available: 64.

7 warmed samples per executable/workload, alternating executable order.
Sort output is validated byte-for-byte once before timing; timed stdout goes to NUL.
Both apps use explicit `--expand --stdout` because this C# build expands by default.

| Workload | Input KiB | App | Median ms | MiB/s | Sampled peak working set MiB |
| --- | ---: | --- | ---: | ---: | ---: |
| startup (-v) | 0.00 | C# | 98.161 | N/A | 57.54 |
| startup (-v) | 0.00 | Rust | 16.183 | N/A | 9.98 |
| small | 0.59 | C# | 102.856 | 0.01 | 58.06 |
| small | 0.59 | Rust | 17.262 | 0.03 | 10.07 |
| medium | 78.12 | C# | 109.289 | 0.70 | 63.89 |
| medium | 78.12 | Rust | 18.098 | 4.22 | 10.50 |
| large | 4093.46 | C# | 414.484 | 9.64 | 121.66 |
| large | 4093.46 | Rust | 73.319 | 54.52 | 40.86 |

Wall time includes process startup and file reading; MiB/s is end-to-end, not an isolated parser benchmark. Memory is process working-set peak observed through GetProcessMemoryInfo at 1 ms intervals; short processes can exit between polls, so the reported peak can be an underestimate. Child-process memory is excluded. File writes and backup/fsync costs are not measured. Each sample is bounded to 30 seconds. These are synthetic, warmed local results, not universal speed claims.

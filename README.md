# sym-filters
Audio filters for rust.

This crate requires `RUSTC_BOOTSTRAP=1` or `RUSTC_BOOTSTRAP=sym_filters` for fast math. If you wish to use this set these environment variables before compiling.

## Examples
### Low-pass filter
```rs
use sym_filters::{Filter, Biquad};

// Create Filter object. This should be stored and reused for future blocks 
// as it has some memory.
let mut filter = Biquad::new();

// Set internal parameters to low-pass.
filter.lpf(cuttoff, q, sample_rate);

filter.process(&[audio_buffer]);
```

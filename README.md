# sym-filters
Audio filters for rust.

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

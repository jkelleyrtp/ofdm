//! This program implements lab3 OFDM using the Rust programming language.

/// Prepare a data stream by encoding it into blocks, adding a preamble, and spacing it out for OFDM
#[optargs::optfn]
pub fn transmit(
    data: Vec<f32>,
    block_size: Option<u32>,
    prefix_length: Option<u32>,
    training_blocks: Option<u32>,
    estimation_blocks: Option<u32>,
    preamble_blocks: Option<u32>,
    guard_bands: Option<bool>,
) -> Vec<f32> {
    let block_size = block_size.unwrap_or_else(|| 0);
    let prefix_length = prefix_length.unwrap_or_else(|| 0);
    let training_blocks = training_blocks.unwrap_or_else(|| 0);
    let estimation_blocks = estimation_blocks.unwrap_or_else(|| 0);
    let preamble_blocks = preamble_blocks.unwrap_or_else(|| 0);
    let guard_bands = guard_bands.unwrap_or_else(|| false);

    fn prefix_block(stream: Vec<f32>, length: u32) {
        // % Rehsape the data into the appropraite block sizes
        // block_data = zeros(...
        //     size(blockstream, 1), ... % Column length (with prefix if prefixenabled)
        //     size(blockstream, 2) + prefix_length ... % Number of rows
        //     );

        // % Encode the data with a prefix
        // kstop = size(blockstream, 1);
        // for k = 1:kstop
        //     % Simply concat the input and the end of the input to create a cyclical data array
        //     %
        //     % Approach illustrated in:
        //     %    https://dspillustrations.com/pages/posts/misc/the-cyclic-prefix-cp-in-ofdm.html
        //     %    symb1 = hstack([ofdm1[-NCP:], ofdm1])
        //     iffted = ifft(blockstream(k, :));

        //     % Apply the prefix
        //     % Grab out the last N samples where N = prefix_length
        //     pref = iffted(length(iffted) - prefix_length + 1:end);

        //     %         Assign the data
        //     block_data(k, :) = transpose([pref iffted]);
        // end
    }
    fn create_preamble(stream: Vec<f32>, length: u32) {
        // %     preamble = (1:preamble_size) ./ preamble_size;
        // preamble = (-1).^(1:preamble_size);
    }

    // // Generate locking features
    // // Generate signals to perform the channel estimation
    // // Generate a preamble for frequency correction
    // // Generate guardbands & dc offset

    // // Take in a datastream of 1s and 0s and convert it to 1s and -1s
    // //             bpsk_stream = bin_stream;
    // bpsk_stream = (bin_stream .* 2) - 1;

    // // Reshape the stream into an abitrary # of columns with a fixed size
    // // The number of blocks automatically expands to fit the input data
    // block_data = reshape(...
    //     bpsk_stream, [], self.block_size);

    // training_signals = repmat(...
    //     Utils.training_signals(self.block_size), self.training_blocks, 1);

    // // Preamble the data for frequency offset correction
    // // Make the preamble as wide as the current data is
    // // This scales with the guard band additions
    // preamble = repmat(...
    //     create_preamble(...
    //     self.block_size ...
    //     ), self.preamble_blocks, 1);

    // block_data = [preamble; block_data];

    // block_data = prefix_block(...
    //     [training_signals; block_data], self.prefix_length);

    // // Add guardbands, DC Offset, etc
    // if self.guard_bands
    //     // Instead of shrinking our channels down, we just make them bigger
    //     // That way, the packet size stays the same but the channel utilization increases

    //     // Currently ripped out to fix frequency shifting
    // end

    // // // Preamble the data for frequency offset correction
    // // // Make the preamble as wide as the current data is
    // // // This scales with the guard band additions
    // // preamble = repmat(...
    // //     create_preamble(...
    // //     self.block_size + self.prefix_length ...
    // //     ), self.preamble_blocks, 1);

    // // block_data = [preamble; block_data];

    // // Flatten the block structure down into a single stream
    // //             transpose is very important, for whatever erason
    // samples = reshape(transpose(block_data), 1, []);

    // // Add features to lock onto the signal easier
    // locking = Utils.locking_features();
    // samples = [locking samples];

    todo!()
}

#[optargs::optfn]
pub fn channel(transmission: Vec<f32>, snr: Option<f32>) -> Vec<f32> {
    todo!()
}

#[optargs::optfn]
pub fn receive(
    samples: Vec<f32>,
    block_size: Option<u32>,
    prefix_length: Option<u32>,
    training_blocks: Option<u32>,
    estimation_blocks: Option<u32>,
    preamble_blocks: Option<u32>,
    guard_bands: Option<bool>,
) -> Vec<f32> {
    todo!()
}

pub mod utils {
    pub fn create_transmission(packet_size: u32) -> Vec<f32> {
        todo!()
    }

    pub fn normalize_received_samples(received: u32) {}

    pub fn estimate_error(data: u32, normalized: u32) {}
}

/// Important functions for doing signal processing not normally included in Rust.
/// These are included manually so they can be swapped out with the realtime implementation.
pub mod signals {
    fn xcorr() {}
    fn convolve() {}
    fn fft() {}
    fn ifft() {}
}

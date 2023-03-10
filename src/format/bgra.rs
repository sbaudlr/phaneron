use crate::io::{InterlaceMode, Packer, Unpacker};

const PIXELS_PER_WORK_ITEM: usize = 64;

fn get_pitch(width: usize) -> usize {
    width
}

fn get_pitch_bytes(width: usize) -> usize {
    get_pitch(width) * 4
}

pub struct BGRA8Reader {
    width: usize,
    height: usize,
    num_bytes: Vec<usize>,
    work_items_per_group: usize,
    global_work_items: usize,
}

impl BGRA8Reader {
    pub fn new(width: usize, height: usize) -> Self {
        let num_bytes = vec![get_pitch_bytes(width) * height];
        let work_items_per_group = get_pitch(width) / PIXELS_PER_WORK_ITEM;
        let global_work_items = work_items_per_group * height;

        Self {
            width,
            height,
            num_bytes,
            work_items_per_group,
            global_work_items,
        }
    }
}

impl Packer for BGRA8Reader {
    fn get_name(&self) -> &str {
        "BGRA8 Reader"
    }

    fn get_kernel(&self) -> &str {
        include_str!("../../shaders/video_process/load/bgra8.cl")
    }

    fn get_width(&self) -> usize {
        self.width
    }

    fn get_height(&self) -> usize {
        self.height
    }

    fn get_num_bits(&self) -> usize {
        8
    }

    fn get_luma_black(&self) -> f32 {
        16.0
    }

    fn get_luma_white(&self) -> f32 {
        235.0
    }

    fn get_chroma_range(&self) -> f32 {
        224.0
    }

    fn get_num_bytes(&self) -> Vec<usize> {
        self.num_bytes.clone()
    }

    fn get_num_bytes_rgba(&self) -> usize {
        self.width * self.height * 4 * 4
    }

    fn get_is_rgb(&self) -> bool {
        true
    }

    fn get_total_bytes(&self) -> usize {
        self.num_bytes.iter().sum()
    }

    fn get_work_items_per_group(&self) -> usize {
        self.work_items_per_group
    }

    fn get_global_work_items(&self) -> usize {
        self.global_work_items
    }

    fn get_kernel_params(
        &self,
        kernel: &mut opencl3::kernel::ExecuteKernel,
        inputs: &[&opencl3::memory::Buffer<opencl3::types::cl_uchar>],
        output: &mut opencl3::memory::Buffer<opencl3::types::cl_uchar>,
    ) {
        if inputs.len() != 1 {
            panic!(
                "Reader for {} requires exactly 1 input, received {}",
                self.get_name(),
                inputs.len()
            );
        }

        let width = self.width as u32;

        unsafe { kernel.set_arg(inputs[0]).set_arg(output).set_arg(&width) };
    }
}

pub struct BGRA8Writer {
    width: usize,
    height: usize,
    interlace: InterlaceMode,
    num_bytes: Vec<usize>,
    work_items_per_group: usize,
    global_work_items: usize,
}

impl BGRA8Writer {
    pub fn new(width: usize, height: usize, interlace: InterlaceMode) -> Self {
        let num_bytes = vec![get_pitch_bytes(width) * height];
        let work_items_per_group = get_pitch(width) / PIXELS_PER_WORK_ITEM;
        let global_work_items = (work_items_per_group * height)
            / (match interlace {
                InterlaceMode::Progressive => 1,
                _ => 2,
            });

        Self {
            width,
            height,
            interlace,
            num_bytes,
            work_items_per_group,
            global_work_items,
        }
    }
}

impl Unpacker for BGRA8Writer {
    fn get_name(&self) -> &str {
        "BGRA8 Writer"
    }

    fn get_kernel(&self) -> &str {
        include_str!("../../shaders/video_process/consume/bgra8.cl")
    }

    fn get_width(&self) -> usize {
        self.width
    }

    fn get_height(&self) -> usize {
        self.height
    }

    fn get_num_bits(&self) -> usize {
        8
    }

    fn get_luma_black(&self) -> f32 {
        16.0
    }

    fn get_luma_white(&self) -> f32 {
        235.0
    }

    fn get_chroma_range(&self) -> f32 {
        224.0
    }

    fn get_num_bytes(&self) -> Vec<usize> {
        self.num_bytes.clone()
    }

    fn get_num_bytes_rgba(&self) -> usize {
        self.width * self.height * 4 * 4
    }

    fn get_is_rgb(&self) -> bool {
        true
    }

    fn get_total_bytes(&self) -> usize {
        self.num_bytes.iter().sum()
    }

    fn get_work_items_per_group(&self) -> usize {
        self.work_items_per_group
    }

    fn get_global_work_items(&self) -> usize {
        self.global_work_items
    }

    fn get_kernel_params(
        &self,
        kernel: &mut opencl3::kernel::ExecuteKernel,
        input: &opencl3::memory::Buffer<opencl3::types::cl_uchar>,
        outputs: &mut Vec<opencl3::memory::Buffer<opencl3::types::cl_uchar>>,
    ) {
        if outputs.len() != 1 {
            panic!(
                "Writer for {} requires exactly 1 output, received {}",
                self.get_name(),
                outputs.len()
            );
        }

        let width = self.width as u32;

        unsafe {
            kernel
                .set_arg(input)
                .set_arg(&outputs[0])
                .set_arg(&width)
                .set_arg(&self.interlace.as_kernel_param())
        };
    }
}
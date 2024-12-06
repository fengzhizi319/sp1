use num::{BigUint, Integer, One};

use sp1_primitives::consts::{bytes_to_words_le, words_to_bytes_le_vec};

use crate::{
    events::{PrecompileEvent, U256xU2048MulEvent},
    syscalls::{Syscall, SyscallCode, SyscallContext},
    Register::{X12, X13},
};

const U256_NUM_WORDS: usize = 8;
const U2048_NUM_WORDS: usize = 64;
const U256_NUM_BYTES: usize = U256_NUM_WORDS * 4;
const U2048_NUM_BYTES: usize = U2048_NUM_WORDS * 4;

pub(crate) struct U256xU2048MulSyscall;

impl Syscall for U256xU2048MulSyscall {
    /// 执行 U256 乘以 U2048 的系统调用。
    ///
    /// # 参数
    /// - `rt`: 系统调用上下文。
    /// - `syscall_code`: 系统调用代码。
    /// - `arg1`: 第一个参数，表示 U256 值的指针。
    /// - `arg2`: 第二个参数，表示 U2048 值的指针。
    ///
    /// # 返回值
    /// 返回一个可选的 u32 值。
    fn execute(
        &self,
        rt: &mut SyscallContext,
        syscall_code: SyscallCode,
        arg1: u32,
        arg2: u32,
    ) -> Option<u32> {
        let clk = rt.clk;

        // 获取 U256 和 U2048 值的指针
        let a_ptr = arg1;
        let b_ptr = arg2;

        // 获取低位和高位结果的指针
        let (lo_ptr_memory, lo_ptr) = rt.mr(X12 as u32);
        let (hi_ptr_memory, hi_ptr) = rt.mr(X13 as u32);

        // 读取 U256 和 U2048 值
        let (a_memory_records, a) = rt.mr_slice(a_ptr, U256_NUM_WORDS);
        let (b_memory_records, b) = rt.mr_slice(b_ptr, U2048_NUM_WORDS);
        let uint256_a = BigUint::from_bytes_le(&words_to_bytes_le_vec(&a));
        let uint2048_b = BigUint::from_bytes_le(&words_to_bytes_le_vec(&b));

        // 计算乘积
        let result = uint256_a * uint2048_b;

        // 计算 2^2048
        let two_to_2048 = BigUint::one() << 2048;

        // 将结果分成高位和低位
        let (hi, lo) = result.div_rem(&two_to_2048);

        // 将低位结果转换为字节并填充到 U2048_NUM_BYTES 长度
        let mut lo_bytes = lo.to_bytes_le();
        lo_bytes.resize(U2048_NUM_BYTES, 0u8);
        let lo_words = bytes_to_words_le::<U2048_NUM_WORDS>(&lo_bytes);

        // 将高位结果转换为字节并填充到 U256_NUM_BYTES 长度
        let mut hi_bytes = hi.to_bytes_le();
        hi_bytes.resize(U256_NUM_BYTES, 0u8);
        let hi_words = bytes_to_words_le::<U256_NUM_WORDS>(&hi_bytes);

        // 增加时钟周期以便写入不与读取在同一个周期
        rt.clk += 1;

        // 将低位和高位结果写入内存
        let lo_memory_records = rt.mw_slice(lo_ptr, &lo_words);
        let hi_memory_records = rt.mw_slice(hi_ptr, &hi_words);
        let lookup_id = rt.syscall_lookup_id;
        let shard = rt.current_shard();
        let event = PrecompileEvent::U256xU2048Mul(U256xU2048MulEvent {
            lookup_id,
            shard,
            clk,
            a_ptr,
            a,
            b_ptr,
            b,
            lo_ptr,
            lo: lo_words.to_vec(),
            hi_ptr,
            hi: hi_words.to_vec(),
            lo_ptr_memory,
            hi_ptr_memory,
            a_memory_records,
            b_memory_records,
            lo_memory_records,
            hi_memory_records,
            local_mem_access: rt.postprocess(),
        });

        // 创建系统调用事件并添加到预编译事件中
        let sycall_event =
            rt.rt.syscall_event(clk, syscall_code.syscall_id(), arg1, arg2, lookup_id);
        rt.add_precompile_event(syscall_code, sycall_event, event);

        None
    }

    fn num_extra_cycles(&self) -> u32 {
        1
    }
}

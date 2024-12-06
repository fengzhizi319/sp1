use num::{BigUint, One, Zero};

use sp1_curves::edwards::WORDS_FIELD_ELEMENT;
use sp1_primitives::consts::{bytes_to_words_le, words_to_bytes_le_vec, WORD_SIZE};

use crate::{
    events::{PrecompileEvent, Uint256MulEvent},
    syscalls::{Syscall, SyscallCode, SyscallContext},
};

pub(crate) struct Uint256MulSyscall;

impl Syscall for Uint256MulSyscall {
    /// 执行 Uint256 乘法系统调用。
    ///
    /// # 参数
    /// - `rt`: 系统调用上下文。
    /// - `syscall_code`: 系统调用代码。
    /// - `arg1`: 第一个参数，表示 x 值的指针。
    /// - `arg2`: 第二个参数，表示 y 值的指针。
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

        let x_ptr = arg1;
        if x_ptr % 4 != 0 {
            panic!(); // 如果 x 指针不是 4 的倍数，则触发 panic
        }
        let y_ptr = arg2;
        if y_ptr % 4 != 0 {
            panic!(); // 如果 y 指针不是 4 的倍数，则触发 panic
        }

        // 首先读取 x 值的字。我们可以在这里读取一个不安全的切片，因为稍后会将计算结果写入 x。
        let x = rt.slice_unsafe(x_ptr, WORDS_FIELD_ELEMENT);

        // 读取 y 值。
        let (y_memory_records, y) = rt.mr_slice(y_ptr, WORDS_FIELD_ELEMENT);

        // 模数存储在 y 值之后。我们通过字长度增加指针。
        let modulus_ptr = y_ptr + WORDS_FIELD_ELEMENT as u32 * WORD_SIZE as u32;
        let (modulus_memory_records, modulus) = rt.mr_slice(modulus_ptr, WORDS_FIELD_ELEMENT);

        // 获取 x、y 和模数的 BigUint 值。
        let uint256_x = BigUint::from_bytes_le(&words_to_bytes_le_vec(&x));
        let uint256_y = BigUint::from_bytes_le(&words_to_bytes_le_vec(&y));
        let uint256_modulus = BigUint::from_bytes_le(&words_to_bytes_le_vec(&modulus));

        // 执行乘法并取模数的结果。
        let result: BigUint = if uint256_modulus.is_zero() {
            let modulus = BigUint::one() << 256;
            (uint256_x * uint256_y) % modulus
        } else {
            (uint256_x * uint256_y) % uint256_modulus
        };

        let mut result_bytes = result.to_bytes_le();
        result_bytes.resize(32, 0u8); // 将结果填充到 32 字节。

        // 将结果转换为小端 u32 字。
        let result = bytes_to_words_le::<8>(&result_bytes);

        // 增加 clk 以便写入不与读取在同一个周期。
        rt.clk += 1;
        // 将结果写入 x 并跟踪内存记录。
        let x_memory_records = rt.mw_slice(x_ptr, &result);

        let lookup_id = rt.syscall_lookup_id;
        let shard = rt.current_shard();
        let event = PrecompileEvent::Uint256Mul(Uint256MulEvent {
            lookup_id,
            shard,
            clk,
            x_ptr,
            x,
            y_ptr,
            y,
            modulus,
            x_memory_records,
            y_memory_records,
            modulus_memory_records,
            local_mem_access: rt.postprocess(),
        });
        let sycall_event =
            rt.rt.syscall_event(clk, syscall_code.syscall_id(), arg1, arg2, lookup_id);
        rt.add_precompile_event(syscall_code, sycall_event, event);

        None
    }

    fn num_extra_cycles(&self) -> u32 {
        1
    }
}

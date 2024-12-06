use std::marker::PhantomData;

use num::BigUint;
use sp1_curves::{
    params::NumWords,
    weierstrass::{FieldType, FpOpField},
};
use typenum::Unsigned;

use crate::{
    events::{Fp2MulEvent, PrecompileEvent},
    syscalls::{Syscall, SyscallCode, SyscallContext},
};

pub struct Fp2MulSyscall<P> {
    _marker: PhantomData<P>,
}

impl<P> Fp2MulSyscall<P> {
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<P: FpOpField> Syscall for Fp2MulSyscall<P> {
    /// 执行 Fp2 乘法系统调用。
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

        // 检查 x 指针是否为 4 的倍数
        let x_ptr = arg1;
        if x_ptr % 4 != 0 {
            panic!();
        }

        // 检查 y 指针是否为 4 的倍数
        let y_ptr = arg2;
        if y_ptr % 4 != 0 {
            panic!();
        }

        // 获取曲线点的字数
        let num_words = <P as NumWords>::WordsCurvePoint::USIZE;

        // 读取 x 值
        let x = rt.slice_unsafe(x_ptr, num_words);
        let (y_memory_records, y) = rt.mr_slice(y_ptr, num_words);
        rt.clk += 1;

        // 将 x 和 y 分成两部分
        let (ac0, ac1) = x.split_at(x.len() / 2);
        let (bc0, bc1) = y.split_at(y.len() / 2);

        // 将分割后的部分转换为 BigUint
        let ac0 = &BigUint::from_slice(ac0);
        let ac1 = &BigUint::from_slice(ac1);
        let bc0 = &BigUint::from_slice(bc0);
        let bc1 = &BigUint::from_slice(bc1);
        let modulus = &BigUint::from_bytes_le(P::MODULUS);

        // 计算 c0 和 c1
        #[allow(clippy::match_bool)]
        let c0 = match (ac0 * bc0) % modulus < (ac1 * bc1) % modulus {
            true => ((modulus + (ac0 * bc0) % modulus) - (ac1 * bc1) % modulus) % modulus,
            false => ((ac0 * bc0) % modulus - (ac1 * bc1) % modulus) % modulus,
        };
        let c1 = ((ac0 * bc1) % modulus + (ac1 * bc0) % modulus) % modulus;

        // 将结果转换为 u32 并填充到 num_words 长度
        let mut result =
            c0.to_u32_digits().into_iter().chain(c1.to_u32_digits()).collect::<Vec<u32>>();
        result.resize(num_words, 0);

        // 将结果写入 x 并跟踪内存记录
        let x_memory_records = rt.mw_slice(x_ptr, &result);

        // 创建事件并添加到预编译事件中
        let lookup_id = rt.syscall_lookup_id;
        let shard = rt.current_shard();
        let event = Fp2MulEvent {
            lookup_id,
            shard,
            clk,
            x_ptr,
            x,
            y_ptr,
            y,
            x_memory_records,
            y_memory_records,
            local_mem_access: rt.postprocess(),
        };
        let syscall_event =
            rt.rt.syscall_event(clk, syscall_code.syscall_id(), arg1, arg2, event.lookup_id);
        match P::FIELD_TYPE {
            FieldType::Bn254 => rt.add_precompile_event(
                syscall_code,
                syscall_event,
                PrecompileEvent::Bn254Fp2Mul(event),
            ),
            FieldType::Bls12381 => rt.add_precompile_event(
                syscall_code,
                syscall_event,
                PrecompileEvent::Bls12381Fp2Mul(event),
            ),
        };
        None
    }

    fn num_extra_cycles(&self) -> u32 {
        1
    }
}

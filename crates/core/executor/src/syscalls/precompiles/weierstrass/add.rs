use std::marker::PhantomData;

use sp1_curves::{CurveType, EllipticCurve};

use crate::{
    events::{create_ec_add_event, PrecompileEvent},
    syscalls::{Syscall, SyscallCode, SyscallContext},
};

pub(crate) struct WeierstrassAddAssignSyscall<E: EllipticCurve> {
    _phantom: PhantomData<E>,
}

impl<E: EllipticCurve> WeierstrassAddAssignSyscall<E> {
    /// Create a new instance of the [`WeierstrassAddAssignSyscall`].
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<E: EllipticCurve> Syscall for WeierstrassAddAssignSyscall<E> {
    /// 执行 Weierstrass 曲线加法系统调用。
    ///
    /// # 参数
    /// - `rt`: 系统调用上下文。
    /// - `syscall_code`: 系统调用代码。
    /// - `arg1`: 第一个参数，表示第一个曲线点的指针。
    /// - `arg2`: 第二个参数，表示第二个曲线点的指针。
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
        // 创建椭圆曲线加法事件
        let event = create_ec_add_event::<E>(rt, arg1, arg2);

        // 创建系统调用事件
        let syscall_event =
            rt.rt.syscall_event(event.clk, syscall_code.syscall_id(), arg1, arg2, event.lookup_id);

        // 根据曲线类型添加预编译事件
        match E::CURVE_TYPE {
            CurveType::Secp256k1 => rt.add_precompile_event(
                syscall_code,
                syscall_event,
                PrecompileEvent::Secp256k1Add(event),
            ),
            CurveType::Bn254 => {
                rt.add_precompile_event(
                    syscall_code,
                    syscall_event,
                    PrecompileEvent::Bn254Add(event),
                );
            }
            CurveType::Bls12381 => rt.add_precompile_event(
                syscall_code,
                syscall_event,
                PrecompileEvent::Bls12381Add(event),
            ),
            CurveType::Secp256r1 => rt.record_mut().add_precompile_event(
                syscall_code,
                syscall_event,
                PrecompileEvent::Secp256r1Add(event),
            ),
            _ => panic!("Unsupported curve"), // 不支持的曲线类型
        }
        None
    }

    /// 返回系统调用执行所需的额外周期数。
    ///
    /// # 返回值
    /// 返回一个 u32 值，表示额外的周期数。
    fn num_extra_cycles(&self) -> u32 {
        1
    }
}
